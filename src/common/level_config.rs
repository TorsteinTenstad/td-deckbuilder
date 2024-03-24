use macroquad::{
    color::{Color, BLUE, ORANGE},
    math::Vec2,
};

use crate::world::{Direction, Zoning};

pub struct LevelConfig {
    pub level_width: i32,
    pub level_height: i32,
    pub spawn_point_radius: f32,
    pub player_configs: Vec<(Vec2, Direction, Color)>,
    pub building_locations: Vec<(Zoning, (f32, f32))>,
    pub paths: Vec<Vec<(f32, f32)>>,
}

pub fn get_prototype_level_config() -> LevelConfig {
    LevelConfig {
        level_width: 2048,
        level_height: 1152,
        spawn_point_radius: 256.0,
        player_configs: vec![
            (Vec2 { x: 152.0, y: 236.0 }, Direction::Positive, ORANGE),
            (
                Vec2 {
                    x: 1817.0,
                    y: 1033.0,
                },
                Direction::Negative,
                BLUE,
            ),
        ],
        building_locations: vec![
            (Zoning::Commerce, (213.0, 376.0)),
            (Zoning::Commerce, (1727.0, 769.0)),
            (Zoning::Commerce, (760.0, 700.0)),
            (Zoning::Commerce, (1050.0, 480.0)),
            (Zoning::Normal, (331.0, 214.0)),
            (Zoning::Normal, (512.0, 243.0)),
            (Zoning::Normal, (803.0, 119.0)),
            (Zoning::Normal, (996.0, 64.0)),
            (Zoning::Normal, (1545.0, 151.0)),
            (Zoning::Normal, (1940.0, 484.0)),
            (Zoning::Normal, (1946.0, 774.0)),
            (Zoning::Normal, (473.0, 492.0)),
            (Zoning::Normal, (1102.0, 267.0)),
            (Zoning::Normal, (1687.0, 375.0)),
            (Zoning::Normal, (1558.0, 832.0)),
            (Zoning::Normal, (1336.0, 848.0)),
            (Zoning::Normal, (1062.0, 666.0)),
            (Zoning::Normal, (272.0, 735.0)),
            (Zoning::Normal, (193.0, 966.0)),
            (Zoning::Normal, (1505.0, 1051.0)),
        ],
        paths: vec![
            vec![
                (243.0, 276.0),
                (420.0, 341.0),
                (582.0, 331.0),
                (679.0, 256.0),
                (750.0, 257.0),
                (1015.0, 173.0),
                (1457.0, 246.0),
                (1700.0, 254.0),
                (1817.0, 381.0),
                (1854.0, 838.0),
            ],
            vec![
                (243.0, 276.0),
                (420.0, 341.0),
                (582.0, 331.0),
                (679.0, 256.0),
                (750.0, 257.0),
                (969.0, 272.0),
                (1086.0, 421.0),
                (1226.0, 492.0),
                (1507.0, 537.0),
                (1484.0, 673.0),
                (1441.0, 782.0),
                (1458.0, 861.0),
                (1487.0, 949.0),
                (1661.0, 933.0),
            ],
            vec![
                (109.0, 352.0),
                (122.0, 687.0),
                (191.0, 830.0),
                (326.0, 854.0),
                (537.0, 640.0),
                (689.0, 600.0),
                (885.0, 650.0),
                (1094.0, 856.0),
                (1173.0, 866.0),
                (1255.0, 948.0),
                (1487.0, 949.0),
                (1661.0, 933.0),
            ],
        ],
    }
}
