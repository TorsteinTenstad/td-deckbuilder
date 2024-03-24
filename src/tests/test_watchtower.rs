#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::{card::Card, entity_blueprint::EntityBlueprint};

    #[test]
    fn test_watchtower() {
        let mut test_env = TestEnvironment::default();
        test_env.play_card(test_env.player_a, Card::Watchtower);

        let mut ranger = EntityBlueprint::ElfWarrior.create();
        ranger.health.health = 1.0;

        test_env
            .play_entity(test_env.player_a, ranger.clone())
            .unwrap();

        test_env
            .play_entity(test_env.player_b, ranger.clone())
            .unwrap();

        let simulation_result = test_env.simulate_until(Condition::PlayerWins(test_env.player_a));
        assert!(simulation_result.is_ok())
    }
}
