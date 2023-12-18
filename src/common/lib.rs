use macroquad::prelude::{Color, Vec2};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    net::SocketAddr,
    vec,
};
pub mod card;
pub mod get_unit_spawnpoints;
pub mod level_config;
pub mod play_target;
pub use play_target::PlayTarget;
mod spawn_entity;
use card::Card;

pub const SERVER_ADDR: &str = "192.168.211.23:7878";
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
pub struct ServerPlayer {
    pub card_draw_counter: f32,
    pub energy_counter: f32,
    pub direction: Direction,
    #[serde(with = "ColorDef")]
    pub color: Color,
}

impl ServerPlayer {
    pub fn new(direction: Direction, color: Color) -> Self {
        Self {
            card_draw_counter: 5.0,
            energy_counter: 10.0,
            direction,
            color,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientCommand {
    PlayCard(Card, PlayTarget),
    JoinGame,
}

#[derive(Serialize, Deserialize)]
pub struct ServerGameState {
    pub dynamic_state: DynamicGameState,
    pub static_state: StaticGameState,
}

#[derive(Serialize, Deserialize)]
pub struct StaticGameState {
    pub game_id: u64,
    pub paths: HashMap<u64, Vec<(f32, f32)>>,
    pub building_locations: HashMap<u64, (f32, f32)>,
}

impl StaticGameState {
    pub fn new() -> Self {
        Self {
            game_id: rand::thread_rng().gen(),
            paths: HashMap::new(),
            building_locations: HashMap::new(),
        }
    }
}

pub fn get_path_pos(static_game_state: &StaticGameState, path_id: u64, path_idx: usize) -> Vec2 {
    static_game_state
        .paths
        .get(&path_id)
        .unwrap()
        .get(path_idx)
        .map(|(x, y)| Vec2 { x: *x, y: *y })
        .unwrap()
}

#[derive(Serialize, Deserialize)]
pub struct DynamicGameState {
    pub server_tick: u32,
    pub entities: HashMap<u64, Entity>,
    pub players: HashMap<u64, ServerPlayer>,
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

impl ServerGameState {
    pub fn new() -> Self {
        ServerGameState {
            dynamic_state: DynamicGameState::new(),
            static_state: StaticGameState::new(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
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
    pub fn to_i32(&self) -> i32 {
        match self {
            Direction::Positive => 1,
            Direction::Negative => -1,
        }
    }
}

pub fn next_path_idx(path_idx: usize, direction: Direction) -> usize {
    let next_path_idx = path_idx as i32 + direction.to_i32();
    if next_path_idx < 0 {
        0
    } else {
        next_path_idx as usize
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityTag {
    Base,
    Tower,
    Unit,
    Bullet,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Entity {
    pub tag: EntityTag,
    pub owner: u64,
    pub behavior: Behavior,
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub radius: f32,
    pub health: f32,
    pub damage_animation: f32,
    pub usable_as_spawn_point: bool,
    pub ranged_attack: Option<RangedAttack>,
    pub melee_attack: Option<MeleeAttack>,
    pub seconds_left_to_live: Option<f32>,
}

impl Entity {
    pub fn new_unit(
        static_game_state: &StaticGameState,
        owner: u64,
        path_id: u64,
        path_idx: usize,
        direction: Direction,
        speed: f32,
        health: f32,
        damage: f32,
        attack_interval: f32,
        range: f32,
        ranged_damage: f32,
        fire_interval: f32,
    ) -> Self {
        Self {
            tag: EntityTag::Unit,
            owner,
            behavior: Behavior::PathUnit {
                0: PathUnitBehavior {
                    path_id,
                    target_path_idx: next_path_idx(path_idx, direction), // Unit is spawned at path_idx, target is next path_idx
                    direction,
                    speed,
                },
            },
            pos: get_path_pos(static_game_state, path_id, path_idx),
            radius: 24.0,
            health,
            damage_animation: 0.0,
            usable_as_spawn_point: false,
            ranged_attack: Some(RangedAttack {
                can_target: vec![EntityTag::Unit],
                range,
                damage: ranged_damage,
                fire_interval,
                cooldown_timer: 0.0,
            }),
            melee_attack: Some(MeleeAttack {
                can_target: vec![EntityTag::Unit],
                range: None,
                damage,
                attack_interval,
                cooldown_timer: 0.0,
                die_on_hit: false,
            }),
            seconds_left_to_live: None,
        }
    }

    pub fn new_tower(
        owner: u64,
        x: f32,
        y: f32,
        range: f32,
        health: f32,
        damage: f32,
        fire_interval: f32,
    ) -> Self {
        Self {
            tag: EntityTag::Tower,
            owner,
            behavior: Behavior::None,
            pos: Vec2 {
                x: x as i32 as f32, // snap to grid
                y: y as i32 as f32, // snap to grid
            },
            radius: 0.25,
            health,
            damage_animation: 0.0,
            usable_as_spawn_point: false,
            ranged_attack: Some(RangedAttack {
                can_target: vec![EntityTag::Unit],
                range,
                damage,
                fire_interval,
                cooldown_timer: 0.0,
            }),
            melee_attack: None,
            seconds_left_to_live: None,
        }
    }
    pub fn new_bullet(
        owner: u64,
        pos: Vec2,
        target_entity_id: u64,
        damage: f32,
        speed: f32,
    ) -> Self {
        Self {
            tag: EntityTag::Bullet,
            owner,
            behavior: Behavior::Bullet(BulletBehavior {
                velocity: Vec2::new(0.0, 0.0),
                target_entity_id: Some(target_entity_id),
                speed,
            }),
            pos,
            seconds_left_to_live: Some(3.0),
            radius: PROJECTILE_RADIUS,
            health: 1.0,
            damage_animation: 0.0,
            usable_as_spawn_point: false,
            ranged_attack: None,
            melee_attack: Some(MeleeAttack {
                can_target: vec![EntityTag::Unit],
                range: None,
                damage,
                attack_interval: 0.5,
                cooldown_timer: 0.0,
                die_on_hit: true,
            }),
        }
    }
}

pub struct EntityExternalEffects {
    pub health: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Behavior {
    Bullet(BulletBehavior),
    PathUnit(PathUnitBehavior),
    None,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StaticKinematics {
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BulletBehavior {
    #[serde(with = "Vec2Def")]
    pub velocity: Vec2,
    pub target_entity_id: Option<u64>,
    pub speed: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PathUnitBehavior {
    pub path_id: u64,
    pub target_path_idx: usize,
    pub direction: Direction,
    pub speed: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DroneBehavior {
    pub can_target: Vec<EntityTag>,
    pub target_entity_id: Option<u64>,
    pub speed: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RangedAttack {
    pub can_target: Vec<EntityTag>,
    pub range: f32,
    pub damage: f32,
    pub fire_interval: f32,
    pub cooldown_timer: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MeleeAttack {
    pub can_target: Vec<EntityTag>,
    pub range: Option<f32>,
    pub damage: f32,
    pub attack_interval: f32,
    pub cooldown_timer: f32,
    pub die_on_hit: bool,
}
