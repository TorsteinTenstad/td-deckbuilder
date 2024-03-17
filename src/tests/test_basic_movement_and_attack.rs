#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::{card::Card, entity_blueprint::EntityBlueprint};

    #[test]
    fn test_basic_movement_and_attack() {
        let speed = std::env::args()
            .next()
            .and_then(|arg| arg.parse().ok())
            .unwrap_or(1.0);
        let mut test_env = TestEnvironment::new(speed);
        test_env.play_card(test_env.player_a, Card::SmallCriminal);
        test_env.play_card(test_env.player_b, Card::SmallCriminal);
        let simulation_result = test_env.simulate_until(Condition::NoUnitsAlive);
        assert!(simulation_result.is_ok())
    }

    #[test]
    fn test_protector_can_attack_ranger() {
        let mut test_env = TestEnvironment::new(1.0);

        let mut protector = EntityBlueprint::HomesickWarrior.create();
        protector.health.health = f32::MAX;
        let protector_id = test_env.play_entity(test_env.player_a, protector).unwrap();

        let ranger = EntityBlueprint::ElfWarrior.create();
        test_env.play_entity(test_env.player_b, ranger).unwrap();

        let simulation_result = test_env.simulate_until(Condition::SingleUnitAlive(protector_id));
        assert!(simulation_result.is_ok())
    }
}
