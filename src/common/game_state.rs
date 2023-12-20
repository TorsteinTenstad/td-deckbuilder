use crate::{
    entity::Entity,
    ids::{BuildingLocationId, GameId, PathId, PlayerId},
    server_player::ServerPlayer,
    world::BuildingLocation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct ServerGameState {
    pub dynamic_state: DynamicGameState,
    pub static_state: StaticGameState,
}

#[derive(Serialize, Deserialize)]
pub struct StaticGameState {
    pub game_id: GameId,
    pub paths: HashMap<PathId, Vec<(f32, f32)>>,
}

impl StaticGameState {
    pub fn new() -> Self {
        Self {
            game_id: GameId::new(),
            paths: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DynamicGameState {
    pub server_tick: u32,
    pub entities: Vec<Entity>,
    pub players: HashMap<PlayerId, ServerPlayer>,
    pub building_locations: HashMap<BuildingLocationId, BuildingLocation>,
}

impl DynamicGameState {
    pub fn new() -> Self {
        Self {
            server_tick: 0,
            entities: Vec::new(),
            players: HashMap::new(),
            building_locations: HashMap::new(),
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
