use crate::{
    game_state::{DynamicGameState, SemiStaticGameState, StaticGameState},
    ids::{BuildingLocationId, EntityId, PathId, PlayerId},
    rect_transform::RectTransform,
    world::{get_path_pos, Direction},
};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldPosTarget {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitSpawnpointTarget {
    pub path_id: PathId,
    pub path_idx: usize,
    pub direction: Direction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingLocationTarget {
    pub id: BuildingLocationId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTarget {
    pub id: EntityId,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayTarget {
    WorldPos(WorldPosTarget),
    UnitSpawnPoint(UnitSpawnpointTarget),
    BuildingSpot(BuildingLocationTarget),
    Entity(EntityTarget),
}

pub struct SpecificPlayFn<T> {
    pub play:
        fn(T, PlayerId, &StaticGameState, &mut SemiStaticGameState, &mut DynamicGameState) -> bool,
    pub target_is_invalid:
        Option<fn(&T, PlayerId, &StaticGameState, &SemiStaticGameState, &DynamicGameState) -> bool>,
}

impl<T> SpecificPlayFn<T> {
    pub fn new(
        play: fn(
            T,
            PlayerId,
            &StaticGameState,
            &mut SemiStaticGameState,
            &mut DynamicGameState,
        ) -> bool,
    ) -> Self {
        SpecificPlayFn {
            play,
            target_is_invalid: None,
        }
    }
    pub fn with_target_is_invalid(
        mut self,
        target_is_invalid: fn(
            &T,
            PlayerId,
            &StaticGameState,
            &SemiStaticGameState,
            &DynamicGameState,
        ) -> bool,
    ) -> Self {
        self.target_is_invalid = Some(target_is_invalid);
        self
    }
}

pub enum PlayFn {
    WorldPos(SpecificPlayFn<WorldPosTarget>),
    UnitSpawnPoint(SpecificPlayFn<UnitSpawnpointTarget>),
    BuildingLocation(SpecificPlayFn<BuildingLocationTarget>),
    Entity(SpecificPlayFn<EntityTarget>),
}

impl PlayFn {
    pub fn exec(
        &self,
        target: PlayTarget,
        owner: PlayerId,
        static_game_state: &StaticGameState,
        semi_static_game_state: &mut SemiStaticGameState,
        dynamic_game_state: &mut DynamicGameState,
    ) -> bool {
        match (self, target) {
            (PlayFn::WorldPos(specific_play_fn), PlayTarget::WorldPos(target)) => {
                if specific_play_fn.target_is_invalid.is_some_and(|f| {
                    f(
                        &target,
                        owner,
                        static_game_state,
                        semi_static_game_state,
                        dynamic_game_state,
                    )
                }) {
                    return false;
                }
                (specific_play_fn.play)(
                    target,
                    owner,
                    static_game_state,
                    semi_static_game_state,
                    dynamic_game_state,
                )
            }
            (PlayFn::UnitSpawnPoint(specific_play_fn), PlayTarget::UnitSpawnPoint(target)) => {
                if specific_play_fn.target_is_invalid.is_some_and(|f| {
                    f(
                        &target,
                        owner,
                        static_game_state,
                        semi_static_game_state,
                        dynamic_game_state,
                    )
                }) {
                    return false;
                }
                (specific_play_fn.play)(
                    target,
                    owner,
                    static_game_state,
                    semi_static_game_state,
                    dynamic_game_state,
                )
            }
            (PlayFn::BuildingLocation(specific_play_fn), PlayTarget::BuildingSpot(target)) => {
                if specific_play_fn.target_is_invalid.is_some_and(|f| {
                    f(
                        &target,
                        owner,
                        static_game_state,
                        semi_static_game_state,
                        dynamic_game_state,
                    )
                }) {
                    return false;
                }
                (specific_play_fn.play)(
                    target,
                    owner,
                    static_game_state,
                    semi_static_game_state,
                    dynamic_game_state,
                )
            }
            (PlayFn::Entity(specific_play_fn), PlayTarget::Entity(target)) => {
                if specific_play_fn.target_is_invalid.is_some_and(|f| {
                    f(
                        &target,
                        owner,
                        static_game_state,
                        semi_static_game_state,
                        dynamic_game_state,
                    )
                }) {
                    return false;
                }
                (specific_play_fn.play)(
                    target,
                    owner,
                    static_game_state,
                    semi_static_game_state,
                    dynamic_game_state,
                )
            }
            _ => {
                println!("Invalid target for play fn");
                false
            }
        }
    }
}

pub fn unit_spawnpoint_target_transform(
    target: &UnitSpawnpointTarget,
    static_game_state: &StaticGameState,
) -> RectTransform {
    let UnitSpawnpointTarget {
        path_id,
        path_idx,
        direction: _,
    } = target;

    let Vec2 { x, y } = get_path_pos(static_game_state, *path_id, *path_idx);
    RectTransform {
        x,
        y,
        w: 50.0,
        h: 50.0,
        offset: Vec2::splat(0.5),
        ..Default::default()
    }
}
