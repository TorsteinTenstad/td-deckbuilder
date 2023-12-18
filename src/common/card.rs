use serde::{Deserialize, Serialize};

use crate::{
    play_target::{BuildingSpotTarget, PlayFn, UnitSpawnpointTarget},
    spawn_entity::spawn_entity,
    BuildingLocation, Entity,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Card {
    BasicTower,
    SpawnPointTest,
    BasicUnit,
    BasicRanger,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CardInstance {
    pub id: u64,
    pub card: Card,
}

pub struct CardData {
    pub name: &'static str,
    pub energy_cost: i32,
    pub play_fn: PlayFn,
}

const CARD_DATA: &[CardData] = &[
    CardData {
        name: "Tower",
        energy_cost: 3,
        play_fn: PlayFn::BuildingSpot(|target, owner, _static_game_state, dynamic_game_state| {
            let BuildingSpotTarget { id } = target;
            let BuildingLocation { position, building } =
                dynamic_game_state.building_locations.get_mut(&id).unwrap();
            if let Some(_) = building {
                return false;
            }
            let entity = Entity::new_tower(owner, position.0, position.1, 3.0, 100.0, 2.0, 0.5);
            let key = spawn_entity(&mut dynamic_game_state.entities, entity);
            *building = Some(key);
            return true;
        }),
    },
    CardData {
        name: "Spawn Point",
        energy_cost: 2,
        play_fn: PlayFn::BuildingSpot(|target, owner, _static_game_state, dynamic_game_state| {
            let BuildingSpotTarget { id } = target;
            let BuildingLocation { position, building } =
                dynamic_game_state.building_locations.get_mut(&id).unwrap();
            if let Some(_) = building {
                return false;
            }
            let mut entity = Entity::new_tower(owner, position.0, position.1, 3.0, 100.0, 2.0, 5.0);
            entity.usable_as_spawn_point = true;
            let key = spawn_entity(&mut dynamic_game_state.entities, entity);
            *building = Some(key);
            return true;
        }),
    },
    CardData {
        name: "Ground Unit",
        energy_cost: 1,
        play_fn: PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            let UnitSpawnpointTarget {
                path_id,
                path_idx,
                direction,
            } = target;
            let entity = Entity::new_unit(
                static_game_state,
                owner,
                path_id,
                path_idx,
                direction,
                25.0,
                100.0,
                10.0,
                0.5,
                0.0,
                0.0,
                0.0,
            );
            spawn_entity(&mut dynamic_game_state.entities, entity);
            return true;
        }),
    },
    CardData {
        name: "Ranger",
        energy_cost: 1,
        play_fn: PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            let UnitSpawnpointTarget {
                path_id,
                path_idx,
                direction,
            } = target;
            let entity = Entity::new_unit(
                static_game_state,
                owner,
                path_id,
                path_idx,
                direction,
                25.0,
                50.0,
                0.0,
                0.0,
                3.0,
                5.0,
                0.5,
            );
            spawn_entity(&mut dynamic_game_state.entities, entity);
            return true;
        }),
    },
];

impl Card {
    pub fn get_card_data(&self) -> &CardData {
        CARD_DATA.get(self.clone() as usize).unwrap()
    }

    pub fn name(&self) -> &'static str {
        self.get_card_data().name
    }
    pub fn energy_cost(&self) -> i32 {
        self.get_card_data().energy_cost
    }
}
