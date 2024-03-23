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
    pub building_locations: Vec<(Zoning, (i32, i32))>,
    pub paths: Vec<Vec<(i32, i32)>>,
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
            (Zoning::Commerce, (213, 376)),
            (Zoning::Commerce, (1727, 769)),
            (Zoning::Commerce, (760, 700)),
            (Zoning::Commerce, (1050, 480)),
            (Zoning::Normal, (331, 214)),
            (Zoning::Normal, (512, 243)),
            (Zoning::Normal, (803, 119)),
            (Zoning::Normal, (996, 64)),
            (Zoning::Normal, (1545, 151)),
            (Zoning::Normal, (1940, 484)),
            (Zoning::Normal, (1946, 774)),
            (Zoning::Normal, (473, 492)),
            (Zoning::Normal, (1102, 267)),
            (Zoning::Normal, (1687, 375)),
            (Zoning::Normal, (1558, 832)),
            (Zoning::Normal, (1336, 848)),
            (Zoning::Normal, (1062, 666)),
            (Zoning::Normal, (272, 735)),
            (Zoning::Normal, (193, 966)),
            (Zoning::Normal, (1505, 1051)),
        ],
        paths: vec![
            vec![
                (243, 276),
                (420, 341),
                (582, 331),
                (679, 256),
                (750, 257),
                (1015, 173),
                (1457, 246),
                (1700, 254),
                (1817, 381),
                (1854, 838),
            ],
            vec![
                (243, 276),
                (420, 341),
                (582, 331),
                (679, 256),
                (750, 257),
                (969, 272),
                (1086, 421),
                (1226, 492),
                (1507, 537),
                (1484, 673),
                (1441, 782),
                (1458, 861),
                (1487, 949),
                (1661, 933),
            ],
            vec![
                (109, 352),
                (122, 687),
                (191, 830),
                (326, 854),
                (537, 640),
                (689, 600),
                (885, 650),
                (1094, 856),
                (1173, 866),
                (1255, 948),
                (1487, 949),
                (1661, 933),
            ],
        ],
    }
}
