use common::{entity::EntityTag, ids::EntityId};

use crate::test_environment::test::TestEnvironment;

pub enum Condition {
    NoUnitsAlive,
    SingleUnitAlive(EntityId),
}

impl Condition {
    pub fn is_met(&self, env: &TestEnvironment) -> bool {
        match self {
            Condition::NoUnitsAlive => env
                .state
                .dynamic_game_state
                .entities
                .iter()
                .filter(|entity_instance| {
                    matches!(
                        entity_instance.entity.tag,
                        EntityTag::Unit | EntityTag::FlyingUnit
                    )
                })
                .count()
                .eq(&0),
            Condition::SingleUnitAlive(entity_id) => env
                .state
                .dynamic_game_state
                .entities
                .iter()
                .filter(|entity_instance| {
                    matches!(
                        entity_instance.entity.tag,
                        EntityTag::Unit | EntityTag::FlyingUnit
                    ) && entity_instance.id != *entity_id
                })
                .count()
                .eq(&0),
        }
    }
}
