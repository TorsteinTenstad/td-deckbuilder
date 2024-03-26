use crate::test_environment::test::TestEnvironment;
use common::{
    entity::{Entity, EntityInstance, EntityState, EntityTag},
    ids::{EntityId, PlayerId},
};
use itertools::Itertools;

pub enum Condition {
    NoUnitsAlive,
    SingleUnitAlive(EntityId),
    PlayerWon(PlayerId),
    EntityIsInState(EntityId, EntityState),
    EntitySatisfies(EntityId, fn(&Entity) -> bool),
    EntityIsDead(EntityId),
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

fn entity_satisfies<P>(test_environment: &TestEnvironment, entity_id: &EntityId, f: P) -> bool
where
    P: Fn(&EntityInstance) -> bool,
{
    let entity_instance = test_environment
        .state
        .dynamic_game_state
        .entities
        .iter()
        .find(|entity_instance| entity_instance.id == *entity_id)
        .unwrap();
    f(entity_instance)
}

impl Condition {
    pub fn is_met(&self, test_environment: &TestEnvironment) -> bool {
        match self {
            Condition::EntitySatisfies(entity_id, f) => {
                entity_satisfies(test_environment, entity_id, |entity_instance| {
                    f(&entity_instance.entity)
                })
            }
            Condition::EntityIsDead(entity_id) => {
                entity_satisfies(test_environment, entity_id, |entity_instance| {
                    entity_instance.state == EntityState::Dead
                })
            }
            Condition::EntityIsInState(entity_id, entity_state) => {
                entity_satisfies(test_environment, entity_id, |entity_instance| {
                    entity_instance.state == *entity_state
                })
            }
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
            Condition::PlayerWon(player_id) => {
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
