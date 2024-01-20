use crate::{
    component_movement::{get_detection_range, PathTargetSetter},
    entity::Entity,
    game_state::{DynamicGameState, StaticGameState},
    get_unit_spawnpoints::get_unit_spawnpoints,
    ids::{BuildingLocationId, EntityId, PathId, PlayerId},
    play_target::{BuildingSpotTarget, UnitSpawnpointTarget},
    serde_defs::Vec2Def,
};
use itertools::Itertools;
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct BuildingLocation {
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub entity_id: Option<EntityId>,
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
    pub fn flipped(&self) -> Self {
        match self {
            Direction::Positive => Direction::Negative,
            Direction::Negative => Direction::Positive,
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
pub fn get_path_pos(static_game_state: &StaticGameState, path_id: PathId, path_idx: usize) -> Vec2 {
    let path = static_game_state.paths.get(&path_id).unwrap();
    let (x, y) = path.get(path_idx).unwrap_or(path.last().unwrap());
    Vec2 { x: *x, y: *y }
}

pub fn path_length(path: &Vec<(f32, f32)>, start_idx: usize, stop_idx: usize) -> f32 {
    if start_idx > stop_idx {
        return path_length(path, stop_idx, start_idx);
    }
    assert!(start_idx <= stop_idx);
    assert!(start_idx < path.len());
    assert!(stop_idx < path.len());

    let mut length = 0.0;
    for i in start_idx..stop_idx {
        let (x1, y1) = path[i];
        let (x2, y2) = path[i + 1];
        length += (x2 - x1).powi(2) + (y2 - y1).powi(2);
    }
    return length.sqrt();
}

pub fn path_length_from_spawnpoint(
    static_game_state: &StaticGameState,
    spawnpoint: &UnitSpawnpointTarget,
    to_pos: Vec2,
    detection_range: f32,
    direction: Direction,
) -> Option<f32> {
    let path = static_game_state.paths.get(&spawnpoint.path_id).unwrap();
    let predicate = |(_, (x, y)): &(usize, &(f32, f32))| {
        ((*x - to_pos.x).powi(2) + (*y - to_pos.y).powi(2)) < detection_range.powi(2)
    };
    let first_path_idx_within_building_range = match direction {
        Direction::Positive => path.iter().enumerate().find(predicate),
        Direction::Negative => path.iter().rev().enumerate().find(predicate),
    }
    .map(|(idx, _)| idx)?;

    Some(path_length(
        path,
        spawnpoint.path_idx,
        first_path_idx_within_building_range,
    ))
}

pub fn world_get_building_locations_on_path(
    path_id: PathId,
    search_range: f32,
    static_game_state: &StaticGameState,
    dynamic_game_state: &DynamicGameState,
) -> Vec<(BuildingLocationId, BuildingLocation, usize)> {
    let path = static_game_state.paths.get(&path_id).unwrap();
    path.iter()
        .enumerate()
        .flat_map(|(path_idx, (x, y))| {
            let pos = Vec2 { x: *x, y: *y };
            dynamic_game_state
                .building_locations
                .iter()
                .filter(move |(_building_location_id, building_location)| {
                    (building_location.pos - pos).length_squared() < search_range.powi(2)
                })
                .map(move |(building_location_id, building_location)| {
                    (
                        building_location_id.clone(),
                        building_location.clone(),
                        path_idx,
                    )
                })
        })
        .collect_vec()
}

pub fn world_get_furthest_planned_or_existing_building(
    path_id: PathId,
    player_id: PlayerId,
    search_range: f32,
    static_game_state: &StaticGameState,
    dynamic_game_state: &DynamicGameState,
) -> Option<(BuildingLocationId, BuildingLocation, usize)> {
    let player_direction = dynamic_game_state
        .players
        .get(&player_id)
        .unwrap()
        .direction;
    let building_locations_along_path = world_get_building_locations_on_path(
        path_id,
        search_range,
        static_game_state,
        &dynamic_game_state,
    );
    let mut owned_building_locations_along_path = building_locations_along_path.iter().filter(
        |(building_location_id, building_location, _)| {
            find_entity(&dynamic_game_state.entities, building_location.entity_id)
                .is_some_and(|entity| entity.owner == player_id)
                || dynamic_game_state.entities.iter().any(|entity| {
                    entity.owner == player_id
                        && entity.building_to_construct.as_ref().is_some_and(
                            |(building_spot_target, _)| {
                                building_spot_target.id == *building_location_id
                            },
                        )
                })
        },
    );
    match player_direction {
        Direction::Positive => owned_building_locations_along_path.last().cloned(),
        Direction::Negative => owned_building_locations_along_path.next().cloned(),
    }
}

pub fn find_entity_mut(entities: &mut Vec<Entity>, id: Option<EntityId>) -> Option<&mut Entity> {
    return id.and_then(|id| entities.iter_mut().find(|entity| entity.id == id));
}

pub fn find_entity(entities: &Vec<Entity>, id: Option<EntityId>) -> Option<&Entity> {
    return id.and_then(|id| entities.iter().find(|entity| entity.id == id));
}

pub fn world_place_path_entity(
    static_game_state: &StaticGameState,
    dynamic_game_state: &mut DynamicGameState,
    mut entity: Entity,
    target: UnitSpawnpointTarget,
) {
    let Some(movement) = &mut entity.movement else {
        assert!(false);
        return;
    };
    entity.pos = get_path_pos(static_game_state, target.path_id, target.path_idx);
    movement.path_target_setter = Some(PathTargetSetter {
        path_state: Some(target.into()),
    });
    dynamic_game_state.entities.push(entity);
}

pub fn world_place_builder(
    dynamic_game_state: &mut DynamicGameState,
    static_game_state: &StaticGameState,
    owner: PlayerId,
    mut entity: Entity,
    target: BuildingSpotTarget,
) -> bool {
    let building_pos = dynamic_game_state
        .building_locations
        .get(&target.id)
        .unwrap()
        .pos;

    let Some(detection_range) = get_detection_range(&entity) else {
        assert!(false);
        return false;
    };

    let Some((building_spot_target, _)) = entity.building_to_construct.as_mut() else {
        assert!(false);
        return false;
    };
    *building_spot_target = target;

    let spawnpoint_target = get_unit_spawnpoints(owner, static_game_state, dynamic_game_state)
        .iter()
        .filter_map(|spawnpoint| {
            path_length_from_spawnpoint(
                static_game_state,
                spawnpoint,
                building_pos,
                detection_range,
                dynamic_game_state.players.get(&owner).unwrap().direction,
            )
            .map(|len| (spawnpoint, len))
        })
        .min_by(|(_, len_a), (_, len_b)| len_a.partial_cmp(&&len_b).unwrap())
        .map(|(spawnpoint, _len)| spawnpoint.clone());
    let Some(spawnpoint_target) = spawnpoint_target else {
        return false;
    };

    world_place_path_entity(
        static_game_state,
        dynamic_game_state,
        entity,
        spawnpoint_target,
    );
    true
}

pub fn world_place_building(
    dynamic_game_state: &mut DynamicGameState,
    mut entity: Entity,
    target: BuildingSpotTarget,
) -> bool {
    let BuildingLocation { pos, entity_id } = dynamic_game_state
        .building_locations
        .get_mut(&target.id)
        .unwrap();
    if let Some(_) = entity_id {
        return false;
    }
    entity.pos = *pos;
    *entity_id = Some(entity.id);
    dynamic_game_state.entities.push(entity);
    return true;
}
