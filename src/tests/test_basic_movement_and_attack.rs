#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::card::Card;

    #[test]
    fn test_basic_movement_and_attack() {
        let mut test_env = TestEnvironment::default();
        test_env.play_card(test_env.player_a, Card::SmallCriminal);
        test_env.play_card(test_env.player_b, Card::SmallCriminal);
        let simulation_result = test_env.simulate_until(Condition::NoUnitsAlive);
        assert!(simulation_result.is_ok());
    }
}
