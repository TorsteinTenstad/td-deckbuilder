use crate::{
    buff::{buff_add_to_components, buff_add_to_entity, Buff},
    entity::EntityState,
    entity_filter::{EntityFilter, Tof32},
    update_args::UpdateArgs,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum BuffRange {
    #[default]
    Default,
}

impl Tof32 for BuffRange {
    fn to_f32(&self) -> f32 {
        let default = 256.0;
        match self {
            Self::Default => default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffSource {
    pub buff: Buff,
    pub condition: BuffCondition,
    pub target_filter: BuffTargetFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum BuffCondition {
    #[default]
    AlwaysSingle,
    EntityFilter(EntityFilter<BuffRange>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuffTargetFilter {
    Me,
    EntityFilter(EntityFilter<BuffRange>),
    OnSpawn(EntityFilter<BuffRange>),
}

impl BuffSource {
    pub fn update(update_args: &mut UpdateArgs) {
        for buff_source in update_args.entity_instance.entity.buff_sources.iter() {
            let apply_n = match buff_source.condition {
                BuffCondition::AlwaysSingle => 1,
                BuffCondition::EntityFilter(ref entity_filter) => update_args
                    .dynamic_game_state
                    .entities
                    .iter()
                    .filter(entity_filter.to_fn(update_args.entity_instance))
                    .count(),
            };
            for _ in 0..apply_n {
                match buff_source.target_filter {
                    BuffTargetFilter::Me => buff_add_to_components(
                        buff_source.buff.clone(),
                        &mut update_args.entity_instance.entity.attacks,
                        &mut update_args.entity_instance.entity.movement,
                        &mut update_args.entity_instance.entity.health,
                    ),
                    BuffTargetFilter::EntityFilter(ref entity_filter) => {
                        for entity_instance in update_args
                            .dynamic_game_state
                            .entities
                            .iter_mut()
                            .filter(entity_filter.to_fn_mut(update_args.entity_instance))
                        {
                            buff_add_to_entity(
                                buff_source.buff.clone(),
                                &mut entity_instance.entity,
                            )
                        }
                    }
                    BuffTargetFilter::OnSpawn(ref entity_filter) => {
                        for entity_instance in update_args
                            .dynamic_game_state
                            .entities
                            .iter_mut()
                            .filter(|entity_instance| {
                                entity_instance.state == EntityState::SpawnFrame
                            })
                            .filter(entity_filter.to_fn_mut(update_args.entity_instance))
                        {
                            buff_add_to_entity(
                                buff_source.buff.clone(),
                                &mut entity_instance.entity,
                            )
                        }
                    }
                }
            }
        }
    }
}
