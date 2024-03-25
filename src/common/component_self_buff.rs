use crate::{
    buff::{buff_add_to_entity, Buff},
    entity_filter::EntityFilter,
    update_args::UpdateArgs,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelfBuffCondition {
    EntityFilter(EntityFilter),
}

// I currently have this implementation of is_met

impl SelfBuffCondition {
    pub fn _is_met(&self, update_args: &UpdateArgs) -> usize {
        match self {
            SelfBuffCondition::EntityFilter(entity_filter) => update_args
                .dynamic_game_state
                .entities
                .iter()
                .filter(|entity_instance| {
                    !entity_filter.range.is_some_and(|r| {
                        (update_args.entity_instance.pos - entity_instance.pos).length_squared()
                            > r.powi(2)
                    })
                })
                .filter(|entity_instance| {
                    entity_filter
                        .target_pool
                        .in_pool(update_args.entity_instance.owner, entity_instance.owner)
                })
                .filter(|entity_instance| {
                    entity_filter.tag_filter.is_set(&entity_instance.entity.tag)
                })
                .count(),
        }
    }
}

// but I want to implement to_fn on EntityFilter and use it like this in is_met. Can you help

impl SelfBuffCondition {
    pub fn is_met(&self, update_args: &UpdateArgs) -> usize {
        match self {
            SelfBuffCondition::EntityFilter(entity_filter) => update_args
                .dynamic_game_state
                .entities
                .iter()
                .filter(entity_filter.to_fn(update_args.entity_instance))
                .count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfBuff {
    pub buff: Buff,
    pub condition: SelfBuffCondition,
}

impl SelfBuff {
    pub fn update(update_args: &mut UpdateArgs) {
        let self_buffs = update_args.entity_instance.entity.self_buffs.clone();
        for self_buff in self_buffs.iter() {
            let n = self_buff.condition.is_met(update_args);
            for _ in 0..n {
                buff_add_to_entity(
                    &mut update_args.entity_instance.entity,
                    self_buff.buff.clone(),
                );
            }
        }
    }
}
