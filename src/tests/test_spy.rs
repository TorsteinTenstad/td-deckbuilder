#[cfg(test)]

pub mod test {
    use common::{
        component_attack::Attack,
        component_health::Health,
        component_movement::{Movement, MovementSpeed},
        component_spy::Spy,
        entity::{Entity, EntityState},
        entity_blueprint::EntityBlueprint,
    };

    use crate::{condition::Condition, test_environment::test::TestEnvironment};

    #[test]
    fn test_spy() {
        let mut test_env = TestEnvironment::default();

        let spy = Entity {
            health: Health::new(1.0),
            movement: Some(Movement::new(MovementSpeed::Fast)),
            spy: Some(Spy::new(2)),
            attacks: vec![Attack {
                damage: 10000.0,
                ..Attack::default()
            }],
            ..Entity::default_unit()
        };

        let spy_id = test_env.play_entity(test_env.player_a, spy);
        test_env.play_entity(test_env.player_b, EntityBlueprint::StreetCriminal.create());
        test_env.play_entity(test_env.player_b, EntityBlueprint::StreetCriminal.create());

        test_env
            .add_percistent_condition(Condition::EntityIsInState(spy_id, EntityState::Dead), false);
        test_env.add_percistent_condition(Condition::PlayerWon(test_env.player_b), false);
        test_env
            .simulate_until(Condition::PlayerWon(test_env.player_a))
            .ok();
    }
}
