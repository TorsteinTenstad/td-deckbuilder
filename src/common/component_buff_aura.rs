use crate::{
    buff::{buff_add_to_entity, Buff},
    component_attack::TargetPool,
    entity::EntityInstance,
    update_args::UpdateArgs,
};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuffAuraRange {
    Short,
    Default,
    Long,
    Infinite,
}

impl BuffAuraRange {
    pub fn is_within(&self, pos_a: Vec2, pos_b: Vec2) -> bool {
        let default_range = 200.0;
        let distance = (pos_a - pos_b).length();
        match self {
            Self::Short => distance < default_range / 1.5,
            Self::Default => distance < default_range,
            Self::Long => distance < default_range * 1.5,
            Self::Infinite => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffAura {
    buff: Buff,
    range: BuffAuraRange,
    target_pool: TargetPool,
}

impl BuffAura {
    pub fn new(buff: Buff, range: BuffAuraRange) -> Self {
        Self {
            buff,
            range,
            target_pool: TargetPool::Allies,
        }
    }
    pub fn update(update_args: &mut UpdateArgs) {
        let pos = update_args.entity_instance.pos;
        for buff_aura in update_args.entity_instance.entity.buff_auras.iter() {
            let in_range = |entity_instance: &&mut EntityInstance| {
                buff_aura.range.is_within(entity_instance.pos, pos)
            };

            let in_pool = |entity_instance: &&mut EntityInstance| {
                buff_aura
                    .target_pool
                    .in_pool(update_args.entity_instance.owner, entity_instance.owner)
            };

            for entity_instance in update_args
                .dynamic_game_state
                .entities
                .iter_mut()
                .filter(in_pool)
                .filter(in_range)
            {
                buff_add_to_entity(&mut entity_instance.entity, buff_aura.buff.clone())
            }
        }
    }
}
