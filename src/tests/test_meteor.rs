#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::{
        card::Card,
        entity_blueprint::EntityBlueprint,
        play_target::{EntityTarget, PlayTarget},
    };

    #[test]
    fn test_meteor() {
        let mut test_env = TestEnvironment::default();
        let tower_id = test_env.place_building(test_env.player_a, EntityBlueprint::Tower.create());
        assert!(!Condition::EntityIsDead(tower_id).is_met(&test_env));
        test_env.play_card_at(
            test_env.player_b,
            Card::Meteor,
            Some(PlayTarget::Entity(EntityTarget { id: tower_id })),
        );
        assert!(Condition::EntityIsDead(tower_id).is_met(&test_env));
    }
}
