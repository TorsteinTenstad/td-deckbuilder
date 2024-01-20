use crate::{
    buff::{apply_arithmetic_buffs, ArithmeticBuff},
    config::CLOSE_ENOUGH_TO_TARGET,
    entity::Entity,
    entity_blueprint::DEFAULT_UNIT_DETECTION_RADIUS,
    find_target::find_target_for_attack,
    game_state::{DynamicGameState, StaticGameState},
    ids::{EntityId, PathId},
    play_target::UnitSpawnpointTarget,
    world::{find_entity, get_path_pos, next_path_idx, Direction},
};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

pub struct Movement {
    pub movement_towards_target: MovementTowardsTarget,
    pub path_target_setter: Option<PathTargetSetter>,
    pub detection_based_target_setter: Option<DetectionBasedTargetSetter>,
    pub entity_target_setter: Option<EntityTargetSetter>,
}

impl Movement {
    pub fn new(speed: MovementSpeed) -> Self {
        Self {
            movement_towards_target: MovementTowardsTarget {
                target_pos: None,
                speed,
                speed_buffs: vec![],
                velocity: Vec2::ZERO,
                keep_moving_on_loss_of_target: false,
            },
            path_target_setter: None,
            detection_based_target_setter: Some(DetectionBasedTargetSetter {
                detection_range: DEFAULT_UNIT_DETECTION_RADIUS,
            }),
            entity_target_setter: None,
        }
    }
    pub fn new_projectile(target_entity_id: EntityId) -> Self {
        Self {
            movement_towards_target: MovementTowardsTarget {
                target_pos: None,
                speed: MovementSpeed::Projectile,
                speed_buffs: vec![],
                velocity: Vec2::ZERO,
                keep_moving_on_loss_of_target: true,
            },
            path_target_setter: None,
            detection_based_target_setter: None,
            entity_target_setter: Some(EntityTargetSetter {
                target_entity_id: Some(target_entity_id),
            }),
        }
    }
}
pub struct MovementTowardsTarget {
    pub target_pos: Option<Vec2>,
    pub speed: MovementSpeed,
    pub speed_buffs: Vec<ArithmeticBuff>,
    pub velocity: Vec2,
    pub keep_moving_on_loss_of_target: bool,
}

pub struct EntityTargetSetter {
    pub target_entity_id: Option<EntityId>,
}

pub struct DetectionBasedTargetSetter {
    pub detection_range: f32,
}

pub fn get_detection_range(entity: &Entity) -> Option<f32> {
    entity
        .movement
        .as_ref()
        .and_then(|movement| movement.detection_based_target_setter.as_ref())
        .map(|detection_based_target_setter| detection_based_target_setter.detection_range)
}

pub struct PathTargetSetter {
    pub path_state: Option<PathState>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PathState {
    pub path_id: PathId,
    pub target_path_idx: usize,
    pub direction: Direction,
}

impl PathState {
    pub fn incr(&mut self) {
        self.target_path_idx = next_path_idx(self.target_path_idx, self.direction);
    }
}

impl From<UnitSpawnpointTarget> for PathState {
    fn from(target: UnitSpawnpointTarget) -> Self {
        Self {
            path_id: target.path_id,
            target_path_idx: next_path_idx(target.path_idx, target.direction),
            direction: target.direction,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MovementSpeed {
    Slow,
    Default,
    Fast,
    Projectile,
    Custom(f32),
}

impl MovementSpeed {
    pub fn to_f32(&self) -> f32 {
        match self {
            MovementSpeed::Slow => 50.0,
            MovementSpeed::Default => 100.0,
            MovementSpeed::Fast => 150.0,
            MovementSpeed::Projectile => 500.0,
            MovementSpeed::Custom(speed) => *speed,
        }
    }
}

impl MovementTowardsTarget {
    pub fn get_speed(&self) -> f32 {
        apply_arithmetic_buffs(self.speed.to_f32(), &self.speed_buffs)
    }
}

impl Movement {
    pub fn update(
        entity: &mut Entity,
        dynamic_game_state: &mut DynamicGameState,
        dt: f32,
        static_game_state: &StaticGameState,
    ) {
        PathTargetSetter::update(entity, dynamic_game_state, dt, static_game_state);
        DetectionBasedTargetSetter::update(entity, dynamic_game_state, dt, static_game_state);
        EntityTargetSetter::update(entity, dynamic_game_state, dt, static_game_state);
        MovementTowardsTarget::update(entity, dynamic_game_state, dt, static_game_state);
    }
}

impl MovementTowardsTarget {
    pub fn update(
        entity: &mut Entity,
        _dynamic_game_state: &mut DynamicGameState,
        dt: f32,
        _static_game_state: &StaticGameState,
    ) {
        let Some(movement) = &mut entity.movement else {
            return;
        };
        let movement_towards_target = &mut movement.movement_towards_target;
        if let Some(target_pos) = movement_towards_target.target_pos {
            let diff = target_pos - entity.pos;
            if diff.length() < movement_towards_target.velocity.length() * dt {
                entity.pos = target_pos;
                movement_towards_target.target_pos = None;
            } else {
                movement_towards_target.velocity =
                    diff.normalize_or_zero() * movement_towards_target.get_speed();
                entity.pos += movement_towards_target.velocity * dt;
            }
        } else if movement_towards_target.keep_moving_on_loss_of_target {
            entity.pos += movement_towards_target.velocity * dt;
        }
    }
}

impl PathTargetSetter {
    pub fn update(
        entity: &mut Entity,
        _dynamic_game_state: &mut DynamicGameState,
        _dt: f32,
        static_game_state: &StaticGameState,
    ) {
        let Some(movement) = entity.movement.as_mut() else {
            return;
        };
        let Some(path_target_setter) = &mut movement.path_target_setter else {
            return;
        };
        let Some(path_state) = &mut path_target_setter.path_state else {
            return;
        };

        let mut target_pos = get_path_pos(
            &static_game_state,
            path_state.path_id,
            path_state.target_path_idx,
        );
        let pos_diff = target_pos - entity.pos;

        if pos_diff.length() < CLOSE_ENOUGH_TO_TARGET {
            path_state.incr();
            target_pos = get_path_pos(
                &static_game_state,
                path_state.path_id,
                path_state.target_path_idx,
            );
        }

        movement.movement_towards_target.target_pos = Some(target_pos);
    }
}

impl DetectionBasedTargetSetter {
    pub fn update(
        entity: &mut Entity,
        dynamic_game_state: &mut DynamicGameState,
        _dt: f32,
        _static_game_state: &StaticGameState,
    ) {
        let Some(movement) = entity.movement.as_mut() else {
            return;
        };
        let Some(detection_based_target_setter) = &mut movement.detection_based_target_setter
        else {
            return;
        };

        let detection_range = detection_based_target_setter.detection_range;

        if let Some((building_spot_target, _)) = entity.building_to_construct.clone() {
            let building_to_construct_pos = dynamic_game_state
                .building_locations
                .get(&building_spot_target.id)
                .unwrap()
                .pos;
            if (building_to_construct_pos - entity.pos).length() < detection_range {
                movement.movement_towards_target.target_pos = Some(building_to_construct_pos);
                return;
            }
        }

        for attack in &entity.attacks {
            if let Some(target_entity_to_attack) = find_target_for_attack(
                entity.pos,
                entity.owner,
                detection_range,
                attack,
                &mut dynamic_game_state.entities,
            ) {
                movement.movement_towards_target.target_pos = Some(target_entity_to_attack.pos);
                return;
            }
        }
    }
}

impl EntityTargetSetter {
    pub fn update(
        entity: &mut Entity,
        dynamic_game_state: &mut DynamicGameState,
        _dt: f32,
        _static_game_state: &StaticGameState,
    ) {
        let Some(movement) = entity.movement.as_mut() else {
            return;
        };
        let Some(entity_target_setter) = &mut movement.entity_target_setter else {
            return;
        };
        movement.movement_towards_target.target_pos = find_entity(
            &dynamic_game_state.entities,
            entity_target_setter.target_entity_id,
        )
        .map(|entity| entity.pos);
    }
}
