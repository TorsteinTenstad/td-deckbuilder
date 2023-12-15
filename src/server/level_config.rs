use common::Direction;
use macroquad::{
    color::{Color, PURPLE, YELLOW},
    math::Vec2,
};

pub const PATHS: &[&[(f32, f32)]] = &[
    &[(1.0, 1.0), (2.0, 3.0), (3.0, 3.0), (4.0, 4.0)],
    &[(1.0, 2.0), (2.0, 5.0), (3.0, 5.0), (4.0, 6.0)],
];

pub const PLAYER_CONFIGS: &[(Vec2, Direction, Color)] = &[
    (Vec2 { x: 0.5, y: 0.5 }, Direction::Positive, YELLOW),
    (Vec2 { x: 5.5, y: 5.5 }, Direction::Negative, PURPLE),
];
