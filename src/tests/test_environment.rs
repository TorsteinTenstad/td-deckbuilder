pub mod test {
    use crate::{condition::Condition, TestMonitorPing, TEST_CLIENT_ADDR};
    use common::{
        card::Card,
        entity::{Entity, EntityInstance},
        entity_blueprint::EntityBlueprint,
        game_loop,
        game_state::ServerControlledGameState,
        get_unit_spawnpoints::get_unit_spawnpoints,
        ids::{BuildingLocationId, EntityId, PlayerId},
        level_config::LevelConfig,
        message_acknowledgement::AckUdpSocket,
        network::{
            send_dynamic_game_state, send_semi_static_game_state, send_static_game_state,
            ServerMessage,
        },
        play_target::{BuildingLocationTarget, PlayArgs, PlayFn, PlayTarget, WorldPosTarget},
        server_player::ServerPlayer,
        world::{
            find_entity, world_place_building, world_place_path_entity, BuildingLocation,
            Direction, Zoning,
        },
    };
    use macroquad::{
        color::{BLUE, RED},
        math::Vec2,
    };
    use std::{
        iter::zip,
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
        pub percistent_condtions: Vec<(Condition, bool)>,
    }

    impl Default for TestEnvironment {
        fn default() -> Self {
            Self::new(Self::default_level_config())
        }
    }

    impl TestEnvironment {
        pub fn default_level_config() -> LevelConfig {
            LevelConfig {
                level_width: 1200,
                level_height: 400,
                spawn_point_radius: 256.0,
                nearby_radius: 256.0,
                player_configs: vec![
                    (Vec2::new(50.0, 200.0), Direction::Positive, RED),
                    (Vec2::new(1150.0, 200.0), Direction::Negative, BLUE),
                ],
                building_locations: vec![(Zoning::Normal, (600.0, 100.0))],
                paths: vec![vec![(100.0, 200.0), (1100.0, 200.0)]],
            }
        }
        pub fn new(level_config: LevelConfig) -> Self {
            let mut test_environment = Self {
                network_state: TestEnvironmentNetworkState::default(),
                state: ServerControlledGameState::default(),
                player_a: PlayerId::new(),
                player_b: PlayerId::new(),
                speed: 1.0,
                sim_time_s: 0.0,
                timeout_s: 120.0,
                percistent_condtions: Vec::new(),
            };

            for (player_id, (base_pos, direction, color)) in zip(
                [test_environment.player_a, test_environment.player_b],
                &level_config.player_configs,
            ) {
                test_environment.state.dynamic_game_state.players.insert(
                    player_id,
                    ServerPlayer::new(direction.clone(), *color, Vec::new()),
                );
                let mut base_entity = EntityBlueprint::Base
                    .create()
                    .instantiate(player_id, *base_pos);
                base_entity.entity.health.health = 1.0;
                test_environment
                    .state
                    .dynamic_game_state
                    .entities
                    .spawn(base_entity);
            }
            test_environment.state.load_level_config(level_config);
            test_environment
                .network_state
                .send_init(&test_environment.state);
            test_environment
        }
    }

    pub enum SimulationBreak {
        Timeout,
        PercistentConditionFail,
    }

    impl TestEnvironment {
        pub fn simulate_for(&mut self, simulated_s: f32) -> Result<(), SimulationBreak> {
            let break_sim_time_s = self.sim_time_s + simulated_s;
            self.simulate(|env| env.sim_time_s >= break_sim_time_s)
        }
        pub fn simulate_until(&mut self, condition: Condition) -> Result<(), SimulationBreak> {
            self.simulate(|env| condition.is_met(env))
        }
        pub fn simulate_frame(&mut self) -> Result<(), SimulationBreak> {
            self.simulate(|_| true)
        }
        pub fn simulate_frames(&mut self, frames: usize) -> Result<(), SimulationBreak> {
            for _ in 0..frames {
                self.simulate_frame()?;
            }
            Ok(())
        }
        pub fn simulate<P>(&mut self, break_condition: P) -> Result<(), SimulationBreak>
        where
            P: Fn(&Self) -> bool,
        {
            loop {
                game_loop::update_game_state(&mut self.state, SIMULATION_DT);
                self.sim_time_s += SIMULATION_DT;
                self.network_state.send_update(&self.state);
                if self.network_state.has_received_ping {
                    sleep(Duration::from_secs_f32(SIMULATION_DT / self.speed));
                }
                for (condidition, is_met) in &self.percistent_condtions {
                    if condidition.is_met(self) != *is_met {
                        return Err(SimulationBreak::PercistentConditionFail);
                    }
                }
                if break_condition(self) {
                    return Ok(());
                }
                if self.sim_time_s > self.timeout_s {
                    return Err(SimulationBreak::Timeout);
                }
            }
        }
        pub fn add_percistent(&mut self, condition: Condition, is_met: bool) {
            self.percistent_condtions.push((condition, is_met))
        }
        pub fn play_entity(&mut self, player_id: PlayerId, entity: Entity) -> EntityId {
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
            .unwrap()
        }
        pub fn place_building(&mut self, player_id: PlayerId, entity: Entity) -> EntityId {
            let building_location_id = self
                .state
                .semi_static_game_state
                .building_locations()
                .iter()
                .find_map(|(id, building_location)| {
                    building_location.entity_id.is_none().then_some(*id)
                })
                .unwrap();
            world_place_building(
                &mut self.state.semi_static_game_state,
                &mut self.state.dynamic_game_state,
                entity,
                &building_location_id,
                player_id,
            )
            .unwrap()
        }
        pub fn place_building_at(
            &mut self,
            player_id: PlayerId,
            entity: Entity,
            pos: (f32, f32),
        ) -> EntityId {
            let building_location_id = BuildingLocationId::new();
            self.state
                .semi_static_game_state
                .building_locations_mut()
                .insert(
                    building_location_id,
                    BuildingLocation {
                        entity_id: None,
                        pos: Vec2::new(pos.0, pos.1),
                        zoning: Zoning::Normal,
                    },
                );
            world_place_building(
                &mut self.state.semi_static_game_state,
                &mut self.state.dynamic_game_state,
                entity,
                &building_location_id,
                player_id,
            )
            .unwrap()
        }
        pub fn play_card(&mut self, player_id: PlayerId, card: Card) {
            self.play_card_at(player_id, card, None)
        }

        pub fn play_card_at(
            &mut self,
            player_id: PlayerId,
            card: Card,
            target: Option<PlayTarget>,
        ) {
            let play_fn = card.get_card_data().play_fn;
            let target = match target {
                Some(target) => target,
                None => match play_fn {
                    PlayFn::UnitSpawnPoint(_) => PlayTarget::UnitSpawnpoint(
                        get_unit_spawnpoints(
                            player_id,
                            &self.state.static_game_state,
                            &self.state.dynamic_game_state,
                        )
                        .first()
                        .unwrap()
                        .clone(),
                    ),
                    PlayFn::BuildingLocation(_) => {
                        PlayTarget::BuildingLocation(BuildingLocationTarget {
                            id: *self
                                .state
                                .semi_static_game_state
                                .building_locations()
                                .iter()
                                .find_map(|(id, building_location)| {
                                    building_location.entity_id.is_none().then_some(id)
                                })
                                .unwrap(),
                        })
                    }
                    PlayFn::WorldPos(_) => PlayTarget::WorldPos(WorldPosTarget { x: 0.0, y: 0.0 }),
                    PlayFn::Entity(_) => todo!(),
                },
            };
            let play_succeded = play_fn.exec(PlayArgs::<PlayTarget> {
                target: &target,
                owner: player_id,
                static_game_state: &self.state.static_game_state,
                semi_static_game_state: &mut self.state.semi_static_game_state,
                dynamic_game_state: &mut self.state.dynamic_game_state,
            });
            assert!(play_succeded);
        }
        pub fn get_entity(&self, entity_id: EntityId) -> &EntityInstance {
            find_entity(&self.state.dynamic_game_state.entities, Some(entity_id)).unwrap()
        }
        pub fn get_entity_position(&self, entity_id: EntityId) -> Vec2 {
            self.get_entity(entity_id).pos
        }
    }
}
