use crate::{
    entities::Entities,
    ids::{BuildingLocationId, GameId, PathId, PlayerId},
    level_config::LevelConfig,
    network::{ServerMessage, ServerMessageData},
    server_player::ServerPlayer,
    world::BuildingLocation,
};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StaticGameState {
    pub paths: HashMap<PathId, Vec<(f32, f32)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DynamicGameState {
    pub entities: Entities,
    pub players: HashMap<PlayerId, ServerPlayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameMetadata {
    pub game_id: GameId,
    pub server_tick: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ServerControlledGameState {
    pub game_metadata: GameMetadata,
    pub static_game_state: StaticGameState,
    pub semi_static_game_state: SemiStaticGameState,
    pub dynamic_game_state: DynamicGameState,
}

impl ServerControlledGameState {
    pub fn update_with_server_message(&mut self, server_message: ServerMessage) -> bool {
        if self.game_metadata.server_tick > server_message.metadata.server_tick
            && server_message.metadata.game_id == self.game_metadata.game_id
        {
            false
        } else {
            self.game_metadata = server_message.metadata;
            match server_message.data {
                ServerMessageData::StaticGameState(static_state) => {
                    self.static_game_state = static_state;
                }
                ServerMessageData::DynamicGameState(dynamic_state) => {
                    self.dynamic_game_state = dynamic_state;
                }
                ServerMessageData::SemiStaticGameState(semi_static_state) => {
                    self.semi_static_game_state = semi_static_state;
                }
            }
            true
        }
    }

    pub fn load_level_config(&mut self, level_config: LevelConfig) {
        for path in level_config.paths {
            self.static_game_state.paths.insert(PathId::new(), path);
        }

        for (zoning, (x, y)) in level_config.building_locations.iter() {
            self.semi_static_game_state.building_locations_mut().insert(
                BuildingLocationId::new(),
                BuildingLocation {
                    pos: Vec2::new(*x, *y),
                    entity_id: None,
                    zoning: zoning.clone(),
                },
            );
        }
    }
}
