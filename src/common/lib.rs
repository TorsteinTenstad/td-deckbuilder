use macroquad::prelude::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const SERVER_ADDR: &str = "192.168.1.120:7878";
pub const TARGET_SERVER_FPS: f32 = 60.0;
pub const UNIT_RADIUS: f32 = 0.1;
pub const PROJECTILE_RADIUS: f32 = 0.04;

#[derive(Serialize, Deserialize)]
#[serde(remote = "Vec2")]
pub struct Vec2Def {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientCommand {
    #[serde(with = "Vec2Def")]
    SpawnUnit(Vec2),
    #[serde(with = "Vec2Def")]
    SetTarget(Vec2),
    JoinGame,
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub server_tick: u32,
    pub units: HashMap<u64, Unit>,
    pub towers: Vec<Tower>,
    pub projectiles: Vec<Projectile>,
    #[serde(with = "Vec2Def")]
    pub target: Vec2,
    pub path: Vec<(i32, i32)>,
    pub grid_w: u32,
    pub grid_h: u32,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            server_tick: 0,
            units: HashMap::new(),
            towers: vec![Vec2::new(1.0, 1.0), Vec2::new(2.0, 2.0)]
                .into_iter()
                .map(|pos| Tower {
                    pos,
                    damage: 50.0,
                    cooldown: 0.5,
                    last_fire: 0.0,
                })
                .collect::<Vec<_>>(),
            projectiles: Vec::new(),
            target: Vec2::new(1.0, 1.0),
            path: Vec::new(),
            grid_h: 0,
            grid_w: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Unit {
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub health: f32,
    pub speed: f32,
    pub damage_animation: f32,
    #[serde(with = "Vec2Def")]
    pub target: Vec2,
}

#[derive(Serialize, Deserialize)]
pub struct Tower {
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub damage: f32,
    pub cooldown: f32,
    pub last_fire: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Projectile {
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub target_id: u64,
    pub speed: f32,
    #[serde(with = "Vec2Def")]
    pub velocity: Vec2,
    pub damage: f32,
    pub seconds_left_to_live: f32,
}
