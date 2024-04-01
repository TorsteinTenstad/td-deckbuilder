use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, EnumIter, Default)]
pub enum SpriteId {
    #[default]
    Empty,
    Map,
    UnitArcher,
    UnitSwordsman,
    BuildingBase,
    BuildingTower,
    BuildingSpawnpoint,
    BuildingHut,
    BuildingTradingPlace,
    BuildingFarm,
    UnitBuilder,
    UnitPriest,
    UnitDemonPig,
    UnitRecklessKnight,
    UnitGovernor,
    UnitDynamiteMan,
    UnitSpy,
    UnitStreetCriminal,
    UnitSmallCriminal,
    UnitDemonWolf,
    UnitOldSwordMaster,
    UnitElfWarrior,
    UnitHomesickWarrior,
    UnitAirBalloon,
    UnitDragon,
    UnitWarEagle,
}

impl SpriteId {
    pub fn to_path(&self) -> &'static str {
        match self {
            SpriteId::Map => "map.png",
            SpriteId::UnitArcher => "unit_archer.png",
            SpriteId::UnitSwordsman => "unit_swordsman.png",
            SpriteId::UnitPriest => "unit_priest.png",
            SpriteId::UnitBuilder => "unit_builder.png",
            SpriteId::UnitDemonPig => "unit_demon_pig.png",
            SpriteId::UnitRecklessKnight => "reckless_knight.png",
            SpriteId::UnitSpy => "spy.png",
            SpriteId::UnitStreetCriminal => "street_criminal.png",
            SpriteId::UnitSmallCriminal => "small_criminal.png",
            SpriteId::UnitDemonWolf => "demon_wolf.png",
            SpriteId::UnitOldSwordMaster => "old_sword_master.png",
            SpriteId::UnitElfWarrior => "elf_warrior.png",
            SpriteId::UnitHomesickWarrior => "homesick_warrior.png",
            SpriteId::UnitGovernor => "governor.png",
            SpriteId::UnitDynamiteMan => "dynamite_man.png",
            SpriteId::UnitAirBalloon => "air_balloon.png",
            SpriteId::UnitDragon => "dragon.png",
            SpriteId::UnitWarEagle => "war_eagle.png",
            SpriteId::Empty => "x.png",
            SpriteId::BuildingBase => "building_base.png",
            SpriteId::BuildingTower => "building_tower.png",
            SpriteId::BuildingSpawnpoint => "building_spawnpoint.png",
            SpriteId::BuildingHut => "building_hut.png",
            SpriteId::BuildingTradingPlace => "building_trading_place.png",
            SpriteId::BuildingFarm => "building_farm.png",
        }
    }
}
