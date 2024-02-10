use crate::{
    entity_blueprint::EntityBlueprint,
    ids::CardInstanceId,
    play_target::PlayFn,
    textures::SpriteId,
    world::{find_entity_mut, world_place_tower, world_place_unit},
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, EnumIter, Serialize, Deserialize, Eq, PartialEq)]
pub enum Card {
    Tower,
    SpawnPoint,
    HomesickWarrior,
    ElfWarrior,
    OldSwordMaster,
    DemonWolf,
    SmallCriminal,
    StreetCriminal,
    Spy,
    RecklessKnight,
    DirectDamage,
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
    pub sprite_id: SpriteId,
    pub play_fn: PlayFn,
    pub description: &'static str,
    pub attack: Option<i32>,
    pub health: Option<i32>,
}

const DEFAULT_CARD_DATA: CardData = CardData {
    name: "",
    energy_cost: 0,
    sprite_id: SpriteId::Empty,
    play_fn: PlayFn::WorldPos(|_, _, _, _| false),
    description: "",
    attack: None,
    health: None,
};

macro_rules! play_building {
    ($builder_blueprint:ident, $building_blueprint:ident) => {
        PlayFn::BuildingSpot(|target, owner, static_game_state, dynamic_game_state| {
            world_place_tower(
                static_game_state,
                dynamic_game_state,
                target,
                owner,
                EntityBlueprint::$builder_blueprint,
                EntityBlueprint::$building_blueprint,
            )
        })
    };
}

macro_rules! play_unit {
    ($unit_blueprint:ident) => {
        PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            world_place_unit(
                static_game_state,
                dynamic_game_state,
                target,
                owner,
                EntityBlueprint::$unit_blueprint,
            )
        })
    };
}

const CARD_DATA: &[CardData] = &[
    CardData {
        name: "Tower",
        energy_cost: 3,
        sprite_id: SpriteId::CardTower,
        play_fn: play_building!(BasicBuilder, Tower),
        attack: Some(3),
        health: Some(500),
        ..DEFAULT_CARD_DATA
    },
    CardData {
        name: "Spawn Point",
        energy_cost: 3,
        sprite_id: SpriteId::CardSpawnPoint,
        play_fn: play_building!(BasicBuilder, SpawnPoint),
        attack: None,
        health: Some(400),
        ..DEFAULT_CARD_DATA
    },
    CardData {
        name: "Homesick Warrior",
        energy_cost: 3,
        sprite_id: SpriteId::CardHomesickWarrior,
        play_fn: play_unit!(HomesickWarrior),
        attack: Some(20),
        health: Some(200),
        ..DEFAULT_CARD_DATA
    },
    CardData {
        name: "Elf Warrior",
        energy_cost: 2,
        sprite_id: SpriteId::CardElfWarrior,
        play_fn: play_unit!(ElfWarrior),
        attack: Some(10),
        health: Some(100),
        ..DEFAULT_CARD_DATA
    },
    CardData {
        name: "Old Sword Master",
        energy_cost: 4,
        sprite_id: SpriteId::CardOldSwordMaster,
        play_fn: play_unit!(OldSwordMaster),
        attack: Some(50),
        health: Some(200),
        ..DEFAULT_CARD_DATA
    },
    CardData {
        name: "Demon Wolf",
        energy_cost: 3,
        sprite_id: SpriteId::CardDemonWolf,
        play_fn: play_unit!(DemonWolf),
        attack: Some(20),
        health: Some(200),
        ..DEFAULT_CARD_DATA
    },
    CardData {
        name: "Small Criminal",
        energy_cost: 1,
        sprite_id: SpriteId::CardSmallCriminal,
        play_fn: play_unit!(SmallCriminal),
        attack: Some(10),
        health: Some(200),
        ..DEFAULT_CARD_DATA
    },
    CardData {
        name: "Street Criminal",
        energy_cost: 2,
        sprite_id: SpriteId::CardStreetCriminal,
        play_fn: play_unit!(StreetCriminal),
        attack: Some(10),
        health: Some(200),
        ..DEFAULT_CARD_DATA
    },
    CardData {
        name: "Spy",
        energy_cost: 3,
        sprite_id: SpriteId::CardSpy,
        play_fn: play_unit!(Spy),
        attack: Some(20),
        health: Some(200),
        ..DEFAULT_CARD_DATA
    },
    CardData {
        name: "Reckless Knight",
        energy_cost: 2,
        sprite_id: SpriteId::CardRecklessKnight,
        play_fn: play_unit!(RecklessKnight),
        attack: Some(30),
        health: Some(100),
        ..DEFAULT_CARD_DATA
    },
    CardData {
        name: "Direct Damage",
        energy_cost: 1,
        sprite_id: SpriteId::CardDirectDamage,
        play_fn: PlayFn::Entity(|target, _owner, _static_game_state, dynamic_game_state| {
            let Some(target_entity) =
                find_entity_mut(&mut dynamic_game_state.entities, Some(target.id))
            else {
                return false;
            };
            target_entity.health.deal_damage(150.0);
            return true;
        }),
        ..DEFAULT_CARD_DATA
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
