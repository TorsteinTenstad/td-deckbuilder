use serde::{Deserialize, Serialize};

use crate::entity::EntityInstance;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Entities {
    entities: Vec<EntityInstance>,
}

impl std::ops::Deref for Entities {
    type Target = [EntityInstance];

    fn deref(&self) -> &Self::Target {
        &self.entities
    }
}

impl std::ops::DerefMut for Entities {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entities
    }
}

impl Entities {
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn at(&self, index: usize) -> Option<&EntityInstance> {
        self.entities.get(index)
    }

    pub fn at_mut(&mut self, index: usize) -> Option<&mut EntityInstance> {
        self.entities.get_mut(index)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, EntityInstance> {
        self.entities.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, EntityInstance> {
        self.entities.iter_mut()
    }

    pub fn swap_remove(&mut self, index: usize) -> EntityInstance {
        self.entities.swap_remove(index)
    }

    pub fn insert(&mut self, index: usize, entity: EntityInstance) {
        self.entities.insert(index, entity);
    }

    pub fn push(&mut self, entity: EntityInstance) {
        self.entities.push(entity);
    }
}
