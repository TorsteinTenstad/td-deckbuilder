pub mod test {
    use common::{
        card::Card,
        entity::EntityTag,
        entity_blueprint::EntityBlueprint,
        game_loop,
        game_state::ServerControledGameState,
        get_unit_spawnpoints::get_unit_spawnpoints,
        ids::{PathId, PlayerId},
        play_target::PlayFn,
        server_player::ServerPlayer,
        world::Direction,
    };
    use macroquad::{
        color::{BLUE, RED},
        math::Vec2,
    };

    pub struct TestEnvironmentPlayer {
        player_id: PlayerId,
        played_cards: Vec<Card>,
    }

    impl TestEnvironmentPlayer {
        pub fn plays(&mut self, card: Card) {
            self.played_cards.push(card);
        }
    }

    pub struct TestEnvironment {
        pub state: ServerControledGameState,
        pub player_a: TestEnvironmentPlayer,
        pub player_b: TestEnvironmentPlayer,
        pub sim_time_s: f32,
        pub timeout_s: f32,
    }

    impl Default for TestEnvironment {
        fn default() -> TestEnvironment {
            let mut state = ServerControledGameState::default();

            for (player_id, base_pos, direction, color) in &[
                (0u64, Vec2::new(0.0, 0.0), Direction::Positive, RED),
                (1u64, Vec2::new(1000.0, 0.0), Direction::Negative, BLUE),
            ] {
                state.dynamic_game_state.players.insert(
                    PlayerId(*player_id),
                    ServerPlayer::new(direction.clone(), *color, Vec::new()),
                );
                let mut base_entity = EntityBlueprint::Base.create(PlayerId(*player_id));
                base_entity.pos = *base_pos;
                state.dynamic_game_state.entities.push(base_entity);
            }
            state
                .static_game_state
                .paths
                .insert(PathId::new(), vec![(0.0, 0.0), (1000.0, 0.0)]);

            TestEnvironment {
                state,
                player_a: TestEnvironmentPlayer {
                    player_id: PlayerId(0),
                    played_cards: Vec::new(),
                },
                player_b: TestEnvironmentPlayer {
                    player_id: PlayerId(1),
                    played_cards: Vec::new(),
                },
                sim_time_s: 0.0,
                timeout_s: 600.0,
            }
        }
    }

    pub enum Condition {
        NoUnitsAlive,
    }

    impl Condition {
        fn is_met(&self, env: &TestEnvironment) -> bool {
            match self {
                Condition::NoUnitsAlive => env
                    .state
                    .dynamic_game_state
                    .entities
                    .iter()
                    .filter(|entity| matches!(entity.tag, EntityTag::Unit | EntityTag::FlyingUnit))
                    .count()
                    .eq(&0),
            }
        }
    }

    pub struct Timeout {}

    impl TestEnvironment {
        pub fn simulate_until(&mut self, condition: Condition) -> Result<(), Timeout> {
            self.exec_played_cards();
            while !condition.is_met(self) {
                self.simulate_step();
                if self.sim_time_s > self.timeout_s {
                    return Err(Timeout {});
                }
            }
            Ok(())
        }
        fn simulate_step(&mut self) {
            let dt = 0.016;
            self.sim_time_s += dt;
            game_loop::update_game_state(&mut self.state, dt);
        }
        fn exec_played_cards(&mut self) {
            for player in [&mut self.player_a, &mut self.player_b].iter_mut() {
                for card in player.played_cards.drain(..) {
                    match &card.get_card_data().play_fn {
                        PlayFn::UnitSpawnPoint(specific_play_fn) => {
                            let spawnpoints = get_unit_spawnpoints(
                                player.player_id,
                                &self.state.static_game_state,
                                &self.state.dynamic_game_state,
                            );
                            let target = spawnpoints.first().unwrap();
                            let in_valid = specific_play_fn.target_is_invalid.is_some_and(|f| {
                                f(
                                    target,
                                    player.player_id,
                                    &self.state.static_game_state,
                                    &self.state.semi_static_game_state,
                                    &self.state.dynamic_game_state,
                                )
                            });
                            assert!(!in_valid);
                            (specific_play_fn.play)(
                                target.clone(),
                                player.player_id,
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
    }
}
