use common::level_config::BUILDING_LOCATIONS;
use common::*;
use rand::Rng;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};
mod game_loop;

fn main() -> std::io::Result<()> {
    let mut rng = rand::thread_rng();
    let mut game_state = ServerGameState::new();
    let mut client_addresses = HashMap::<u64, SocketAddr>::new();

    for path in level_config::PATHS {
        game_state.static_state.paths.insert(
            rng.gen(),
            path.to_vec()
                .iter()
                .map(|(x, y)| (*x as f32, *y as f32))
                .collect(),
        );
    }

    for (x, y) in BUILDING_LOCATIONS {
        game_state.dynamic_state.building_locations.insert(
            rng.gen(),
            BuildingLocation {
                position: (*x as f32, *y as f32),
                building: None,
            },
        );
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
            let client_message_buf = &mut [0; 200];
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
                        ClientCommand::PlayCard(card, target) => {
                            card.get_card_data().play_fn.exec(
                                target,
                                client_id,
                                &game_state.static_state,
                                &mut game_state.dynamic_state,
                            );
                        }
                        ClientCommand::JoinGame => {
                            if !client_addresses.contains_key(&client_id) {
                                client_addresses.insert(client_id, client_addr);
                                if let Some(available_config) = level_config::PLAYER_CONFIGS
                                    .get(game_state.dynamic_state.players.len())
                                {
                                    let (base_pos, available_direction, available_color) =
                                        available_config;
                                    game_state.dynamic_state.players.insert(
                                        client_id,
                                        ServerPlayer::new(
                                            available_direction.clone(),
                                            *available_color,
                                        ),
                                    );
                                    game_state.dynamic_state.entities.insert(
                                        client_id,
                                        Entity {
                                            owner: client_id,
                                            pos: base_pos.clone(),
                                            tag: EntityTag::Base,
                                            behavior: Behavior::None,
                                            radius: 0.5,
                                            health: 1000.0,
                                            damage_animation: 0.0,
                                            usable_as_spawn_point: true,
                                            ranged_attack: None,
                                            melee_attack: None,
                                            seconds_left_to_live: None,
                                        },
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
            client.card_draw_counter += dt / 12.0;
            client.energy_counter += dt / 8.0;
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
