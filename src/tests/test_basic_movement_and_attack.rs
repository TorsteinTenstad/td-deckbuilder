#[cfg(test)]

pub mod test {
    use crate::test_environment::test::{Condition, TestEnvironment};
    use common::card::Card;

    #[test]
    fn test_basic_movement_and_attack() {
        let mut test_env = TestEnvironment::default();
        test_env.player_a.plays(Card::SmallCriminal);
        test_env.player_b.plays(Card::SmallCriminal);
        let simulation_result = test_env.simulate_until(Condition::NoUnitsAlive);
        assert!(simulation_result.is_ok())
    }
}