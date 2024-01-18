use crate::{
    buff::{apply_buffs, Buff},
    entity::{Entity, EntityState, EntityTag},
    find_target::find_enemy_entity_in_range,
    game_state::{DynamicGameState, StaticGameState},
    ids::{EntityId, PathId},
    play_target::UnitSpawnpointTarget,
    serde_defs::Vec2Def,
    world::{find_entity, get_path_pos, next_path_idx, world_place_building, Direction},
};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum MovementBehavior {
    Bullet(BulletMovementBehavior),
    Path(PathMovementBehavior),
    None,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct BulletMovementBehavior {
    #[serde(with = "Vec2Def")]
    pub velocity: Vec2,
    pub target_entity_id: Option<EntityId>,
    pub speed: MovementSpeed,
    pub speed_buffs: Vec<Buff>,
}

impl BulletMovementBehavior {
    pub fn get_speed(&self) -> f32 {
        apply_buffs(self.speed.to_f32(), &self.speed_buffs)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PathMovementBehavior {
    pub speed: MovementSpeed,
    pub detection_radius: f32,
    pub path_state: Option<PathState>,
    pub speed_buffs: Vec<Buff>,
}

impl PathMovementBehavior {
    pub fn new(speed: MovementSpeed, detection_radius: f32) -> Self {
        Self {
            speed,
            detection_radius,
            path_state: None,
            speed_buffs: Vec::new(),
        }
    }
    pub fn get_speed(&self) -> f32 {
        apply_buffs(self.speed.to_f32(), &self.speed_buffs)
    }
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

impl MovementBehavior {
    pub fn update(
        entity: &mut Entity,
        dynamic_game_state: &mut DynamicGameState,
        dt: f32,
        static_game_state: &StaticGameState,
    ) {
        match &mut entity.movement_behavior {
            MovementBehavior::Path(path_movement_behavior) => {
                let path_state = path_movement_behavior.path_state.as_mut().unwrap();
                if path_state.target_path_idx
                    < static_game_state
                        .paths
                        .get(&path_state.path_id)
                        .unwrap()
                        .len()
                {
                    let can_target_out_of_path: Vec<EntityTag> = entity
                        .attacks
                        .iter()
                        .flat_map(|attack| {
                            attack
                                .can_target
                                .clone()
                                .into_iter()
                                .filter(|tag| tag != &EntityTag::Unit)
                        })
                        .collect();

                    let update_position = |pos: &mut Vec2, target_pos: Vec2| -> bool {
                        let delta = target_pos - *pos;
                        *pos +=
                            delta.normalize_or_zero() * path_movement_behavior.speed.to_f32() * dt;
                        let updated_delta = target_pos - *pos;
                        delta.length_squared() < updated_delta.length_squared()
                    };

                    match entity.building_to_construct.clone() {
                        Some((building_spot_target, entity_blueprint))
                            if (dynamic_game_state
                                .building_locations
                                .get(&building_spot_target.id)
                                .unwrap()
                                .pos
                                - entity.pos)
                                .length()
                                < path_movement_behavior.detection_radius =>
                        {
                            let building_pos = dynamic_game_state
                                .building_locations
                                .get_mut(&building_spot_target.id)
                                .unwrap()
                                .pos;
                            if update_position(&mut entity.pos, building_pos) {
                                world_place_building(
                                    dynamic_game_state,
                                    entity_blueprint.create(entity.owner),
                                    building_spot_target,
                                );
                                entity.state = EntityState::Dead;
                            }
                        }
                        _ => {
                            // TODO: This is inconsistent with other logic, but works because we never go out of the path to heal
                            match find_enemy_entity_in_range(
                                entity.pos,
                                entity.owner,
                                path_movement_behavior.detection_radius,
                                &can_target_out_of_path,
                                &mut dynamic_game_state.entities,
                            ) {
                                Some(target_entity) => {
                                    update_position(&mut entity.pos, target_entity.pos);
                                }
                                None => {
                                    let target_pos = get_path_pos(
                                        static_game_state,
                                        path_state.path_id,
                                        path_state.target_path_idx,
                                    );

                                    if update_position(&mut entity.pos, target_pos) {
                                        path_state.incr();
                                    }
                                }
                            }
                        }
                    };
                }
            }
            MovementBehavior::Bullet(bullet_movement_behavior) => {
                bullet_movement_behavior.velocity = find_entity(
                    &mut dynamic_game_state.entities,
                    bullet_movement_behavior.target_entity_id,
                )
                .map(|target_entity| {
                    (target_entity.pos - entity.pos).normalize_or_zero()
                        * bullet_movement_behavior.get_speed()
                })
                .unwrap_or(bullet_movement_behavior.velocity);

                entity.pos += bullet_movement_behavior.velocity * dt;
            }
            MovementBehavior::None => {}
        }
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
