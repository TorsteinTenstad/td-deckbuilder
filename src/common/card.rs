use crate::{
    entity_blueprint::EntityBlueprint,
    ids::CardInstanceId,
    play_target::PlayFn,
    world::{find_entity_mut, world_place_builder, world_place_path_entity},
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, EnumIter, Serialize, Deserialize, Eq, PartialEq)]
pub enum Card {
    BasicTower,
    SpawnPointTest,
    BasicSwordsman,
    Priest,
    DemonPig,
    BasicRanger,
    DirectDamageTest,
}

impl Card {
    pub fn iter() -> impl Iterator<Item = Card> {
        <Card as IntoEnumIterator>::iter()
    }
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
        play_fn: PlayFn::BuildingSpot(|target, owner, static_game_state, dynamic_game_state| {
            let entity = EntityBlueprint::BasicTowerBuilder.create(owner);
            return world_place_builder(
                dynamic_game_state,
                static_game_state,
                owner,
                entity,
                target,
            );
        }),
    },
    CardData {
        name: "Spawn Point",
        energy_cost: 2,
        play_fn: PlayFn::BuildingSpot(|target, owner, static_game_state, dynamic_game_state| {
            let entity = EntityBlueprint::SpawnPointBuilder.create(owner);
            return world_place_builder(
                dynamic_game_state,
                static_game_state,
                owner,
                entity,
                target,
            );
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
        name: "Priest",
        energy_cost: 1,
        play_fn: PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            let entity = EntityBlueprint::Priest.create(owner);
            world_place_path_entity(static_game_state, dynamic_game_state, entity, target);
            return true;
        }),
    },
    CardData {
        name: "Demon Pig",
        energy_cost: 1,
        play_fn: PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            let entity = EntityBlueprint::DemonPig.create(owner);
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
