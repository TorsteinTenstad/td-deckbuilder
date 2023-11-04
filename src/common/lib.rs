use macroquad::prelude::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const SERVER_ADDR: &str = "192.168.1.120:7878";
pub const TARGET_SERVER_FPS: f32 = 60.0;
pub const UNIT_RADIUS: f32 = 0.25;
pub const PROJECTILE_RADIUS: f32 = 0.04;

#[derive(Serialize, Deserialize)]
#[serde(remote = "Vec2")]
pub struct Vec2Def {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientCommand {
    SpawnUnit,
    SpawnTower(i32, i32),
    JoinGame,
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub dynamic_state: DynamicGameState,
    pub static_state: StaticGameState,
}
#[derive(Serialize, Deserialize)]
pub struct StaticGameState {
    pub path: Vec<(i32, i32)>,
    pub grid_w: u32,
    pub grid_h: u32,
}

impl StaticGameState {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            grid_h: 0,
            grid_w: 0,
        }
    }
    pub fn path_to_world_pos(&self, path_pos: f32) -> Vec2 {
        let (low_x, low_y) = self.path[(path_pos as usize).min(self.path.len() - 1)];
        let (high_x, high_y) = self.path[(path_pos as usize + 1).min(self.path.len() - 1)];
        let high_weight = path_pos.fract();
        let low_weight = 1.0 - high_weight;
        Vec2 {
            x: low_x as f32 * low_weight + high_x as f32 * high_weight,
            y: low_y as f32 * low_weight + high_y as f32 * high_weight,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DynamicGameState {
    pub server_tick: u32,
    pub units: HashMap<u64, Unit>,
    pub towers: HashMap<u64, Tower>,
    pub projectiles: Vec<Projectile>,
}

impl DynamicGameState {
    pub fn new() -> Self {
        Self {
            server_tick: 0,
            units: HashMap::new(),
            towers: HashMap::new(),
            projectiles: Vec::new(),
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            dynamic_state: DynamicGameState::new(),
            static_state: StaticGameState::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Unit {
    pub path_pos: f32,
    pub health: f32,
    pub speed: f32,
    pub damage_animation: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Tower {
    pub pos_x: i32,
    pub pos_y: i32,
    pub range: f32,
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
