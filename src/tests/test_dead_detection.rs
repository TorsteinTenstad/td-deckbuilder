#[cfg(test)]

pub mod test {
    use common::{entity::EntityState, entity_blueprint::EntityBlueprint};

    use crate::{condition::Condition, test_environment::test::TestEnvironment};

    #[test]
    fn test_basic_movement_and_attack() {
        let mut test_env = TestEnvironment::default();
        let entity_a =
            test_env.play_entity(test_env.player_a, EntityBlueprint::SmallCriminal.create());
        let entity_b =
            test_env.play_entity(test_env.player_b, EntityBlueprint::SmallCriminal.create());
        let a_is_dead = Condition::EntityIsInState(entity_a, EntityState::Dead);
        let b_is_dead = Condition::EntityIsInState(entity_b, EntityState::Dead);
        let simulation_result = test_env.simulate_until(a_is_dead);
        assert!(simulation_result.is_ok());
        assert!(b_is_dead.is_met(&test_env));
    }
}
