use crate::{
    component_movement::{get_detection_range, PathTargetSetter},
    entity::{Entity, EntityInstance},
    entity_blueprint::EntityBlueprint,
    game_state::{DynamicGameState, SemiStaticGameState, StaticGameState},
    get_unit_spawnpoints::get_unit_spawnpoints,
    ids::{BuildingLocationId, EntityId, PathId, PlayerId},
    play_target::{BuildingSpotTarget, UnitSpawnpointTarget},
    serde_defs::Vec2Def,
};
use itertools::Itertools;
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingLocation {
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub entity_id: Option<EntityId>,
    pub zoning: Zoning,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum Zoning {
    Normal,
    Commerce,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
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
    length.sqrt()
}

pub fn path_length_from_spawnpoint(
    static_game_state: &StaticGameState,
    spawnpoint: &UnitSpawnpointTarget,
    to_pos: Vec2,
    detection_range: f32,
) -> Option<(usize, f32)> {
    let path = static_game_state.paths.get(&spawnpoint.path_id).unwrap();
    let path_nodes_in_range_predicate = |(_, (x, y)): &(usize, &(f32, f32))| {
        ((*x - to_pos.x).powi(2) + (*y - to_pos.y).powi(2)) < detection_range.powi(2)
    };
    let path_nodes_in_range = path
        .iter()
        .enumerate()
        .filter(path_nodes_in_range_predicate);

    let path_lengths =
        path_nodes_in_range.map(|(idx, _)| (idx, path_length(path, spawnpoint.path_idx, idx)));

    path_lengths.min_by(|(_, len_a), (_, len_b)| len_a.partial_cmp(len_b).unwrap())
}

pub fn world_get_building_locations_on_path(
    path_id: PathId,
    search_range: f32,
    static_game_state: &StaticGameState,
    semi_static_game_state: &SemiStaticGameState,
) -> Vec<(BuildingLocationId, BuildingLocation, usize)> {
    let path = static_game_state.paths.get(&path_id).unwrap();
    path.iter()
        .enumerate()
        .flat_map(|(path_idx, (x, y))| {
            let pos = Vec2 { x: *x, y: *y };
            semi_static_game_state
                .building_locations()
                .iter()
                .filter(move |(_building_location_id, building_location)| {
                    (building_location.pos - pos).length_squared() < search_range.powi(2)
                })
                .map(move |(building_location_id, building_location)| {
                    (*building_location_id, building_location.clone(), path_idx)
                })
        })
        .collect_vec()
}

pub fn world_get_furthest_planned_or_existing_building(
    path_id: PathId,
    player_id: PlayerId,
    search_range: f32,
    static_game_state: &StaticGameState,
    semi_static_game_state: &SemiStaticGameState,
    dynamic_game_state: &DynamicGameState,
) -> Option<(BuildingLocationId, BuildingLocation, usize)> {
    let player_direction = dynamic_game_state
        .players
        .get(&player_id)
        .unwrap()
        .direction
        .clone();
    let building_locations_along_path = world_get_building_locations_on_path(
        path_id,
        search_range,
        static_game_state,
        semi_static_game_state,
    );
    let mut owned_building_locations_along_path = building_locations_along_path.iter().filter(
        |(building_location_id, building_location, _)| {
            find_entity(&dynamic_game_state.entities, building_location.entity_id)
                .is_some_and(|entity| entity.owner == player_id)
                || dynamic_game_state.entities.iter().any(|entity_instance| {
                    entity_instance.owner == player_id
                        && entity_instance
                            .entity
                            .building_to_construct
                            .as_ref()
                            .is_some_and(|(building_spot_target, _)| {
                                building_spot_target.id == *building_location_id
                            })
                })
        },
    );
    match player_direction {
        Direction::Positive => owned_building_locations_along_path.last().cloned(),
        Direction::Negative => owned_building_locations_along_path.next().cloned(),
    }
}

pub fn find_entity_mut(
    entities: &mut [EntityInstance],
    id: Option<EntityId>,
) -> Option<&mut EntityInstance> {
    return id.and_then(|id| entities.iter_mut().find(|entity| entity.id == id));
}

pub fn find_entity(entities: &[EntityInstance], id: Option<EntityId>) -> Option<&EntityInstance> {
    return id.and_then(|id| entities.iter().find(|entity| entity.id == id));
}

pub fn world_place_path_entity(
    static_game_state: &StaticGameState,
    dynamic_game_state: &mut DynamicGameState,
    target: UnitSpawnpointTarget,
    mut entity: Entity,
    owner: PlayerId,
) -> Option<EntityId> {
    let Some(movement) = &mut entity.movement else {
        debug_assert!(false);
        return None;
    };
    let pos = get_path_pos(static_game_state, target.path_id, target.path_idx);
    movement.path_target_setter = Some(PathTargetSetter {
        path_state: Some(target.into()),
    });
    let entity_instance = entity.instantiate(owner, pos);
    let entity_id = entity_instance.id;
    dynamic_game_state.entities.push(entity_instance);
    Some(entity_id)
}

pub fn world_place_builder(
    static_game_state: &StaticGameState,
    semi_static_game_state: &SemiStaticGameState,
    dynamic_game_state: &mut DynamicGameState,
    target: BuildingSpotTarget,
    builder_entity: Entity,
    building_blueprint: EntityBlueprint,
    owner: PlayerId,
) -> bool {
    let mut builder_entity = builder_entity;
    let building_pos = semi_static_game_state
        .building_locations()
        .get(&target.id)
        .unwrap()
        .pos;
    let Some(detection_range) = get_detection_range(&builder_entity) else {
        debug_assert!(false);
        return false;
    };

    builder_entity.building_to_construct = Some((target, building_blueprint.clone()));

    let Some((target_path_idx, mut spawnpoint_target)) =
        get_unit_spawnpoints(owner, static_game_state, dynamic_game_state)
            .iter()
            .filter_map(|spawnpoint| {
                path_length_from_spawnpoint(
                    static_game_state,
                    spawnpoint,
                    building_pos,
                    detection_range,
                )
                .map(|(target_path_idx, len)| (spawnpoint, target_path_idx, len))
            })
            .min_by(|(_, _, len_a), (_, _, len_b)| len_a.partial_cmp(len_b).unwrap())
            .map(|(spawnpoint, target_path_idx, _len)| (target_path_idx, spawnpoint.clone()))
    else {
        return false;
    };

    spawnpoint_target.direction = if target_path_idx < spawnpoint_target.path_idx {
        Direction::Negative
    } else {
        Direction::Positive
    };
    world_place_path_entity(
        static_game_state,
        dynamic_game_state,
        spawnpoint_target,
        builder_entity,
        owner,
    );
    true
}

pub fn world_place_building(
    semi_static_game_state: &mut SemiStaticGameState,
    dynamic_game_state: &mut DynamicGameState,
    entity: Entity,
    building_location_id: &BuildingLocationId,
    owner: PlayerId,
) -> bool {
    let BuildingLocation { pos, entity_id, .. } = semi_static_game_state
        .building_locations_mut()
        .get_mut(building_location_id)
        .unwrap();
    if entity_id.is_some() {
        return false;
    }
    let entity_instance = entity.instantiate(owner, *pos);
    *entity_id = Some(entity_instance.id);
    dynamic_game_state.entities.push(entity_instance);
    true
}
