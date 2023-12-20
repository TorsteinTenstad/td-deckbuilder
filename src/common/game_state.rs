use crate::{entity::Entity, server_player::ServerPlayer, world::BuildingLocation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct ServerGameState {
    pub dynamic_state: DynamicGameState,
    pub static_state: StaticGameState,
}

#[derive(Serialize, Deserialize)]
pub struct StaticGameState {
    pub game_id: u64,
    pub paths: HashMap<u64, Vec<(f32, f32)>>,
}

impl StaticGameState {
    pub fn new() -> Self {
        Self {
            game_id: rand::thread_rng().gen(),
            paths: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DynamicGameState {
    pub server_tick: u32,
    pub entities: Vec<Entity>,
    pub players: HashMap<u64, ServerPlayer>,
    pub building_locations: HashMap<u64, BuildingLocation>,
    pub next_entity_id: u64,
}

impl DynamicGameState {
    pub fn new() -> Self {
        Self {
            server_tick: 0,
            entities: Vec::new(),
            players: HashMap::new(),
            building_locations: HashMap::new(),
            next_entity_id: 0,
        }
    }
}

impl ServerGameState {
    pub fn new() -> Self {
        ServerGameState {
            dynamic_state: DynamicGameState::new(),
            static_state: StaticGameState::new(),
        }
    }
}
