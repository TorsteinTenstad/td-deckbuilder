#[cfg(test)]

pub mod test {
    use common::{entity::EntityTag, entity_blueprint::EntityBlueprint};

    use crate::{condition::Condition, test_environment::test::TestEnvironment};

    #[test]
    fn test_small_tower() {
        let mut config = TestEnvironment::default_level_config();
        config.building_locations.clear();
        let mut test_env = TestEnvironment::new(config);

        let mut small_tower = EntityBlueprint::SmallTower.create();

        test_env.place_building_at(test_env.player_a, small_tower.clone(), (550.0, 100.0));
        test_env.place_building_at(test_env.player_a, small_tower.clone(), (600.0, 100.0));
        test_env.place_building_at(test_env.player_a, small_tower.clone(), (650.0, 100.0));

        small_tower
            .attacks
            .first_mut()
            .unwrap()
            .can_target
            .set(&EntityTag::Tower);

        let tower_a =
            test_env.place_building_at(test_env.player_a, small_tower.clone(), (600.0, 150.0));
        let tower_b =
            test_env.place_building_at(test_env.player_b, small_tower.clone(), (600.0, 250.0));

        let simulation_result = test_env.simulate_until(Condition::EntityIsDead(tower_b));
        assert!(simulation_result.is_ok());
        assert!(!Condition::EntityIsDead(tower_a).is_met(&test_env));
    }
}
