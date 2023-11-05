use common::card::Card;
use common::*;
use image::GenericImageView;
use macroquad::prelude::Vec2;
use rand::Rng;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};

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
    let mut next_unit_id = 0;
    let mut time = SystemTime::now();
    loop {
        let old_time = time;
        time = SystemTime::now();
        let dt = time.duration_since(old_time).unwrap().as_secs_f32();

        let client_message_buf = &mut [0; 50];
        let read_client_message = udp_socket.recv_from(client_message_buf);
        match read_client_message {
            Err(e) => match e.kind() {
                std::io::ErrorKind::ConnectionReset => {}
                std::io::ErrorKind::TimedOut => {}
                _ => {
                    dbg!(e);
                    panic!()
                }
            },
            Ok((amt, client_addr)) => {
                let command =
                    serde_json::from_slice::<ClientCommand>(&client_message_buf[..amt]).unwrap();
                match command {
                    ClientCommand::PlayCard(x, y, card) => match card {
                        Card::Unit => {
                            game_state.dynamic_state.units.insert(
                                next_unit_id,
                                Unit {
                                    path_pos: 0.0,
                                    speed: 1.0,
                                    health: 100.0,
                                    damage_animation: 0.0,
                                },
                            );
                            next_unit_id += 1;
                        }
                        Card::Tower => {
                            println!("tower at {}, {}", x, y);
                            game_state.dynamic_state.towers.insert(
                                rng.gen::<u64>(),
                                Tower {
                                    pos_x: x as i32,
                                    pos_y: y as i32,
                                    range: 3.0,
                                    damage: 50.0,
                                    cooldown: 0.5,
                                    last_fire: 0.0,
                                },
                            );
                        }
                    },
                    ClientCommand::JoinGame => {
                        let client_id = hash_client_addr(&client_addr);
                        if !game_state.dynamic_state.clients.contains_key(&client_id) {
                            game_state
                                .dynamic_state
                                .clients
                                .insert(client_id, Client::new());
                            client_addresses.insert(client_id, client_addr);
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
        for (_client_id, client) in game_state.dynamic_state.clients.iter_mut() {
            client.card_draw_counter += dt;
        }

        let mut units_to_remove = Vec::<u64>::new();
        for (id, unit) in game_state.dynamic_state.units.iter_mut() {
            if unit.health <= 0.0 && !units_to_remove.contains(id) {
                units_to_remove.push(*id);
            } else {
                unit.path_pos += unit.speed * dt;
                unit.damage_animation -= dt;
            }
        }
        for id in units_to_remove {
            game_state.dynamic_state.units.remove(&id);
        }

        for (_id, tower) in game_state.dynamic_state.towers.iter_mut() {
            let tower_pos = Vec2 {
                x: tower.pos_x as f32,
                y: tower.pos_y as f32,
            };
            if tower.last_fire < 0.0 {
                if let Some((id, _unit)) = game_state
                    .dynamic_state
                    .units
                    .iter()
                    .filter(|(_, unit)| {
                        (tower_pos - game_state.static_state.path_to_world_pos(unit.path_pos))
                            .length()
                            < tower.range
                    })
                    .min_by_key(|(_, unit)| {
                        (game_state.static_state.path_to_world_pos(unit.path_pos) - tower_pos)
                            .length_squared();
                    })
                {
                    tower.last_fire = tower.cooldown;
                    game_state.dynamic_state.projectiles.push(Projectile {
                        pos: tower_pos,
                        target_id: *id,
                        speed: 5.0,
                        velocity: Vec2::new(0.0, 0.0),
                        damage: tower.damage,
                        seconds_left_to_live: 3.0,
                    });
                }
            } else {
                tower.last_fire -= dt;
            }
        }
        game_state
            .dynamic_state
            .projectiles
            .retain_mut(|projectile| {
                if let Some(target_unit) = game_state
                    .dynamic_state
                    .units
                    .get_mut(&projectile.target_id)
                {
                    projectile.velocity = (game_state
                        .static_state
                        .path_to_world_pos(target_unit.path_pos)
                        - projectile.pos)
                        .normalize_or_zero()
                        * projectile.speed;
                }
                projectile.pos += projectile.velocity * dt;
                for (_id, unit) in game_state.dynamic_state.units.iter_mut() {
                    if (game_state.static_state.path_to_world_pos(unit.path_pos) - projectile.pos)
                        .length()
                        < UNIT_RADIUS + PROJECTILE_RADIUS
                    {
                        unit.health -= projectile.damage;
                        unit.damage_animation = 0.05;
                        return false;
                    }
                }
                projectile.seconds_left_to_live -= dt;
                projectile.seconds_left_to_live > 0.0
            });
    }
}
