use macroquad::prelude::{Color, Vec2};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    net::SocketAddr,
};
pub mod card;
use card::Card;
pub const SERVER_ADDR: &str = "192.168.1.120:7878";
pub const TARGET_SERVER_FPS: f32 = 60.0;
pub const PROJECTILE_RADIUS: f32 = 0.04;

pub fn hash_client_addr(addr: &SocketAddr) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    addr.to_string().hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(remote = "Vec2")]
pub struct Vec2Def {
    pub x: f32,
    pub y: f32,
}
#[derive(Serialize, Deserialize)]
#[serde(remote = "Color")]
pub struct ColorDef {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub card_draw_counter: f32,
    pub direction: Direction,
    #[serde(with = "ColorDef")]
    pub color: Color,
}

impl Player {
    pub fn new(direction: Direction, color: Color) -> Self {
        Self {
            card_draw_counter: 5.0,
            direction,
            color,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientCommand {
    PlayCard(f32, f32, Card),
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
    pub entities: HashMap<u64, Entity>,
    pub players: HashMap<u64, Player>,
}

impl DynamicGameState {
    pub fn new() -> Self {
        Self {
            server_tick: 0,
            entities: HashMap::new(),
            players: HashMap::new(),
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

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Direction {
    Positive,
    Negative,
}
impl Direction {
    pub fn to_f32(&self) -> f32 {
        match self {
            Direction::Positive => 1.0,
            Direction::Negative => -1.0,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Entity {
    pub owner: u64,
    pub movement: Kinematics,
    pub radius: f32,
    pub health: f32,
    pub damage_animation: f32,
    pub ranged_attack: Option<RangedAttack>,
    pub melee_attack: Option<MeleeAttack>,
    pub seconds_left_to_live: Option<f32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Kinematics {
    Static(StaticKinematics),
    Free(FreeKinematics),
    Path(PathKinematics),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StaticKinematics {
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FreeKinematics {
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    #[serde(with = "Vec2Def")]
    pub velocity: Vec2,
    pub target_entity_id: Option<u64>,
    pub speed: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PathKinematics {
    pub path_pos: f32,
    pub direction: Direction,
    pub speed: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RangedAttack {
    pub range: f32,
    pub damage: f32,
    pub fire_rate: f32,
    pub cooldown_timer: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MeleeAttack {
    pub damage: f32,
    pub fire_rate: f32,
    pub cooldown_timer: f32,
}
