#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::{card::Card, entity_blueprint::EntityBlueprint};

    #[test]
    fn test_governor() {
        let mut test_env = TestEnvironment::default();

        for pos in [(50.0, 100.0), (100.0, 100.0), (150.0, 100.0)] {
            test_env.place_building_at(test_env.player_a, EntityBlueprint::Wall.create(), pos);
        }

        test_env.play_card(test_env.player_a, Card::Governor);
        test_env.play_card(test_env.player_b, Card::Governor);

        let simulation_result = test_env.simulate_until(Condition::PlayerWon(test_env.player_a));
        assert!(simulation_result.is_ok());
    }
}
