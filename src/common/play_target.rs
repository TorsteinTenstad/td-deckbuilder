use crate::{Direction, DynamicGameState, StaticGameState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldPosTarget {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitSpawnpointTarget {
    pub path_id: u64,
    pub path_idx: usize,
    pub direction: Direction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingSpotTarget {
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTarget {
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayTarget {
    WorldPos(WorldPosTarget),
    UnitSpawnPoint(UnitSpawnpointTarget),
    BuildingSpot(BuildingSpotTarget),
    Entity(EntityTarget),
}

pub enum PlayFn {
    WorldPos(fn(WorldPosTarget, u64, &StaticGameState, &mut DynamicGameState) -> bool),
    UnitSpawnPoint(fn(UnitSpawnpointTarget, u64, &StaticGameState, &mut DynamicGameState) -> bool),
    BuildingSpot(fn(BuildingSpotTarget, u64, &StaticGameState, &mut DynamicGameState) -> bool),
    Entity(fn(EntityTarget, u64, &StaticGameState, &mut DynamicGameState) -> bool),
}

impl PlayFn {
    pub fn exec(
        &self,
        target: PlayTarget,
        owner: u64,
        static_game_state: &StaticGameState,
        dynamic_game_state: &mut DynamicGameState,
    ) -> bool {
        match (self, target) {
            (PlayFn::WorldPos(f), PlayTarget::WorldPos(target)) => {
                f(target, owner, static_game_state, dynamic_game_state)
            }
            (PlayFn::UnitSpawnPoint(f), PlayTarget::UnitSpawnPoint(target)) => {
                f(target, owner, static_game_state, dynamic_game_state)
            }
            (PlayFn::BuildingSpot(f), PlayTarget::BuildingSpot(target)) => {
                f(target, owner, static_game_state, dynamic_game_state)
            }
            (PlayFn::Entity(f), PlayTarget::Entity(target)) => {
                f(target, owner, static_game_state, dynamic_game_state)
            }
            _ => panic!("Invalid target for play fn"),
        }
    }
}
