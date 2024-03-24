#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::{
        component_attack::Attack,
        component_health::Health,
        component_movement::{Movement, MovementSpeed},
        component_spy::Spy,
        entity::Entity,
        entity_blueprint::EntityBlueprint,
        ids::EntityId,
    };

    fn test_setup(hide_capacity: u32, opposing_units: u32) -> (TestEnvironment, EntityId) {
        let mut test_env = TestEnvironment::default();

        let spy = Entity {
            health: Health::new(1.0),
            movement: Some(Movement::new(MovementSpeed::Fast)),
            spy: Some(Spy::new(hide_capacity)),
            attacks: vec![Attack {
                damage: 10000.0,
                ..Attack::default()
            }],
            ..Entity::default_unit()
        };

        let spy_id = test_env.play_entity(test_env.player_a, spy);

        for _ in 0..opposing_units {
            test_env.play_entity(test_env.player_b, EntityBlueprint::StreetCriminal.create());
        }
        (test_env, spy_id)
    }

    #[test]
    fn test_spy_hiding() {
        let (mut test_env, _spy_id) = test_setup(2, 2);
        let simulation_result = test_env.simulate_until(Condition::PlayerWon(test_env.player_a));
        assert!(simulation_result.is_ok());
    }

    #[test]
    fn test_spy_seen() {
        let (mut test_env, spy_id) = test_setup(2, 3);
        let simulation_result = test_env.simulate_until(Condition::EntityIsDead(spy_id));
        assert!(simulation_result.is_ok());
    }
}
