use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum SpriteId {
    X,
    Hourglass,
    HourglassBow,
    HourglassSword,
    Range,
    Shield,
    Sword,
    Bow,
    Concept,
    UnitArcher,
    UnitSoldier,
}
