#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::{entity::EntityState, entity_blueprint::EntityBlueprint};

    fn test_impl(building: Option<EntityBlueprint>) {
        let mut test_env = TestEnvironment::default();

        if let Some(building) = building {
            test_env.place_building(test_env.player_a, building.create());
        }

        let ranger_id =
            test_env.play_entity(test_env.player_a, EntityBlueprint::ElfWarrior.create());

        let simulation_result = test_env.simulate_until(Condition::EntityIsInState(
            ranger_id,
            EntityState::Attacking,
        ));
        assert!(simulation_result.is_ok());

        let simulation_result =
            test_env.simulate_until(Condition::EntityIsInState(ranger_id, EntityState::Moving));
        assert!(simulation_result.is_ok());

        assert!(Condition::PlayerWon(test_env.player_a).is_met(&test_env))
    }

    #[test]
    fn test_ranger_stops_to_attack() {
        test_impl(None);
    }

    #[test]
    fn test_ranger_stops_to_attack_when_buffed() {
        test_impl(Some(EntityBlueprint::Watchtower));
    }
}
