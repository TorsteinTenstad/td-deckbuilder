use common::{
    entity::{EntityInstance, EntityTag},
    ids::{EntityId, PlayerId},
};
use itertools::Itertools;

use crate::test_environment::test::TestEnvironment;

pub enum Condition {
    NoUnitsAlive,
    SingleUnitAlive(EntityId),
    PlayerWins(PlayerId),
}

fn filtered_count_equals<P>(count: usize, test_environment: &TestEnvironment, predicate: P) -> bool
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
        .eq(&count)
}

fn is_unit(entity_instance: &&EntityInstance) -> bool {
    matches!(
        entity_instance.entity.tag,
        EntityTag::Unit | EntityTag::FlyingUnit
    )
}

impl Condition {
    pub fn is_met(&self, test_environment: &TestEnvironment) -> bool {
        match self {
            Condition::NoUnitsAlive => filtered_count_equals(0, test_environment, is_unit),
            Condition::SingleUnitAlive(entity_id) => {
                test_environment
                    .state
                    .dynamic_game_state
                    .entities
                    .iter()
                    .filter_map(|x| is_unit(&x).then_some(x.id))
                    .collect_vec()
                    == vec![*entity_id]
            }
            Condition::PlayerWins(player_id) => {
                test_environment
                    .state
                    .dynamic_game_state
                    .entities
                    .iter()
                    .filter_map(|entity_instance| {
                        matches!(entity_instance.entity.tag, EntityTag::Base)
                            .then_some(entity_instance.owner)
                    })
                    .collect_vec()
                    == vec![*player_id]
            }
        }
    }
}
