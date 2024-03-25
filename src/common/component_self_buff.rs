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
