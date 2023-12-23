use crate::{
    entity_blueprint::EntityBlueprint,
    ids::CardInstanceId,
    play_target::PlayFn,
    world::{find_entity_mut, world_place_building, world_place_path_entity},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Card {
    BasicTower,
    SpawnPointTest,
    BasicSwordsman,
    BasicRanger,
    DirectDamageTest,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CardInstance {
    pub id: CardInstanceId,
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
            let entity = EntityBlueprint::BasicTower.create(owner);
            return world_place_building(dynamic_game_state, entity, target);
        }),
    },
    CardData {
        name: "Spawn Point",
        energy_cost: 2,
        play_fn: PlayFn::BuildingSpot(|target, owner, _static_game_state, dynamic_game_state| {
            let entity = EntityBlueprint::SpawnPointTest.create(owner);
            return world_place_building(dynamic_game_state, entity, target);
        }),
    },
    CardData {
        name: "Swordsman",
        energy_cost: 1,
        play_fn: PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            let entity = EntityBlueprint::BasicSwordsman.create(owner);
            world_place_path_entity(static_game_state, dynamic_game_state, entity, target);
            return true;
        }),
    },
    CardData {
        name: "Ranger",
        energy_cost: 1,
        play_fn: PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            let entity = EntityBlueprint::BasicRanger.create(owner);
            world_place_path_entity(static_game_state, dynamic_game_state, entity, target);
            return true;
        }),
    },
    CardData {
        name: "Direct Damage",
        energy_cost: 1,
        play_fn: PlayFn::Entity(|target, _owner, _static_game_state, dynamic_game_state| {
            let Some(target_entity) =
                find_entity_mut(&mut dynamic_game_state.entities, Some(target.id))
            else {
                return false;
            };
            target_entity.health -= 100.0;
            target_entity.damage_animation = 0.1;
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
