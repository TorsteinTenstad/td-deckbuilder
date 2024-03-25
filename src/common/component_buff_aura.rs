use crate::{
    buff::{buff_add_to_entity, Buff},
    component_attack::TargetPool,
    entity::EntityTag,
    entity_filter::{EntityFilter, Range, ToRange},
    enum_flags::EnumFlags,
    level_config::get_prototype_level_config,
    update_args::UpdateArgs,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuffAuraRange {
    Default,
    Infinite,
}

impl ToRange for BuffAuraRange {
    fn to_range(&self) -> Range {
        let default = get_prototype_level_config().nearby_radius;
        match self {
            Self::Default => Range::Finite(default),
            Self::Infinite => Range::Infinite,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffAura {
    buff: Buff,
    filter: EntityFilter<BuffAuraRange>,
}

impl BuffAura {
    pub fn new(buff: Buff, range: BuffAuraRange) -> Self {
        Self {
            buff,
            filter: EntityFilter::<BuffAuraRange> {
                range,
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
