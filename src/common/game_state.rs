use crate::{
    entity::Entity,
    ids::{BuildingLocationId, PathId, PlayerId, SemiStaticGameStateVersionId},
    server_player::ServerPlayer,
    world::BuildingLocation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct StaticGameState {
    pub paths: HashMap<PathId, Vec<(f32, f32)>>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SemiStaticGameState {
    pub version_id: SemiStaticGameStateVersionId,
    pub building_locations: HashMap<BuildingLocationId, BuildingLocation>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DynamicGameState {
    pub entities: Vec<Entity>,
    pub players: HashMap<PlayerId, ServerPlayer>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct GameMetadata {
    pub game_id: u32,
    pub server_tick: u32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct ServerControledGameState {
    pub game_metadata: GameMetadata,
    pub static_game_state: StaticGameState,
    pub semi_static_game_state: SemiStaticGameState,
    pub dynamic_game_state: DynamicGameState,
}
