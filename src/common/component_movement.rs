use std::f32::consts::PI;

use crate::{
    buff::{apply_arithmetic_buffs, ArithmeticBuff},
    config::CLOSE_ENOUGH_TO_TARGET,
    entity::{AbilityFlag, Entity, EntityTag},
    entity_blueprint::DEFAULT_UNIT_DETECTION_RADIUS,
    find_target::find_target_for_attack,
    game_state::{DynamicGameState, SemiStaticGameState, StaticGameState},
    ids::{EntityId, PathId},
    play_target::UnitSpawnpointTarget,
    serde_defs::Vec2Def,
    world::{
        find_entity, get_path_pos, next_path_idx, world_get_furthest_planned_or_existing_building,
        Direction,
    },
};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementTowardsTarget {
    #[serde(skip)] // Not used by client, skipped to avoid hassle of serializing Option<Vec2>
    pub target_pos: Option<Vec2>,
    pub speed: MovementSpeed,
    pub speed_buffs: Vec<ArithmeticBuff>,
    #[serde(with = "Vec2Def")]
    pub velocity: Vec2,
    pub keep_moving_on_loss_of_target: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTargetSetter {
    pub target_entity_id: Option<EntityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathTargetSetter {
    pub path_state: Option<PathState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathState {
    pub path_id: PathId,
    pub target_path_idx: usize,
    pub direction: Direction,
}

pub fn get_path_id(entity: &Entity) -> Option<PathId> {
    entity
        .movement
        .as_ref()
        .and_then(|movement| movement.path_target_setter.as_ref())
        .and_then(|path_target_setter| path_target_setter.path_state.as_ref())
        .map(|path_state| path_state.path_id)
}

impl PathState {
    pub fn incr(&mut self, static_game_state: &StaticGameState) {
        self.target_path_idx = next_path_idx(self.target_path_idx, self.direction.clone());
        self.target_path_idx = usize::min(
            self.target_path_idx,
            static_game_state.paths.get(&self.path_id).unwrap().len() - 1,
        );
    }

    pub fn set_direction(&mut self, direction: Direction) {
        if direction == self.direction {
            return;
        }
        self.direction = direction;
        self.target_path_idx = next_path_idx(self.target_path_idx, self.direction.clone());
    }
}

impl From<UnitSpawnpointTarget> for PathState {
    fn from(target: UnitSpawnpointTarget) -> Self {
        Self {
            path_id: target.path_id,
            target_path_idx: next_path_idx(target.path_idx, target.direction.clone()),
            direction: target.direction,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MovementSpeed {
    VerySlow,
    Slow,
    Default,
    Fast,
    VeryFast,
    Projectile,
    Custom(f32),
}

impl MovementSpeed {
    pub fn to_f32(&self) -> f32 {
        let default_speed = 50.0;
        match self {
            MovementSpeed::VerySlow => default_speed / 2.0,
            MovementSpeed::Slow => default_speed / 1.5,
            MovementSpeed::Default => default_speed,
            MovementSpeed::Fast => default_speed * 1.5,
            MovementSpeed::VeryFast => default_speed * 2.0,
            MovementSpeed::Projectile => default_speed * 2.0,
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
        static_game_state: &StaticGameState,
        semi_static_game_state: &SemiStaticGameState,
        dynamic_game_state: &mut DynamicGameState,
        entity: &mut Entity,
        dt: f32,
    ) {
        PathTargetSetter::update(
            static_game_state,
            semi_static_game_state,
            dynamic_game_state,
            entity,
            dt,
        );
        DetectionBasedTargetSetter::update(
            static_game_state,
            semi_static_game_state,
            dynamic_game_state,
            entity,
            dt,
        );
        EntityTargetSetter::update(
            static_game_state,
            semi_static_game_state,
            dynamic_game_state,
            entity,
            dt,
        );
        MovementTowardsTarget::update(
            static_game_state,
            semi_static_game_state,
            dynamic_game_state,
            entity,
            dt,
        );
    }
}

impl MovementTowardsTarget {
    pub fn update(
        _static_game_state: &StaticGameState,
        _semi_static_game_state: &SemiStaticGameState,
        _dynamic_game_state: &mut DynamicGameState,
        entity: &mut Entity,
        dt: f32,
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
        static_game_state: &StaticGameState,
        semi_static_game_state: &SemiStaticGameState,
        dynamic_game_state: &mut DynamicGameState,
        entity: &mut Entity,
        _dt: f32,
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

        if entity.ability_flags.contains(&AbilityFlag::Protector) {
            if let Some((_, _, building_location_closest_path_idx)) =
                world_get_furthest_planned_or_existing_building(
                    path_state.path_id,
                    entity.owner,
                    DEFAULT_UNIT_DETECTION_RADIUS, //TODO: Maybe not hardcode?
                    static_game_state,
                    semi_static_game_state,
                    dynamic_game_state,
                )
            {
                if (get_path_pos(
                    static_game_state,
                    path_state.path_id,
                    building_location_closest_path_idx,
                ) - entity.pos)
                    .length()
                    < CLOSE_ENOUGH_TO_TARGET
                {
                    movement.movement_towards_target.target_pos = None;
                    return;
                }
                if building_location_closest_path_idx != path_state.target_path_idx {
                    path_state.set_direction(
                        match building_location_closest_path_idx > path_state.target_path_idx {
                            true => Direction::Positive,
                            false => Direction::Negative,
                        },
                    );
                }
            } else {
                path_state.set_direction(
                    dynamic_game_state
                        .players
                        .get(&entity.owner)
                        .unwrap()
                        .direction
                        .flipped(),
                );
            }
        }

        let mut target_pos = get_path_pos(
            static_game_state,
            path_state.path_id,
            path_state.target_path_idx,
        );
        let pos_diff = target_pos - entity.pos;

        if pos_diff.length() < CLOSE_ENOUGH_TO_TARGET {
            path_state.incr(static_game_state);
            target_pos = get_path_pos(
                static_game_state,
                path_state.path_id,
                path_state.target_path_idx,
            );
        }

        movement.movement_towards_target.target_pos = Some(target_pos);
    }
}

impl DetectionBasedTargetSetter {
    pub fn update(
        _static_game_state: &StaticGameState,
        semi_static_game_state: &SemiStaticGameState,
        dynamic_game_state: &mut DynamicGameState,
        entity: &mut Entity,
        _dt: f32,
    ) {
        let entity_path_id = get_path_id(entity);
        let Some(movement) = entity.movement.as_mut() else {
            return;
        };
        let Some(detection_based_target_setter) = &mut movement.detection_based_target_setter
        else {
            return;
        };

        let detection_range = detection_based_target_setter.detection_range;

        if let Some((building_spot_target, _)) = entity.building_to_construct.clone() {
            let building_to_construct_pos = semi_static_game_state
                .building_locations()
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
                entity.id,
                entity.tag.clone(),
                entity.pos,
                entity.owner,
                entity.spy.as_ref(),
                detection_range,
                attack,
                &mut dynamic_game_state.entities,
            ) {
                if entity_path_id.is_some_and(|id| {
                    get_path_id(target_entity_to_attack).is_some_and(|other_id| id != other_id)
                }) {
                    continue;
                }
                movement.movement_towards_target.target_pos = Some(target_entity_to_attack.pos);
                return;
            }
        }
    }
}

impl EntityTargetSetter {
    pub fn update(
        _static_game_state: &StaticGameState,
        _semi_static_game_state: &SemiStaticGameState,
        dynamic_game_state: &mut DynamicGameState,
        entity: &mut Entity,
        _dt: f32,
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
