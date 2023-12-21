use macroquad::{
    color::{Color, BLUE, ORANGE},
    math::Vec2,
};

use crate::world::Direction;

pub const LEVEL_WIDTH: i32 = 1920;
pub const LEVEL_HEIGHT: i32 = 1080;

pub const BUILDING_LOCATIONS: &[(i32, i32)] = &[
    (524, 256),
    (1395, 243),
    (1680, 375),
    (896, 728),
    (1604, 708),
    (105, 939),
];

pub const PATHS: &[&[(i32, i32)]] = &[
    &[
        (262, 148),
        (524, 148),
        (1154, 166),
        (1282, 166),
        (1658, 137),
        (1758, 211),
        (1779, 399),
        (1755, 534),
        (1752, 813),
    ],
    &[
        (262, 148),
        (524, 148),
        (1154, 166),
        (1272, 228),
        (1424, 424),
        (1689, 477),
        (1755, 534),
        (1752, 813),
    ],
    &[
        (231, 219),
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
    &[(84, 250), (86, 722), (156, 849), (294, 924), (1672, 976)],
];

pub const SPAWN_POINT_RADIUS: f32 = 192.0;

pub const PLAYER_CONFIGS: &[(Vec2, Direction, Color)] = &[
    (Vec2 { x: 144.0, y: 148.0 }, Direction::Positive, ORANGE),
    (
        Vec2 {
            x: 1772.0,
            y: 968.0,
        },
        Direction::Negative,
        BLUE,
    ),
];
