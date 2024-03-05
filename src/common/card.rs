use crate::{
    entity_blueprint::EntityBlueprint,
    ids::CardInstanceId,
    play_target::{PlayFn, SpecificPlayFn},
    world::{find_entity_mut, world_place_builder, world_place_path_entity, Zoning},
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Hash, EnumIter, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Card {
    Tower,
    Farm,
    TradingPlace,
    SpawnPoint,
    HomesickWarrior,
    ElfWarrior,
    OldSwordMaster,
    DemonWolf,
    SmallCriminal,
    StreetCriminal,
    Spy,
    RecklessKnight,
    WarEagle,
    AirBalloon,
    Dragon,
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
    pub enum_variant: Option<Card>,
    pub name: &'static str,
    pub energy_cost: i32,
    pub play_fn: PlayFn,
    pub description: &'static str,
    pub card_art_path: &'static str,
    pub attack: Option<i32>,
    pub health: Option<i32>,
}

const DEFAULT_CARD_DATA: CardData = CardData {
    enum_variant: None,
    name: "",
    energy_cost: 0,
    play_fn: PlayFn::WorldPos(SpecificPlayFn::new(|_, _, _, _, _| false)),
    card_art_path: "",
    description: "",
    attack: None,
    health: None,
};

macro_rules! play_normal_building {
    ($builder_blueprint:ident, $building_blueprint:ident) => {
        PlayFn::BuildingSpot(
            SpecificPlayFn::new(
                |target, owner, static_game_state, semi_static_game_state, dynamic_game_state| {
                    world_place_builder(
                        static_game_state,
                        semi_static_game_state,
                        dynamic_game_state,
                        target,
                        EntityBlueprint::$builder_blueprint.create(),
                        EntityBlueprint::$building_blueprint,
                        owner,
                    )
                },
            )
            .with_target_is_invalid(
                |target,
                 _owner,
                 _static_game_state,
                 semi_static_game_state,
                 _dynamic_game_state| {
                    semi_static_game_state
                        .building_locations()
                        .get(&target.id)
                        .unwrap()
                        .zoning
                        != Zoning::Normal
                },
            ),
        )
    };
}

macro_rules! play_commerce_builging {
    ($builder_blueprint:ident, $building_blueprint:ident) => {
        PlayFn::BuildingSpot(
            SpecificPlayFn::new(
                |target, owner, static_game_state, semi_static_game_state, dynamic_game_state| {
                    world_place_builder(
                        static_game_state,
                        semi_static_game_state,
                        dynamic_game_state,
                        target,
                        EntityBlueprint::$builder_blueprint.create(),
                        EntityBlueprint::$building_blueprint,
                        owner,
                    )
                },
            )
            .with_target_is_invalid(
                |target,
                 _owner,
                 _static_game_state,
                 semi_static_game_state,
                 _dynamic_game_state| {
                    semi_static_game_state
                        .building_locations()
                        .get(&target.id)
                        .unwrap()
                        .zoning
                        != Zoning::Commerce
                },
            ),
        )
    };
}

macro_rules! play_unit {
    ($unit_blueprint:ident) => {
        PlayFn::UnitSpawnPoint(SpecificPlayFn::new(
            |target, owner, static_game_state, _semi_static_game_state, dynamic_game_state| {
                world_place_path_entity(
                    static_game_state,
                    dynamic_game_state,
                    target,
                    EntityBlueprint::$unit_blueprint.create(),
                    owner,
                )
            },
        ))
    };
}

const CARD_DATA: &[CardData] = &[
    CardData {
        enum_variant: Some(Card::Tower),
        name: "Tower",
        energy_cost: 3,
        play_fn: play_normal_building!(BasicBuilder, Tower),
        card_art_path: "tower.jpg",
        attack: Some(3),
        health: Some(500),
        description: "[Ranged]",
    },
    CardData {
        enum_variant: Some(Card::Farm),
        name: "Farm",
        energy_cost: 4,
        play_fn: play_commerce_builging!(BasicBuilder, Farm),
        card_art_path: "farm.jpg",
        attack: None,
        health: Some(200),
        description: "Increases drawing speed by 40%",
    },
    CardData {
        enum_variant: Some(Card::TradingPlace),
        name: "Trading Place",
        energy_cost: 4,
        play_fn: play_commerce_builging!(BasicBuilder, TradingPlace),
        card_art_path: "trading_place.jpg",
        attack: None,
        health: Some(200),
        description: "Increases energy generation by 40%",
    },
    CardData {
        enum_variant: Some(Card::SpawnPoint),
        name: "Spawn Point",
        energy_cost: 3,
        play_fn: play_normal_building!(BasicBuilder, SpawnPoint),
        card_art_path: "spawn_point.jpg",
        attack: None,
        health: Some(400),
        description: "You may spawn units\nfrom this building",
    },
    CardData {
        enum_variant: Some(Card::HomesickWarrior),
        name: "Homesick Warrior",
        energy_cost: 3,
        play_fn: play_unit!(HomesickWarrior),
        card_art_path: "homesick_warrior.jpg",
        attack: Some(20),
        health: Some(200),
        description: "[Protector]",
    },
    CardData {
        enum_variant: Some(Card::ElfWarrior),
        name: "Elf Warrior",
        energy_cost: 2,
        play_fn: play_unit!(ElfWarrior),
        card_art_path: "elf_warrior.jpg",
        attack: Some(10),
        health: Some(100),
        description: "[Fast attacking], [Ranged]",
    },
    CardData {
        enum_variant: Some(Card::OldSwordMaster),
        name: "Old Sword Master",
        energy_cost: 4,
        play_fn: play_unit!(OldSwordMaster),
        card_art_path: "old_sword_master.jpg",
        attack: Some(50),
        health: Some(200),
        description: "[Very slow moving]",
    },
    CardData {
        enum_variant: Some(Card::DemonWolf),
        name: "Demon Wolf",
        energy_cost: 3,
        play_fn: play_unit!(DemonWolf),
        card_art_path: "demon_wolf.jpg",
        attack: Some(20),
        health: Some(200),
        description: "[Fast moving]",
    },
    CardData {
        enum_variant: Some(Card::SmallCriminal),
        name: "Small Criminal",
        energy_cost: 1,
        play_fn: play_unit!(SmallCriminal),
        card_art_path: "small_criminal.jpg",
        attack: Some(10),
        health: Some(200),
        description: "[Fast moving]",
    },
    CardData {
        enum_variant: Some(Card::StreetCriminal),
        name: "Street Criminal",
        energy_cost: 2,
        play_fn: play_unit!(StreetCriminal),
        card_art_path: "street_criminal.jpg",
        attack: Some(10),
        health: Some(200),
        description: "[Fast attacking]",
    },
    CardData {
        enum_variant: Some(Card::Spy),
        name: "Spy",
        energy_cost: 3,
        play_fn: play_unit!(Spy),
        card_art_path: "spy.jpg",
        attack: Some(20),
        health: Some(200),
        description: "Will not be seen\nby the first\n2 enimes it passes",
    },
    CardData {
        enum_variant: Some(Card::RecklessKnight),
        name: "Reckless Knight",
        energy_cost: 2,
        play_fn: play_unit!(RecklessKnight),
        card_art_path: "reckless_knight.jpg",
        attack: Some(30),
        health: Some(100),
        description: "[Fast moving]",
    },
    CardData {
        enum_variant: Some(Card::WarEagle),
        name: "War Eagle",
        energy_cost: 3,
        play_fn: play_unit!(WarEagle),
        card_art_path: "war_eagle.jpg",
        attack: Some(10),
        health: Some(100),
        description: "[Flying]",
    },
    CardData {
        enum_variant: Some(Card::AirBalloon),
        name: "Air Balloon",
        energy_cost: 5,
        play_fn: play_unit!(AirBalloon),
        card_art_path: "air_balloon.jpg",
        attack: Some(20),
        health: Some(400),
        description: "[Flying]",
    },
    CardData {
        enum_variant: Some(Card::Dragon),
        name: "Dragon",
        energy_cost: 7,
        play_fn: play_unit!(Dragon),
        card_art_path: "dragon.jpg",
        attack: Some(40),
        health: Some(400),
        description: "[Flying]",
    },
    CardData {
        enum_variant: Some(Card::DirectDamage),
        name: "Direct Damage",
        energy_cost: 1,
        play_fn: PlayFn::Entity(SpecificPlayFn::new(
            |target, _owner, _static_game_state, _semi_static_game_state, dynamic_game_state| {
                let Some(target_entity_instance) =
                    find_entity_mut(&mut dynamic_game_state.entities, Some(target.id))
                else {
                    return false;
                };
                target_entity_instance.entity.health.deal_damage(150.0);
                true
            },
        )),
        card_art_path: "direct_damage.jpg",
        description: "Deal 150 damage\nto a unit or building",
        ..DEFAULT_CARD_DATA
    },
];

impl Card {
    pub fn validate_card_data() {
        for card in Card::iter() {
            let card_data = CARD_DATA.get(card.clone() as usize).unwrap();
            assert_eq!(card_data.enum_variant, Some(card));
        }
    }
    pub fn get_card_data(&self) -> &CardData {
        CARD_DATA.get(self.clone() as usize).unwrap()
    }

    pub fn name(&self) -> &'static str {
        self.get_card_data().name
    }
    pub fn energy_cost(&self) -> i32 {
        self.get_card_data().energy_cost
    }
    pub fn get_texture_path(&self) -> String {
        format!("assets/cards/{}.png", self.get_card_data().name)
    }
}
