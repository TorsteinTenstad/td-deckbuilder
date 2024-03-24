#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::{
        component_attack::Attack,
        component_health::Health,
        entity::{Entity, EntityTag},
        entity_blueprint::EntityBlueprint,
        enum_flags::{flags, EnumFlags},
    };

    #[test]
    fn test_watchtower() {
        let mut test_env = TestEnvironment::default();

        test_env.place_building(test_env.player_a, EntityBlueprint::Watchtower.create());

        let ranger = Entity {
            health: Health::new(10000.0),
            attacks: vec![Attack {
                damage: 10000.0,
                can_target: flags![EntityTag::Base, EntityTag::Unit],
                ..Attack::default_ranged()
            }],
            ..Entity::default_unit()
        };

        test_env
            .play_entity(test_env.player_a, ranger.clone())
            .unwrap();

        test_env
            .play_entity(test_env.player_b, ranger.clone())
            .unwrap();

        let simulation_result = test_env.simulate_until(Condition::PlayerWins(test_env.player_a));
        assert!(simulation_result.is_ok())
    }
}
