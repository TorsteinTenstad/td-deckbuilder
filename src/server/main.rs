use common::*;
use image::GenericImageView;
use macroquad::prelude::{PURPLE, YELLOW};
use rand::Rng;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};
mod game_loop;

fn main() -> std::io::Result<()> {
    let mut rng = rand::thread_rng();
    let mut game_state = GameState::new();
    let mut client_addresses = HashMap::<u64, SocketAddr>::new();

    let img = image::open("path.png").unwrap();
    game_state.static_state.grid_w = img.dimensions().0;
    game_state.static_state.grid_h = img.dimensions().1;

    let is_path = |x: i32, y: i32| match (x.try_into(), y.try_into()) {
        (Ok(x), Ok(y))
            if x < game_state.static_state.grid_w && y < game_state.static_state.grid_h =>
        {
            img.get_pixel(x, y).0.get(0).is_some_and(|v| v > &0)
        }
        _ => false,
    };

    let path_start = (0..game_state.static_state.grid_w as i32)
        .into_iter()
        .flat_map(|x| (0..game_state.static_state.grid_w as i32).map(move |y| (x, y)))
        .find_map(|(x, y)| {
            (is_path(x, y)
                && (is_path(x - 1, y) as i32
                    + is_path(x, y - 1) as i32
                    + is_path(x + 1, y) as i32
                    + is_path(x, y + 1) as i32)
                    <= 1)
                .then(|| (x, y))
        });

    let (mut x, mut y) = path_start.unwrap();
    game_state.static_state.path = vec![(x, y)];
    while let Some(next) = vec![(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
        .into_iter()
        .find_map(|next_xy| {
            (is_path(next_xy.0, next_xy.1) && !game_state.static_state.path.contains(&next_xy))
                .then(|| next_xy)
        })
    {
        game_state.static_state.path.push(next);
        (x, y) = next;
    }

    let udp_socket = UdpSocket::bind(SERVER_ADDR).unwrap();
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();
    let mut time = SystemTime::now();
    loop {
        let old_time = time;
        time = SystemTime::now();
        let dt = time.duration_since(old_time).unwrap().as_secs_f32();

        loop {
            let client_message_buf = &mut [0; 50];
            let read_client_message = udp_socket.recv_from(client_message_buf);
            match read_client_message {
                Err(e) => match e.kind() {
                    std::io::ErrorKind::ConnectionReset => {}
                    std::io::ErrorKind::TimedOut => {
                        break;
                    }
                    _ => {
                        dbg!(e);
                        panic!()
                    }
                },
                Ok((amt, client_addr)) => {
                    let client_id = hash_client_addr(&client_addr);
                    let command =
                        serde_json::from_slice::<ClientCommand>(&client_message_buf[..amt])
                            .unwrap();
                    match command {
                        ClientCommand::PlayCard(x, y, card) => {
                            let player = game_state
                                .dynamic_state
                                .players
                                .get_mut(&client_id)
                                .unwrap();
                            game_state
                                .dynamic_state
                                .entities
                                .insert(rng.gen(), card.to_entity(client_id, player, x, y));
                        }
                        ClientCommand::JoinGame => {
                            let client_id = hash_client_addr(&client_addr);
                            if !client_addresses.contains_key(&client_id) {
                                client_addresses.insert(client_id, client_addr);
                                if let Some(available_config) = vec![
                                    (Direction::Positive, YELLOW),
                                    (Direction::Negative, PURPLE),
                                ]
                                .get(game_state.dynamic_state.players.len())
                                {
                                    let (available_direction, available_color) = available_config;
                                    game_state.dynamic_state.players.insert(
                                        client_id,
                                        Player::new(
                                            available_direction.clone(),
                                            game_state.static_state.path_to_world_pos(
                                                available_direction.to_start_path_pos(),
                                            ),
                                            *available_color,
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        let msg = serde_json::to_string(&game_state).unwrap();
        for (_client_id, client_addr) in &client_addresses {
            udp_socket.send_to(msg.as_bytes(), client_addr).unwrap();
        }

        game_state.dynamic_state.server_tick += 1;
        for (_client_id, client) in game_state.dynamic_state.players.iter_mut() {
            client.card_draw_counter += dt / 5.0;
            client.energy_counter += dt / 3.0;
        }

        let mut other_entities_external_effects = HashMap::<u64, EntityExternalEffects>::new();
        game_state.dynamic_state.entities = game_state
            .dynamic_state
            .entities
            .iter()
            .flat_map(|(id, entity)| {
                game_loop::update_entity(
                    &id,
                    &entity,
                    &game_state.dynamic_state.entities,
                    &mut other_entities_external_effects,
                    dt,
                    &game_state.static_state,
                    &mut rng,
                )
            })
            .collect();
        for (id, entity) in game_state.dynamic_state.entities.iter_mut() {
            if let Some(external_effects) = other_entities_external_effects.get(id) {
                entity.health += external_effects.health;
                entity.damage_animation = 0.1;
            }
        }
    }
}
