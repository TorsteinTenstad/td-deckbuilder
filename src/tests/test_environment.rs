pub mod test {
    use common::{
        card::Card,
        entity::EntityTag,
        entity_blueprint::EntityBlueprint,
        game_loop,
        game_state::ServerControlledGameState,
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

    pub struct TestEnvironment {
        pub state: ServerControlledGameState,
        pub player_a: PlayerId,
        pub player_b: PlayerId,
        pub sim_time_s: f32,
        pub timeout_s: f32,
    }

    impl Default for TestEnvironment {
        fn default() -> Self {
            let mut test_environment = Self {
                state: ServerControlledGameState::default(),
                player_a: PlayerId::new(),
                player_b: PlayerId::new(),
                sim_time_s: 0.0,
                timeout_s: 1000.0,
            };

            let path = vec![(0.0, 0.0), (1000.0, 0.0)];

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
                    .filter(|entity_instance| {
                        matches!(
                            entity_instance.entity.tag,
                            EntityTag::Unit | EntityTag::FlyingUnit
                        )
                    })
                    .count()
                    .eq(&0),
            }
        }
    }

    pub struct Timeout {}

    impl TestEnvironment {
        pub fn simulate_until(&mut self, condition: Condition) -> Result<(), Timeout> {
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
        pub fn play(&mut self, player_id: PlayerId, card: Card) {
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
