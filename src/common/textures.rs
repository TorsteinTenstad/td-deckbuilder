use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum SpriteId {
    Empty,
    Hourglass,
    HourglassBow,
    HourglassSword,
    Range,
    Shield,
    Sword,
    Bow,
    Concept,
    UnitArcher,
    UnitSwordsman,
    BuildingBase,
    BuildingTower,
    BuildingSpawnpoint,
    UnitBuilder,
    UnitPriest,
    UnitDemonPig,
    CardTower,
    CardSpawnPoint,
    CardSwordsman,
    CardRanger,
    CardPriest,
    CardDirectDamage,
    CardDemonPig,
}
