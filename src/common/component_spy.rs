use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{entity::EntityTag, ids::EntityId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spy {
    pub hide_capacity: u32,
    pub is_hidden_from: HashSet<EntityId>,
}
impl Spy {
    pub fn new(hide_capacity: u32) -> Self {
        Self {
            hide_capacity,
            is_hidden_from: HashSet::new(),
        }
    }
    pub fn is_hidden(&self) -> bool {
        self.is_hidden_from.len() <= self.hide_capacity as usize
    }
    pub fn can_hide_from(&mut self, entity_id: EntityId, entity_tag: EntityTag) -> bool {
        if entity_tag == EntityTag::Unit {
            self.is_hidden_from.insert(entity_id);
        }
        self.is_hidden()
    }
}
