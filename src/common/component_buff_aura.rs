use crate::{
    buff::{buff_add_to_entity, Buff},
    component_attack::TargetPool,
    entity::EntityTag,
    entity_filter::EntityFilter,
    enum_flags::EnumFlags,
    update_args::UpdateArgs,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuffAuraRange {
    Short,
    Default,
    Long,
    Infinite,
}

impl BuffAuraRange {
    pub fn to_range(&self) -> Option<f32> {
        let default_range = 200.0;
        match self {
            Self::Short => Some(default_range / 1.5),
            Self::Default => Some(default_range),
            Self::Long => Some(default_range * 1.5),
            Self::Infinite => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffAura {
    buff: Buff,
    filter: EntityFilter,
}

impl BuffAura {
    pub fn new(buff: Buff, range: BuffAuraRange) -> Self {
        Self {
            buff,
            filter: EntityFilter {
                range: range.to_range(),
                target_pool: TargetPool::Allies,
                tag_filter: EnumFlags::<EntityTag>::all(),
            },
        }
    }
    pub fn update(update_args: &mut UpdateArgs) {
        for buff_aura in update_args.entity_instance.entity.buff_auras.iter() {
            for entity_instance in update_args
                .dynamic_game_state
                .entities
                .iter_mut()
                .filter(buff_aura.filter.to_fn_mut(update_args.entity_instance))
            {
                buff_add_to_entity(&mut entity_instance.entity, buff_aura.buff.clone())
            }
        }
    }
}
