use common::config::SERVER_ADDR;
use common::entity::{Entity, EntityState, EntityTag};
use common::game_state::ServerGameState;
use common::ids::{BuildingLocationId, EntityId, PathId, PlayerId};
use common::level_config::BUILDING_LOCATIONS;
use common::network::{hash_client_addr, ClientCommand};
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
    let mut game_state = ServerGameState::new();
    let mut client_addresses = HashMap::<PlayerId, SocketAddr>::new();

    for path in level_config::PATHS {
        game_state.static_state.paths.insert(
            PathId::new(),
            path.to_vec()
                .iter()
                .map(|(x, y)| (*x as f32, *y as f32))
                .collect(),
        );
    }

    for (x, y) in BUILDING_LOCATIONS {
        game_state.dynamic_state.building_locations.insert(
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
                        ClientCommand::PlayCard(card_id, target) => {
                            if let Some(card_from_idx) = game_state
                                .dynamic_state
                                .players
                                .get_mut(&client_id)
                                .unwrap()
                                .hand
                                .try_play(card_id)
                            {
                                card_from_idx.get_card_data().play_fn.exec(
                                    target,
                                    client_id,
                                    &game_state.static_state,
                                    &mut game_state.dynamic_state,
                                );
                            }
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
                                    let server_player = game_state
                                        .dynamic_state
                                        .players
                                        .get_mut(&client_id)
                                        .unwrap();
                                    server_player.hand.energy = 10;
                                    for _ in 0..7 {
                                        server_player.hand.draw();
                                    }
                                    let mut base_entity = Entity::new(
                                        EntityTag::Base,
                                        client_id,
                                        EntityState::Attacking,
                                    );
                                    base_entity.pos = *base_pos;
                                    base_entity.radius = 48.0;
                                    base_entity.health = 1000.0;
                                    base_entity.hitbox_radius = 150.0;
                                    game_state.dynamic_state.entities.push(base_entity);
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
            client.hand.step(dt)
        }

        let mut new_entities: Vec<Entity> = Vec::new();
        let mut entity_ids_to_remove: Vec<EntityId> = Vec::new();

        for i in 0..game_state.dynamic_state.entities.len() {
            let mut entity = game_state.dynamic_state.entities.swap_remove(i);
            update_entity(
                &mut entity,
                &mut game_state.dynamic_state.entities,
                dt,
                &game_state.static_state,
                &mut new_entities,
                &mut entity_ids_to_remove,
            );
            game_state.dynamic_state.entities.insert(i, entity);
        }
        game_state.dynamic_state.entities.append(&mut new_entities);
        for entity_id in entity_ids_to_remove.iter() {
            cleanup_entity(*entity_id, &mut game_state);
        }
        game_state
            .dynamic_state
            .entities
            .retain(|entity| !entity_ids_to_remove.contains(&entity.id));
    }
}

fn cleanup_entity(entity_id: EntityId, game_state: &mut ServerGameState) {
    if let Some((_id, building_location)) = game_state
        .dynamic_state
        .building_locations
        .iter_mut()
        .find(|(_id, building_location)| building_location.entity_id == Some(entity_id))
    {
        building_location.entity_id = None;
    }
}
