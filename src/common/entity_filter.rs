use crate::{
    component_attack::TargetPool,
    entity::{EntityInstance, EntityTag},
    enum_flags::EnumFlags,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityFilter<Range: Clone> {
    pub range_filter: Option<Range>,
    pub pool_filter: Option<TargetPool>,
    pub tag_filter: Option<EnumFlags<EntityTag>>,
}

pub trait Tof32 {
    fn to_f32(&self) -> f32;
}

impl<Range: Tof32 + Clone> EntityFilter<Range> {
    fn predicate(&self, source: &EntityInstance, other: &EntityInstance) -> bool {
        let out_of_range = self.range_filter.as_ref().is_some_and(|range| {
            (source.pos - other.pos).length_squared() > range.to_f32().powi(2)
        });
        let out_of_pool = self
            .pool_filter
            .as_ref()
            .is_some_and(|pool| !pool.in_pool(source.owner, other.owner));
        let out_of_tag_filter = self
            .tag_filter
            .as_ref()
            .is_some_and(|tag_filter| !tag_filter.is_set(&other.entity.tag));
        !out_of_range && !out_of_pool && !out_of_tag_filter
    }
    pub fn to_fn<'a>(
        &'a self,
        source: &'a EntityInstance,
    ) -> impl FnMut(&&EntityInstance) -> bool + 'a {
        move |other| self.predicate(source, other)
    }
    pub fn to_fn_mut<'a>(
        &'a self,
        source: &'a EntityInstance,
    ) -> impl FnMut(&&mut EntityInstance) -> bool + 'a {
        move |other| self.predicate(source, other)
    }
}
