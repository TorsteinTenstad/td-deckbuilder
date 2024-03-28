#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::card::Card;

    #[test]
    fn test_black_death() {
        let mut test_env = TestEnvironment::default();
        for _ in 0..10 {
            test_env.play_card(test_env.player_a, Card::SmallCriminal);
            test_env.play_card(test_env.player_a, Card::WarEagle);
        }
        test_env.play_card(test_env.player_a, Card::BlackDeath);
        let simulation_result = test_env.simulate_for(61.0);
        assert!(simulation_result.is_ok());
        assert!(Condition::NoUnitsAlive.is_met(&test_env));
    }
}
