use common::config::SERVER_PORT;
use common::entity::EntityState;
use common::entity_blueprint::EntityBlueprint;
use common::game_state::ServerControledGameState;
use common::ids::{BuildingLocationId, EntityId, PathId, PlayerId};
use common::level_config::BUILDING_LOCATIONS;
use common::network::{hash_client_addr, ClientMessage, ServerMessage, ServerMessageData};
use common::server_player::ServerPlayer;
use common::world::BuildingLocation;
use common::*;
use game_loop::update_entity;
use macroquad::math::Vec2;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};
mod game_loop;

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

    for (x, y) in BUILDING_LOCATIONS {
        game_state.semi_static_game_state.building_locations.insert(
            BuildingLocationId::new(),
            BuildingLocation {
                pos: Vec2 {
                    x: *x as f32,
                    y: *y as f32,
                },
                entity_id: None,
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
                        rmp_serde::from_slice::<ClientMessage>(&client_message_buf[..amt]).unwrap();
                    match command {
                        ClientMessage::PlayCard(card_id, target) => {
                            if let Some(card_from_idx) = game_state
                                .dynamic_game_state
                                .players
                                .get_mut(&client_id)
                                .unwrap()
                                .hand
                                .try_play(card_id)
                            {
                                card_from_idx.get_card_data().play_fn.exec(
                                    target,
                                    client_id,
                                    &game_state.static_game_state,
                                    &mut game_state.semi_static_game_state,
                                    &mut game_state.dynamic_game_state,
                                );
                            }
                        }
                        ClientMessage::JoinGame(deck) => {
                            if !client_addresses.contains_key(&client_id) {
                                client_addresses.insert(client_id, client_addr);
                                if let Some(available_config) = level_config::PLAYER_CONFIGS
                                    .get(game_state.dynamic_game_state.players.len())
                                {
                                    let (base_pos, available_direction, available_color) =
                                        available_config;
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
                                    server_player.hand.energy = 1;
                                    for _ in 0..4 {
                                        server_player.hand.draw();
                                    }
                                    let mut base_entity = EntityBlueprint::Base.create(client_id);
                                    base_entity.pos = *base_pos;
                                    game_state.dynamic_game_state.entities.push(base_entity);
                                }
                            }
                        }
                        ClientMessage::RequestStaticGameState => {
                            let server_message = ServerMessage {
                                metadata: game_state.game_metadata.clone(),
                                data: ServerMessageData::StaticGameState(
                                    game_state.static_game_state.clone(),
                                ),
                            };
                            let msg_vec = rmp_serde::to_vec(&server_message).unwrap();
                            udp_socket.send_to(msg_vec.as_slice(), client_addr).unwrap();
                        }
                    }
                }
            }
        }
        let mut server_messages: Vec<Vec<u8>> = Vec::new();
        let dynamic_game_state_message = ServerMessage {
            metadata: game_state.game_metadata.clone(),
            data: ServerMessageData::DynamicGameState(game_state.dynamic_game_state.clone()),
        };
        server_messages.push(rmp_serde::to_vec(&dynamic_game_state_message).unwrap());

        if game_state.semi_static_game_state.dirty {
            let semi_static_game_state_message = ServerMessage {
                metadata: game_state.game_metadata.clone(),
                data: ServerMessageData::SemiStaticGameState(
                    game_state.semi_static_game_state.clone(),
                ),
            };
            server_messages.push(rmp_serde::to_vec(&semi_static_game_state_message).unwrap());
            game_state.semi_static_game_state.dirty = false;
        }

        for (_client_id, client_addr) in &client_addresses {
            for server_message in &server_messages {
                udp_socket
                    .send_to(&server_message.as_slice(), client_addr)
                    .unwrap();
            }
        }

        game_state.game_metadata.server_tick += 1;
        for (_client_id, client) in game_state.dynamic_game_state.players.iter_mut() {
            client.hand.step(dt)
        }

        //TODO: This implementation may cause entities to not be updated if the update_entities directly removes entities.
        // This could be solved by cashing the update state of all entities, or by only killing entities by setting their state to dead.
        let mut i = 0;
        while i < game_state.dynamic_game_state.entities.len() {
            let mut entity = game_state.dynamic_game_state.entities.swap_remove(i);
            update_entity(
                &game_state.static_game_state,
                &mut game_state.semi_static_game_state,
                &mut game_state.dynamic_game_state,
                &mut entity,
                dt,
            );
            // TODO: Inserting at i causes a lot of memory movement, this can be optimized using a better swap routine for updating.
            game_state.dynamic_game_state.entities.insert(i, entity);
            i += 1;
        }

        let mut i = 0;
        while i < game_state.dynamic_game_state.entities.len() {
            let entity = &game_state.dynamic_game_state.entities.get(i).unwrap();
            if entity.state == EntityState::Dead {
                cleanup_entity(entity.id, &mut game_state);
                game_state.dynamic_game_state.entities.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }
}

fn cleanup_entity(
    entity_id: EntityId,
    server_controlled_game_state: &mut ServerControledGameState,
) {
    if let Some((_id, building_location)) = server_controlled_game_state
        .semi_static_game_state
        .building_locations
        .iter_mut()
        .find(|(_id, building_location)| building_location.entity_id == Some(entity_id))
    {
        building_location.entity_id = None;
    }
}
