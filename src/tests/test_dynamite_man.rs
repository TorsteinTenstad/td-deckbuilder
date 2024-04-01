#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::entity_blueprint::EntityBlueprint;

    #[test]
    fn test_dynamite_man() {
        let mut test_env = TestEnvironment::default();

        test_env.play_entity(test_env.player_a, EntityBlueprint::DynamiteMan.create());

        test_env.play_entity(test_env.player_b, EntityBlueprint::StreetCriminal.create());
        assert!(test_env.simulate_for(0.5).is_ok());
        test_env.play_entity(test_env.player_b, EntityBlueprint::StreetCriminal.create());
        assert!(test_env.simulate_for(0.5).is_ok());
        test_env.play_entity(test_env.player_b, EntityBlueprint::StreetCriminal.create());

        let simulation_result = test_env.simulate_until(Condition::NoUnitsAlive);
        assert!(simulation_result.is_ok());
    }
}
