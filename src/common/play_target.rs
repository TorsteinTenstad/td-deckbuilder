use crate::{Direction, DynamicGameState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldPosTarget {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitSpawnpointTarget {
    pub path_id: u64,
    pub direction: Direction,
    pub path_pos: f32,
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
    WorldPos(fn(WorldPosTarget, u64, &mut DynamicGameState)),
    UnitSpawnPoint(fn(UnitSpawnpointTarget, u64, &mut DynamicGameState)),
    BuildingSpot(fn(BuildingSpotTarget, u64, &mut DynamicGameState)),
    Entity(fn(EntityTarget, u64, &mut DynamicGameState)),
}

impl PlayFn {
    pub fn exec(&self, target: PlayTarget, owner: u64, dynamic_game_state: &mut DynamicGameState) {
        match (self, target) {
            (PlayFn::WorldPos(f), PlayTarget::WorldPos(target)) => {
                f(target, owner, dynamic_game_state)
            }
            (PlayFn::UnitSpawnPoint(f), PlayTarget::UnitSpawnPoint(target)) => {
                f(target, owner, dynamic_game_state)
            }
            (PlayFn::BuildingSpot(f), PlayTarget::BuildingSpot(target)) => {
                f(target, owner, dynamic_game_state)
            }
            (PlayFn::Entity(f), PlayTarget::Entity(target)) => f(target, owner, dynamic_game_state),
            _ => panic!("Invalid target for play fn"),
        }
    }
}
