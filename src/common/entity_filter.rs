use crate::{
    component_attack::TargetPool,
    entity::{EntityInstance, EntityTag},
    enum_flags::EnumFlags,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityFilter {
    pub range: Option<f32>,
    pub target_pool: TargetPool,
    pub tag_filter: EnumFlags<EntityTag>,
}

impl EntityFilter {
    pub fn to_fn(
        &self,
        entity_instance: &EntityInstance,
    ) -> Box<dyn FnMut(&&EntityInstance) -> bool> {
        let filter = self.clone();
        let pos = entity_instance.pos;
        let owner = entity_instance.owner;

        Box::new(move |other| {
            let distance_check = !filter
                .range
                .is_some_and(|r| (pos - other.pos).length_squared() > r.powi(2));
            let owner_check = filter.target_pool.in_pool(owner, other.owner);
            let tag_check = filter.tag_filter.is_set(&other.entity.tag);
            distance_check && owner_check && tag_check
        })
    }
}
