use common::{
    entity::{EntityInstance, EntityTag},
    ids::{EntityId, PlayerId},
};

use crate::test_environment::test::TestEnvironment;

pub enum Condition {
    NoUnitsAlive,
    SingleUnitAlive(EntityId),
    PlayerWins(PlayerId),
}

fn single_entity_matches<P>(test_environment: &TestEnvironment, predicate: P) -> bool
where
    P: Fn(&&EntityInstance) -> bool,
{
    test_environment
        .state
        .dynamic_game_state
        .entities
        .iter()
        .filter(predicate)
        .count()
        .eq(&0)
}

impl Condition {
    pub fn is_met(&self, test_environment: &TestEnvironment) -> bool {
        match self {
            Condition::NoUnitsAlive => single_entity_matches(test_environment, |entity_instance| {
                matches!(
                    entity_instance.entity.tag,
                    EntityTag::Unit | EntityTag::FlyingUnit
                )
            }),
            Condition::SingleUnitAlive(entity_id) => {
                single_entity_matches(test_environment, |entity_instance| {
                    matches!(
                        entity_instance.entity.tag,
                        EntityTag::Unit | EntityTag::FlyingUnit
                    ) && entity_instance.id != *entity_id
                })
            }
            Condition::PlayerWins(player_id) => {
                single_entity_matches(test_environment, |entity_instance| {
                    matches!(entity_instance.entity.tag, EntityTag::Base)
                        && entity_instance.owner == *player_id
                })
            }
        }
    }
}
