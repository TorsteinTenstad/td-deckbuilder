#[cfg(test)]

pub mod test {
    use crate::test_environment::test::TestEnvironment;
    use common::entity_blueprint::EntityBlueprint;

    #[test]
    fn test_iron_mine() {
        let mut test_env = TestEnvironment::default();
        assert!(test_env.simulate_frames(2).is_ok());

        let tower = EntityBlueprint::Tower.create();
        let tower_base_health = tower.health.get_health();

        test_env.place_building(test_env.player_a, EntityBlueprint::IronMine.create());
        let tower_id_a =
            test_env.place_building_at(test_env.player_a, tower.clone(), (100.0, 100.0));
        let tower_id_b =
            test_env.place_building_at(test_env.player_b, tower.clone(), (100.0, 200.0));

        assert!(test_env.get_entity(tower_id_a).entity.health.get_health() == tower_base_health);
        assert!(test_env.get_entity(tower_id_b).entity.health.get_health() == tower_base_health);
        assert!(test_env.simulate_frame().is_ok());
        assert!(test_env.get_entity(tower_id_a).entity.health.get_health() > tower_base_health);
        assert!(test_env.get_entity(tower_id_b).entity.health.get_health() == tower_base_health);
    }
}
