use crate::Direction;
use macroquad::{
    color::{Color, BLUE, RED},
    math::Vec2,
};

pub const LEVEL_WIDTH: i32 = 1920;
pub const LEVEL_HEIGHT: i32 = 1080;

pub const BUILDING_LOCATIONS: &[(i32, i32)] = &[
    (524, 256),
    (1072, 264),
    (1544, 308),
    (896, 728),
    (1604, 708),
    (96, 996),
];

pub const PATHS: &[&[(i32, i32)]] = &[
    &[
        (262, 148),
        (1154, 166),
        (1282, 166),
        (1658, 137),
        (1758, 211),
        (1778, 804),
    ],
    &[
        (262, 148),
        (1154, 166),
        (1272, 228),
        (1424, 424),
        (1758, 506),
        (1778, 804),
    ],
    &[
        (282, 220),
        (760, 484),
        (828, 478),
        (900, 542),
        (916, 618),
        (1336, 798),
        (1384, 760),
        (1482, 764),
        (1516, 802),
        (1700, 848),
    ],
    &[(84, 250), (86, 722), (294, 924), (1672, 976)],
];

pub const SPAWN_POINT_RADIUS: f32 = 192.0;

pub const PLAYER_CONFIGS: &[(Vec2, Direction, Color)] = &[
    (Vec2 { x: 144.0, y: 148.0 }, Direction::Positive, RED),
    (
        Vec2 {
            x: 1772.0,
            y: 968.0,
        },
        Direction::Negative,
        BLUE,
    ),
];
