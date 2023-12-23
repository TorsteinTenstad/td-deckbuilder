use crate::{
    entity::{Entity, EntityState, EntityTag},
    game_state::{DynamicGameState, StaticGameState},
    ids::{EntityId, PathId},
    play_target::UnitSpawnpointTarget,
    serde_defs::Vec2Def,
    world::{
        find_entity, find_entity_in_range, get_path_pos, next_path_idx, world_place_building,
        Direction,
    },
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
pub struct BulletMovementBehavior {
    #[serde(with = "Vec2Def")]
    pub velocity: Vec2,
    pub target_entity_id: Option<EntityId>,
    pub speed: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PathMovementBehavior {
    pub speed: f32,
    pub detection_radius: f32,
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
                    let can_target: Vec<EntityTag> = entity
                        .attacks
                        .iter()
                        .flat_map(|attack| attack.can_target.clone().into_iter())
                        .collect();

                    let update_position = |pos: &mut Vec2, target_pos: Vec2| -> bool {
                        let delta = target_pos - *pos;
                        *pos += delta.normalize_or_zero() * path_movement_behavior.speed * dt;
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
                            match find_entity_in_range(
                                entity.pos,
                                entity.owner,
                                path_movement_behavior.detection_radius,
                                &can_target,
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
            MovementBehavior::Bullet(BulletMovementBehavior {
                speed,
                velocity,
                target_entity_id,
            }) => {
                *velocity = find_entity(&mut dynamic_game_state.entities, *target_entity_id)
                    .map(|target_entity| {
                        (target_entity.pos - entity.pos).normalize_or_zero() * *speed
                    })
                    .unwrap_or(*velocity);

                entity.pos += *velocity * dt;
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
