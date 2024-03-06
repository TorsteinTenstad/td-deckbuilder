use crate::{
    entity::EntityInstance,
    ids::{BuildingLocationId, GameId, PathId, PlayerId},
    server_player::ServerPlayer,
    world::BuildingLocation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct StaticGameState {
    pub paths: HashMap<PathId, Vec<(f32, f32)>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SemiStaticGameState {
    building_locations: HashMap<BuildingLocationId, BuildingLocation>,
    pub dirty: bool,
}

impl SemiStaticGameState {
    pub fn building_locations_mut(&mut self) -> &mut HashMap<BuildingLocationId, BuildingLocation> {
        self.dirty = true;
        &mut self.building_locations
    }
    pub fn building_locations(&self) -> &HashMap<BuildingLocationId, BuildingLocation> {
        &self.building_locations
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DynamicGameState {
    pub entities: Vec<EntityInstance>,
    pub players: HashMap<PlayerId, ServerPlayer>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameMetadata {
    pub game_id: GameId,
    pub server_tick: u32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct ServerControledGameState {
    pub game_metadata: GameMetadata,
    pub static_game_state: StaticGameState,
    pub semi_static_game_state: SemiStaticGameState,
    pub dynamic_game_state: DynamicGameState,
}
