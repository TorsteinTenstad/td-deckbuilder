#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::{entity::Entity, entity_blueprint::EntityBlueprint};

    fn is_ranged_buffed(entity: &Entity) -> bool {
        entity
            .attacks
            .iter()
            .all(|attack| !attack.range_buffs.is_empty())
    }

    fn not_ranged_buffed(entity: &Entity) -> bool {
        entity
            .attacks
            .iter()
            .all(|attack| attack.range_buffs.is_empty())
    }

    #[test]
    fn test_continuous_buff_application() {
        let mut test_env = TestEnvironment::default();

        test_env.place_building(test_env.player_a, EntityBlueprint::Watchtower.create());

        let ranger_id =
            test_env.play_entity(test_env.player_a, EntityBlueprint::ElfWarrior.create());

        test_env
            .simulate_until(Condition::EntitySatisfies(ranger_id, is_ranged_buffed))
            .ok();

        test_env
            .simulate_until(Condition::EntitySatisfies(ranger_id, not_ranged_buffed))
            .ok();

        test_env.add_percistent_condition(
            Condition::EntitySatisfies(ranger_id, is_ranged_buffed),
            false,
        );

        test_env
            .simulate_until(Condition::PlayerWon(test_env.player_a))
            .ok();
    }
}
