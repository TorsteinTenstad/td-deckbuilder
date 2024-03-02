use serde::{Deserialize, Serialize};

use crate::{
    buff::{buff_add_to_entity, Buff},
    entity::Entity,
    update_args::UpdateArgs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffAura {
    buff: Buff,
    radius: Option<f32>,
}

impl BuffAura {
    pub fn update(update_args: &mut UpdateArgs) {
        let pos = update_args.entity.pos;
        for buff_aura in update_args.entity.buff_auras.iter() {
            let in_range = |entity: &&mut Entity| {
                !buff_aura
                    .radius
                    .is_some_and(|r| (entity.pos - pos).length_squared() > r * r)
            };

            for entity in update_args
                .dynamic_game_state
                .entities
                .iter_mut()
                .filter(in_range)
            {
                buff_add_to_entity(entity, buff_aura.buff.clone())
            }
        }
    }
}
