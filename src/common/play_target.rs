use crate::get_unit_spawnpoints::UnitSpawnpoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayTarget {
    WorldPos(f32, f32),
    UnitSpawnPoint(UnitSpawnpoint),
    BuildingSpot(u64),
    //Entity(u64),  //It would be nice to allow targeting specific entities, but how do we handle card played on entities that are removed within the time it takes for the server to receive the message?
}
impl PlayTarget {
    pub fn world_pos(&self) -> (f32, f32) {
        if let PlayTarget::WorldPos(x, y) = self {
            (*x, *y)
        } else {
            panic!("PlayTarget::world_pos() called on {:?}", self);
        }
    }
    pub fn unit_spawnpoint(&self) -> UnitSpawnpoint {
        if let PlayTarget::UnitSpawnPoint(unit_spawnpoint) = self {
            unit_spawnpoint.clone()
        } else {
            panic!("PlayTarget::unit_spawnpoint() called on {:?}", self);
        }
    }
    fn building_spot(&self) -> u64 {
        if let PlayTarget::BuildingSpot(building_spot) = self {
            *building_spot
        } else {
            panic!("PlayTarget::building_spot() called on {:?}", self);
        }
    }
}
