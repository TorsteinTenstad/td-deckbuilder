use crate::component_movement_behavior::{
    BulletMovementBehavior, MovementBehavior, PathMovementBehavior,
};
use crate::serde_defs::Vec2Def;
use crate::world::{get_path_pos, next_path_idx, Direction};
use crate::{
    component_attack_melee::MeleeAttack, component_attack_ranged::RangedAttack,
    config::PROJECTILE_RADIUS, game_state::StaticGameState,
};
use macroquad::math::Vec2;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityTag {
    Base,
    Tower,
    Unit,
    Bullet,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum EntityState {
    Moving,
    Attacking,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: u64,
    pub tag: EntityTag,
    pub owner: u64,
    pub state: EntityState,
    pub movement_behavior: MovementBehavior,
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub radius: f32,
    pub health: f32,
    pub damage_animation: f32,
    pub hitbox_radius: f32,
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
        let radius = 24.0;
        Self {
            id: rand::thread_rng().gen(),
            tag: EntityTag::Unit,
            state: EntityState::Moving,
            owner,
            movement_behavior: MovementBehavior::Path {
                0: PathMovementBehavior {
                    path_id,
                    target_path_idx: next_path_idx(path_idx, direction), // Unit is spawned at path_idx, target is next path_idx
                    direction,
                    speed,
                },
            },
            pos: get_path_pos(static_game_state, path_id, path_idx),
            radius,
            health,
            damage_animation: 0.0,
            hitbox_radius: radius,
            usable_as_spawn_point: false,
            ranged_attack: Some(RangedAttack {
                can_target: vec![EntityTag::Unit, EntityTag::Tower],
                range,
                damage: ranged_damage,
                fire_interval,
                cooldown_timer: 0.0,
            }),
            melee_attack: Some(MeleeAttack {
                can_target: vec![EntityTag::Unit, EntityTag::Tower],
                range,
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
        let radius = 24.0;
        Self {
            id: rand::thread_rng().gen(),
            tag: EntityTag::Tower,
            state: EntityState::Attacking,
            owner,
            movement_behavior: MovementBehavior::None,
            pos: Vec2 {
                x: x as i32 as f32, // snap to grid
                y: y as i32 as f32, // snap to grid
            },
            radius,
            health,
            damage_animation: 0.0,
            hitbox_radius: radius,
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
        let radius = PROJECTILE_RADIUS;
        Self {
            id: rand::thread_rng().gen(),
            tag: EntityTag::Bullet,
            state: EntityState::Moving,
            owner,
            movement_behavior: MovementBehavior::Bullet(BulletMovementBehavior {
                velocity: Vec2::new(0.0, 0.0),
                target_entity_id: Some(target_entity_id),
                speed,
            }),
            pos,
            seconds_left_to_live: Some(3.0),
            radius,
            health: 1.0,
            damage_animation: 0.0,
            hitbox_radius: radius,
            usable_as_spawn_point: false,
            ranged_attack: None,
            melee_attack: Some(MeleeAttack {
                can_target: vec![EntityTag::Unit],
                range: radius,
                damage,
                attack_interval: 0.5,
                cooldown_timer: 0.0,
                die_on_hit: true,
            }),
        }
    }
}
