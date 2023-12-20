use crate::serde_defs::ColorDef;
use crate::{hand::Hand, world::Direction};
use macroquad::color::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerPlayer {
    pub direction: Direction,
    #[serde(with = "ColorDef")]
    pub color: Color,
    pub hand: Hand,
}

impl ServerPlayer {
    pub fn new(direction: Direction, color: Color) -> Self {
        Self {
            direction,
            color,
            hand: Hand::new(),
        }
    }
}
