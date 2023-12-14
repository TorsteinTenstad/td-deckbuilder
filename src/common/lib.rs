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
mod get_unit_spawnpoints;
pub mod play_target;
pub use play_target::PlayTarget;
mod spawn_entity;
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
pub struct ServerPlayer {
    pub card_draw_counter: f32,
    pub energy_counter: f32,
    pub direction: Direction,
    #[serde(with = "Vec2Def")]
    pub unit_start_pos: Vec2,
    #[serde(with = "ColorDef")]
    pub color: Color,
}

impl ServerPlayer {
    pub fn new(direction: Direction, unit_start_pos: Vec2, color: Color) -> Self {
        Self {
            card_draw_counter: 5.0,
            energy_counter: 0.0,
            direction,
            unit_start_pos,
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
    pub path: HashMap<u64, Vec<(f32, f32)>>,
}

impl StaticGameState {
    pub fn new() -> Self {
        Self {
            game_id: rand::thread_rng().gen(),
            path: HashMap::new(),
        }
    }
    pub fn path_to_world_pos(&self, path_id: u64, path_pos: f32) -> Vec2 {
        let path = self.path.get(&path_id).unwrap();
        let path_pos = path_pos * (path.len() - 1) as f32;
        let (low_x, low_y) = path.get((path_pos as usize).min(path.len() - 1)).unwrap();
        let (high_x, high_y) = path
            .get((path_pos as usize + 1).min(path.len() - 1))
            .unwrap();
        let high_weight = path_pos.fract();
        let low_weight = 1.0 - high_weight;
        Vec2 {
            x: *low_x as f32 * low_weight + *high_x as f32 * high_weight + 0.5,
            y: *low_y as f32 * low_weight + *high_y as f32 * high_weight + 0.5,
        }
    }
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
    pub fn to_start_path_pos(&self) -> f32 {
        match self {
            Direction::Positive => 0.0,
            Direction::Negative => 1.0,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityTag {
    Tower,
    Unit,
    Drone,
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
        owner: u64,
        path_id: u64,
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
                    path_pos: direction.to_start_path_pos(),
                    direction,
                    speed,
                },
            },
            pos: Vec2::ZERO,
            radius: 0.25,
            health,
            damage_animation: 0.0,
            usable_as_spawn_point: true,
            ranged_attack: Some(RangedAttack {
                can_target: vec![EntityTag::Unit, EntityTag::Drone],
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
                x: x as i32 as f32 + 0.5, // snap to grid
                y: y as i32 as f32 + 0.5, // snap to grid
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
                can_target: vec![EntityTag::Unit, EntityTag::Drone],
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
    Drone(DroneBehavior),
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
    pub path_pos: f32,
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
