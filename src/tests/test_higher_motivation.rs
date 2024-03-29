#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::{card::Card, entity_blueprint::EntityBlueprint};

    #[test]
    fn test_higher_motivation() {
        let mut test_env = TestEnvironment::default();
        let ranger_a =
            test_env.play_entity(test_env.player_a, EntityBlueprint::ElfWarrior.create());
        let ranger_b =
            test_env.play_entity(test_env.player_b, EntityBlueprint::ElfWarrior.create());
        test_env.play_card(test_env.player_a, Card::HigherMotivation);
        let simulation_result = test_env.simulate_until(Condition::EntityIsDead(ranger_b));
        assert!(simulation_result.is_ok());
        assert!(!Condition::EntityIsDead(ranger_a).is_met(&test_env));
    }
}
