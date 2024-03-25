use crate::{
    component_attack::TargetPool,
    entity::{EntityInstance, EntityTag},
    enum_flags::EnumFlags,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityFilter<R> {
    pub range: R,
    pub target_pool: TargetPool,
    pub tag_filter: EnumFlags<EntityTag>,
}

pub enum Range {
    Finite(f32),
    Infinite,
}

pub trait ToRange {
    fn to_range(&self) -> Range;
}

impl<R: ToRange> EntityFilter<R> {
    pub fn to_fn(
        &self,
        entity_instance: &EntityInstance,
    ) -> Box<dyn FnMut(&&EntityInstance) -> bool> {
        let filter_range = self.range.to_range();
        let filter_target_pool = self.target_pool.clone();
        let filter_tag_filter = self.tag_filter.clone();
        let pos = entity_instance.pos;
        let owner = entity_instance.owner;

        Box::new(move |other| {
            let distance_check = match filter_range {
                Range::Finite(value) => (pos - other.pos).length_squared() < value.powi(2),
                Range::Infinite => true,
            };
            let owner_check = filter_target_pool.in_pool(owner, other.owner);
            let tag_check = filter_tag_filter.is_set(&other.entity.tag);
            distance_check && owner_check && tag_check
        })
    }
    pub fn to_fn_mut(
        &self,
        entity_instance: &EntityInstance,
    ) -> Box<dyn FnMut(&&mut EntityInstance) -> bool> {
        let filter_range = self.range.to_range();
        let filter_target_pool = self.target_pool.clone();
        let filter_tag_filter = self.tag_filter.clone();
        let pos = entity_instance.pos;
        let owner = entity_instance.owner;

        Box::new(move |other| {
            let distance_check = match filter_range {
                Range::Finite(value) => (pos - other.pos).length_squared() < value.powi(2),
                Range::Infinite => true,
            };
            let owner_check = filter_target_pool.in_pool(owner, other.owner);
            let tag_check = filter_tag_filter.is_set(&other.entity.tag);
            distance_check && owner_check && tag_check
        })
    }
}
