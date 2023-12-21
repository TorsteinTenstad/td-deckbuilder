use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
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
}
