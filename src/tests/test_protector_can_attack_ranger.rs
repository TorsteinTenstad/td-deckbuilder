#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::entity_blueprint::EntityBlueprint;

    #[test]
    fn test_protector_can_attack_ranger() {
        let mut test_env = TestEnvironment::default();

        let mut protector = EntityBlueprint::HomesickWarrior.create();
        protector.health.health = f32::MAX;
        let protector_id = test_env.play_entity(test_env.player_a, protector);

        let ranger = EntityBlueprint::ElfWarrior.create();
        test_env.play_entity(test_env.player_b, ranger);

        let simulation_result = test_env.simulate_until(Condition::SingleUnitAlive(protector_id));
        assert!(simulation_result.is_ok())
    }
}
