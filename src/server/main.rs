use common::config::SERVER_PORT;
use common::entity::EntityState;
use common::entity_blueprint::EntityBlueprint;
use common::game_state::ServerControledGameState;
use common::gameplay_config::{STARTING_ENERGY, STARTING_HAND_SIZE};
use common::ids::{BuildingLocationId, PathId, PlayerId};
use common::level_config::BUILDING_LOCATIONS;
use common::message_acknowledgement::AckUdpSocket;
use common::network::{hash_client_addr, ClientMessage, ServerMessage, ServerMessageData};
use common::server_player::ServerPlayer;
use common::world::BuildingLocation;
use common::*;
use itertools::Itertools;
use macroquad::math::Vec2;
use std::collections::hash_map;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};

fn main() -> std::io::Result<()> {
    let mut game_state = ServerControledGameState::default();
    let mut client_addresses = HashMap::<PlayerId, SocketAddr>::new();

    for path in level_config::PATHS {
        game_state.static_game_state.paths.insert(
            PathId::new(),
            path.to_vec()
                .iter()
                .map(|(x, y)| (*x as f32, *y as f32))
                .collect(),
        );
    }

    for (zoning, (x, y)) in BUILDING_LOCATIONS {
        game_state
            .semi_static_game_state
            .building_locations_mut()
            .insert(
                BuildingLocationId::new(),
                BuildingLocation {
                    pos: Vec2 {
                        x: *x as f32,
                        y: *y as f32,
                    },
                    entity_id: None,
                    zoning: zoning.clone(),
                },
            );
    }

    let server_ip = local_ip_address::local_ip()
        .map(|ip| format!("{}:{}", ip, SERVER_PORT))
        .unwrap();
    let udp_socket = UdpSocket::bind(server_ip).unwrap();
    println!("Server started on {}", udp_socket.local_addr().unwrap());
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();
    let mut ack_udp_socket =
        AckUdpSocket::<ServerMessage, ClientMessage>::new(udp_socket, Duration::from_secs(1));
    let mut time = SystemTime::now();

    loop {
        let old_time = time;
        time = SystemTime::now();
        let dt = time.duration_since(old_time).unwrap().as_secs_f32();

        while let Some((client_message, client_addr)) = ack_udp_socket.receive() {
            let client_id = hash_client_addr(&client_addr);
            match client_message {
                ClientMessage::PlayCard(card_id, target) => {
                    if let Some(card_from_idx) = &mut game_state
                        .dynamic_game_state
                        .players
                        .get_mut(&client_id)
                        .unwrap()
                        .hand
                        .try_get(card_id)
                    {
                        let played = card_from_idx.get_card_data().play_fn.exec(
                            target,
                            client_id,
                            &game_state.static_game_state,
                            &mut game_state.semi_static_game_state,
                            &mut game_state.dynamic_game_state,
                        );

                        if played {
                            game_state
                                .dynamic_game_state
                                .players
                                .get_mut(&client_id)
                                .unwrap()
                                .hand
                                .play(card_id);
                        }
                    }
                }
                ClientMessage::JoinGame(deck) => {
                    if let hash_map::Entry::Vacant(vacant_entry) = client_addresses.entry(client_id)
                    {
                        vacant_entry.insert(client_addr);
                        if let Some(available_config) = level_config::PLAYER_CONFIGS
                            .get(game_state.dynamic_game_state.players.len())
                        {
                            let (base_pos, available_direction, available_color) = available_config;
                            game_state.dynamic_game_state.players.insert(
                                client_id,
                                ServerPlayer::new(
                                    available_direction.clone(),
                                    *available_color,
                                    deck,
                                ),
                            );
                            let server_player = game_state
                                .dynamic_game_state
                                .players
                                .get_mut(&client_id)
                                .unwrap();
                            server_player.hand.energy = STARTING_ENERGY;
                            for _ in 0..STARTING_HAND_SIZE {
                                server_player.hand.draw();
                            }
                            let mut base_entity = EntityBlueprint::Base
                                .create()
                                .instantiate(client_id, EntityState::Passive);
                            base_entity.pos = *base_pos;
                            game_state.dynamic_game_state.entities.push(base_entity);
                        }
                    }
                    ack_udp_socket.send_to(
                        ServerMessage {
                            metadata: game_state.game_metadata.clone(),
                            data: ServerMessageData::StaticGameState(
                                game_state.static_game_state.clone(),
                            ),
                        },
                        &client_addr,
                        true,
                    );
                    ack_udp_socket.send_to(
                        ServerMessage {
                            metadata: game_state.game_metadata.clone(),
                            data: ServerMessageData::SemiStaticGameState(
                                game_state.semi_static_game_state.clone(),
                            ),
                        },
                        &client_addr,
                        true,
                    );
                }
            }
        }

        for client_addr in client_addresses.values() {
            ack_udp_socket.send_to(
                ServerMessage {
                    metadata: game_state.game_metadata.clone(),
                    data: ServerMessageData::DynamicGameState(
                        game_state.dynamic_game_state.clone(),
                    ),
                },
                client_addr,
                false,
            );
        }
        if game_state.semi_static_game_state.dirty {
            game_state.semi_static_game_state.dirty = false;
            for client_addr in client_addresses.values() {
                ack_udp_socket.send_to(
                    ServerMessage {
                        metadata: game_state.game_metadata.clone(),
                        data: ServerMessageData::SemiStaticGameState(
                            game_state.semi_static_game_state.clone(),
                        ),
                    },
                    client_addr,
                    false,
                );
            }
        }

        game_state.game_metadata.server_tick += 1;
        for (client_id, client) in game_state.dynamic_game_state.players.iter_mut() {
            let draw_speed_buffs = game_state
                .dynamic_game_state
                .entities
                .iter()
                .filter_map(|entity_instance| {
                    if entity_instance.owner != *client_id {
                        return None;
                    }
                    entity_instance.entity.draw_speed_buff.clone()
                })
                .collect_vec();
            let energy_generation_buffs = game_state
                .dynamic_game_state
                .entities
                .iter()
                .filter_map(|entity_instance| {
                    if entity_instance.owner != *client_id {
                        return None;
                    }
                    entity_instance.entity.energy_generation_buff.clone()
                })
                .collect_vec();
            client
                .hand
                .step(dt, &draw_speed_buffs, &energy_generation_buffs);
        }

        game_loop::update_game_state(&mut game_state, dt);
    }
}
