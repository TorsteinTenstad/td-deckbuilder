use common::card::Card;
use common::*;
use image::GenericImageView;
use macroquad::prelude::{Vec2, PURPLE, YELLOW};
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

    let get_pos = |entity: &Entity| -> Vec2 {
        match entity.movement {
            Kinematics::Path(PathKinematics { path_pos, .. }) => {
                game_state.static_state.path_to_world_pos(path_pos)
            }
            Kinematics::Static(StaticKinematics { pos }) => pos,
            Kinematics::Free(FreeKinematics { pos, .. }) => pos,
        }
    };

    let udp_socket = UdpSocket::bind(SERVER_ADDR).unwrap();
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();
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
                let client_id = hash_client_addr(&client_addr);
                let player = game_state.dynamic_state.players.get_mut(&client_id);
                let command =
                    serde_json::from_slice::<ClientCommand>(&client_message_buf[..amt]).unwrap();
                match command {
                    ClientCommand::PlayCard(x, y, card) => match card {
                        Card::Unit => {
                            let player = player.unwrap();
                            game_state.dynamic_state.entities.insert(
                                rng.gen::<u64>(),
                                Entity {
                                    owner: client_id,
                                    movement: Kinematics::Path {
                                        0: PathKinematics {
                                            direction: player.direction.clone(),
                                            speed: 1.0,
                                            path_pos: match player.direction {
                                                Direction::Positive => 0.0,
                                                Direction::Negative => {
                                                    game_state.static_state.path.len() as f32
                                                }
                                            },
                                        },
                                    },
                                    radius: 0.25,
                                    health: 100.0,
                                    damage_animation: 0.0,
                                    ranged_attack: None,
                                    seconds_left_to_live: None,
                                },
                            );
                        }
                        Card::Tower => {
                            println!("tower at {}, {}", x, y);
                            game_state.dynamic_state.entities.insert(
                                rng.gen::<u64>(),
                                Entity {
                                    owner: client_id,
                                    movement: Kinematics::Static {
                                        0: StaticKinematics {
                                            pos: Vec2 {
                                                x: x as f32,
                                                y: y as f32,
                                            },
                                        },
                                    },
                                    radius: 0.25,
                                    health: 100.0,
                                    damage_animation: 0.0,
                                    ranged_attack: Some(RangedAttack {
                                        range: 3.0,
                                        damage: 50.0,
                                        cooldown: 0.5,
                                        last_fire: 0.0,
                                    }),
                                    seconds_left_to_live: None,
                                },
                            );
                        }
                    },
                    ClientCommand::JoinGame => {
                        let client_id = hash_client_addr(&client_addr);
                        if !client_addresses.contains_key(&client_id) {
                            client_addresses.insert(client_id, client_addr);
                            if let Some(available_config) =
                                vec![(Direction::Positive, YELLOW), (Direction::Negative, PURPLE)]
                                    .get(game_state.dynamic_state.players.len())
                            {
                                let (available_direction, available_color) = available_config;
                                game_state.dynamic_state.players.insert(
                                    client_id,
                                    Player::new(available_direction.clone(), *available_color),
                                );
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
            client.card_draw_counter += dt;
        }

        let mut new_entities = HashMap::<u64, Entity>::new();
        game_state.dynamic_state.entities = game_state
            .dynamic_state
            .entities
            .iter()
            .filter_map(|(id, entity)| {
                let owner = game_state.dynamic_state.players.get(&entity.owner).unwrap();
                let entity_pos = get_pos(entity);
                let mut entity = entity.clone();

                match &mut entity.movement {
                    Kinematics::Path(PathKinematics {
                        path_pos,
                        direction,
                        speed,
                    }) => {
                        if !game_state.dynamic_state.entities.iter().any(
                            |(other_id, other_entity)| {
                                id != other_id
                                    && (get_pos(other_entity) - entity_pos).length_squared()
                                        < (entity.radius + other_entity.radius).powi(2)
                            },
                        ) {
                            *path_pos += *speed * direction.to_f32() * dt;
                        }
                    }
                    Kinematics::Static(StaticKinematics { pos: _ }) => {}
                    Kinematics::Free(FreeKinematics {
                        mut pos,
                        mut velocity,
                        target_entity_id,
                        speed,
                    }) => {
                        velocity = target_entity_id
                            .and_then(|target_entity_id| {
                                game_state
                                    .dynamic_state
                                    .entities
                                    .get(&target_entity_id)
                                    .map(|target_entity| {
                                        (get_pos(target_entity) - pos).normalize_or_zero() * *speed
                                    })
                            })
                            .unwrap_or(velocity);

                        pos += velocity * dt;
                    }
                };

                match entity.ranged_attack.as_mut() {
                    Some(RangedAttack {
                        range,
                        damage,
                        cooldown,
                        mut last_fire,
                    }) => {
                        if last_fire < 0.0 {
                            if let Some((target_entity_id, _entity)) = game_state
                                .dynamic_state
                                .entities
                                .iter()
                                .filter(|(_, entity)| entity.owner != entity.owner)
                                .map(|(id, entity)| {
                                    (id, (entity_pos - get_pos(entity)).length_squared())
                                })
                                .min_by(|(_, length_squared_a), (_, length_squared_b)| {
                                    length_squared_a.partial_cmp(length_squared_b).unwrap()
                                })
                                .filter(|(id, length_squared)| length_squared < &range.powi(2))
                            {
                                last_fire = *cooldown;
                                new_entities.insert(
                                    rng.gen::<u64>(),
                                    Entity {
                                        owner: entity.owner,
                                        movement: Kinematics::Free(FreeKinematics {
                                            pos: entity_pos,
                                            velocity: Vec2::new(0.0, 0.0),
                                            target_entity_id: Some(target_entity_id.clone()),
                                            speed: 5.0,
                                        }),
                                        seconds_left_to_live: Some(3.0),
                                        radius: PROJECTILE_RADIUS,
                                        health: 1.0,
                                        damage_animation: 0.0,
                                        ranged_attack: None,
                                    },
                                );
                            }
                        } else {
                            last_fire -= dt;
                        }
                    }
                    None => {}
                }

                entity.damage_animation -= dt;
                if let Some(seconds_left_to_live) = &mut entity.seconds_left_to_live {
                    *seconds_left_to_live -= dt;
                    if seconds_left_to_live < &mut 0.0 {
                        entity.health = 0.0;
                    }
                }
                (entity.health > 0.0).then_some((id.clone(), entity))
            })
            .collect();
    }
}
