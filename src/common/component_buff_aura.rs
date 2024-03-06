use serde::{Deserialize, Serialize};

use crate::{
    buff::{buff_add_to_entity, Buff},
    entity::EntityInstance,
    update_args::UpdateArgs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffAura {
    buff: Buff,
    radius: Option<f32>,
}

impl BuffAura {
    pub fn update(update_args: &mut UpdateArgs) {
        let pos = update_args.entity_instance.pos;
        for buff_aura in update_args.entity_instance.entity.buff_auras.iter() {
            let in_range = |entity: &&mut EntityInstance| {
                !buff_aura
                    .radius
                    .is_some_and(|r| (entity.pos - pos).length_squared() > r * r)
            };

            for entity_instance in update_args
                .dynamic_game_state
                .entities
                .iter_mut()
                .filter(in_range)
            {
                buff_add_to_entity(&mut entity_instance.entity, buff_aura.buff.clone())
            }
        }
    }
}
