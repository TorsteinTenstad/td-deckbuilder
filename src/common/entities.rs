use serde::{Deserialize, Serialize};

use crate::{
    entity::{EntityInstance, EntityState},
    game_loop::{cleanup_entity, update_entity},
    game_state::ServerControlledGameState,
    update_args::UpdateArgs,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Entities(Vec<EntityInstance>);

impl std::ops::Deref for Entities {
    type Target = [EntityInstance];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Entities {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Entities {
    pub fn at(&self, index: usize) -> Option<&EntityInstance> {
        self.0.get(index)
    }

    pub fn at_mut(&mut self, index: usize) -> Option<&mut EntityInstance> {
        self.0.get_mut(index)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, EntityInstance> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, EntityInstance> {
        self.0.iter_mut()
    }

    pub fn push(&mut self, entity: EntityInstance) {
        self.0.push(entity);
    }
}

pub fn update_entities(server_controlled_game_state: &mut ServerControlledGameState, dt: f32) {
    let mut i = server_controlled_game_state
        .dynamic_game_state
        .entities
        .0
        .len()
        - 1;
    while i > 0 {
        let mut entity_instance = server_controlled_game_state
            .dynamic_game_state
            .entities
            .0
            .swap_remove(i);
        update_entity(&mut UpdateArgs {
            static_game_state: &server_controlled_game_state.static_game_state,
            semi_static_game_state: &mut server_controlled_game_state.semi_static_game_state,
            dynamic_game_state: &mut server_controlled_game_state.dynamic_game_state,
            entity_instance: &mut entity_instance,
            dt,
        });
        server_controlled_game_state
            .dynamic_game_state
            .entities
            .push(entity_instance);
        i -= 1;
    }
}

pub fn remove_dead_entities(server_controlled_game_state: &mut ServerControlledGameState) {
    let mut i = 0;
    while i < server_controlled_game_state
        .dynamic_game_state
        .entities
        .len()
    {
        let entity = &server_controlled_game_state
            .dynamic_game_state
            .entities
            .get(i)
            .unwrap();
        if entity.state == EntityState::Dead {
            cleanup_entity(entity.id, server_controlled_game_state);
            server_controlled_game_state
                .dynamic_game_state
                .entities
                .0
                .swap_remove(i);
        } else {
            i += 1;
        }
    }
}
