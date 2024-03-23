pub mod test {
    use crate::{condition::Condition, TestMonitorPing, TEST_CLIENT_ADDR};
    use common::{
        card::Card,
        entity::Entity,
        entity_blueprint::EntityBlueprint,
        game_loop,
        game_state::ServerControlledGameState,
        get_unit_spawnpoints::get_unit_spawnpoints,
        ids::{BuildingLocationId, EntityId, PathId, PlayerId},
        message_acknowledgement::AckUdpSocket,
        network::{
            send_dynamic_game_state, send_semi_static_game_state, send_static_game_state,
            ServerMessage,
        },
        play_target::PlayFn,
        server_player::ServerPlayer,
        world::{world_place_path_entity, BuildingLocation, Direction, Zoning},
    };
    use macroquad::{
        color::{BLUE, RED},
        math::Vec2,
    };
    use std::{
        net::{Ipv4Addr, SocketAddr, UdpSocket},
        thread::sleep,
        time::Duration,
    };

    const SIMULATION_FPS: f32 = 60.0;
    const SIMULATION_DT: f32 = 1.0 / SIMULATION_FPS;

    struct TestEnvironmentNetworkState {
        ack_udp_socket:
            common::message_acknowledgement::AckUdpSocket<ServerMessage, TestMonitorPing>,
        client_addr: SocketAddr,
        has_received_ping: bool,
    }

    impl Default for TestEnvironmentNetworkState {
        fn default() -> Self {
            let udp_socket = std::iter::successors(Some(6968), |port| Some(port + 1))
                .find_map(|port| {
                    let socket_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), port);
                    UdpSocket::bind(socket_addr).ok()
                })
                .unwrap();
            udp_socket.set_nonblocking(true).unwrap();
            Self {
                ack_udp_socket: AckUdpSocket::new(udp_socket, Duration::from_secs(1)),
                client_addr: TEST_CLIENT_ADDR.parse().unwrap(),
                has_received_ping: false,
            }
        }
    }

    impl TestEnvironmentNetworkState {
        pub fn send_init(&mut self, state: &ServerControlledGameState) {
            send_static_game_state(&mut self.ack_udp_socket, state, &self.client_addr);
        }
        pub fn send_update(&mut self, state: &ServerControlledGameState) {
            self.has_received_ping |= self.ack_udp_socket.receive().is_some();
            send_semi_static_game_state(&mut self.ack_udp_socket, state, &self.client_addr);
            send_dynamic_game_state(&mut self.ack_udp_socket, state, &self.client_addr);
        }
    }

    pub struct TestEnvironment {
        network_state: TestEnvironmentNetworkState,
        pub state: ServerControlledGameState,
        pub player_a: PlayerId,
        pub player_b: PlayerId,
        pub speed: f32,
        pub sim_time_s: f32,
        pub timeout_s: f32,
    }

    impl Default for TestEnvironment {
        fn default() -> Self {
            Self::new()
        }
    }

    impl TestEnvironment {
        pub fn new() -> Self {
            let mut test_environment = Self {
                network_state: TestEnvironmentNetworkState::default(),
                state: ServerControlledGameState::default(),
                player_a: PlayerId::new(),
                player_b: PlayerId::new(),
                speed: 1.0,
                sim_time_s: 0.0,
                timeout_s: 1000.0,
            };

            let path = vec![(100.0, 100.0), (1100.0, 100.0)];

            for (player_id, base_pos, direction, color) in &[
                (
                    test_environment.player_a,
                    Vec2::new(path[0].0, path[0].1),
                    Direction::Positive,
                    RED,
                ),
                (
                    test_environment.player_b,
                    Vec2::new(path[1].0, path[1].1),
                    Direction::Negative,
                    BLUE,
                ),
            ] {
                test_environment.state.dynamic_game_state.players.insert(
                    *player_id,
                    ServerPlayer::new(direction.clone(), *color, Vec::new()),
                );
                let base_entity = EntityBlueprint::Base
                    .create()
                    .instantiate(*player_id, *base_pos);
                test_environment
                    .state
                    .dynamic_game_state
                    .entities
                    .push(base_entity);
            }
            test_environment
                .state
                .static_game_state
                .paths
                .insert(PathId::new(), path);
            test_environment
        }
    }

    pub struct Timeout {}

    impl TestEnvironment {
        pub fn simulate_until(&mut self, condition: Condition) -> Result<(), Timeout> {
            self.network_state.send_init(&self.state);
            while !condition.is_met(self) {
                let dt = SIMULATION_DT;
                self.sim_time_s += dt;
                game_loop::update_game_state(&mut self.state, dt);
                self.network_state.send_update(&self.state);
                if self.network_state.has_received_ping {
                    sleep(Duration::from_secs_f32(dt / self.speed));
                }
                if self.sim_time_s > self.timeout_s {
                    return Err(Timeout {});
                }
            }
            Ok(())
        }
        pub fn play_entity(&mut self, player_id: PlayerId, entity: Entity) -> Option<EntityId> {
            let spawnpoint = get_unit_spawnpoints(
                player_id,
                &self.state.static_game_state,
                &self.state.dynamic_game_state,
            )
            .first()
            .unwrap()
            .clone();
            world_place_path_entity(
                &self.state.static_game_state,
                &mut self.state.dynamic_game_state,
                spawnpoint,
                entity,
                player_id,
            )
        }
        pub fn play_card(&mut self, player_id: PlayerId, card: Card) {
            match &card.get_card_data().play_fn {
                PlayFn::UnitSpawnPoint(specific_play_fn) => {
                    let spawnpoints = get_unit_spawnpoints(
                        player_id,
                        &self.state.static_game_state,
                        &self.state.dynamic_game_state,
                    );
                    let target = spawnpoints.first().unwrap();
                    let in_valid = specific_play_fn.target_is_invalid.is_some_and(|f| {
                        f(
                            target,
                            player_id,
                            &self.state.static_game_state,
                            &self.state.semi_static_game_state,
                            &self.state.dynamic_game_state,
                        )
                    });
                    assert!(!in_valid);
                    (specific_play_fn.play)(
                        target.clone(),
                        player_id,
                        &self.state.static_game_state,
                        &mut self.state.semi_static_game_state,
                        &mut self.state.dynamic_game_state,
                    );
                }
                PlayFn::BuildingSpot(_) => {
                    todo!()
                }
                PlayFn::WorldPos(_) => {
                    todo!()
                }
                PlayFn::Entity(_) => {
                    todo!()
                }
            }
        }
    }
}
