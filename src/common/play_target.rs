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
impl WorldPosTarget {
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
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
    UnitSpawnpoint(UnitSpawnpointTarget),
    BuildingLocation(BuildingLocationTarget),
    Entity(EntityTarget),
}

pub struct PlayArgs<'a, T> {
    pub target: &'a T,
    pub owner: PlayerId,
    pub static_game_state: &'a StaticGameState,
    pub semi_static_game_state: &'a mut SemiStaticGameState,
    pub dynamic_game_state: &'a mut DynamicGameState,
}

pub struct TargetIsInvalidArgs<'a, T> {
    pub target: &'a T,
    pub owner: PlayerId,
    pub static_game_state: &'a StaticGameState,
    pub semi_static_game_state: &'a SemiStaticGameState,
    pub dynamic_game_state: &'a DynamicGameState,
}

impl<'a, T> PlayArgs<'a, T> {
    pub fn to_target_is_invalid_args(&'a self) -> TargetIsInvalidArgs<'a, T> {
        TargetIsInvalidArgs {
            target: self.target,
            owner: self.owner,
            static_game_state: self.static_game_state,
            semi_static_game_state: self.semi_static_game_state,
            dynamic_game_state: self.dynamic_game_state,
        }
    }
}

macro_rules! try_from_play_args {
    ($target:ident, $variant:ident) => {
        impl<'a> TryFrom<PlayArgs<'a, PlayTarget>> for PlayArgs<'a, $target> {
            type Error = &'static str;

            fn try_from(play_args: PlayArgs<'a, PlayTarget>) -> Result<Self, Self::Error> {
                if let PlayTarget::$variant(target) = play_args.target {
                    Ok(PlayArgs {
                        target,
                        owner: play_args.owner,
                        static_game_state: play_args.static_game_state,
                        semi_static_game_state: play_args.semi_static_game_state,
                        dynamic_game_state: play_args.dynamic_game_state,
                    })
                } else {
                    Err(concat!("PlayTarget is not ", stringify!($variant)))
                }
            }
        }
    };
}

try_from_play_args!(WorldPosTarget, WorldPos);
try_from_play_args!(UnitSpawnpointTarget, UnitSpawnpoint);
try_from_play_args!(BuildingLocationTarget, BuildingLocation);
try_from_play_args!(EntityTarget, Entity);

pub struct SpecificPlayFn<T> {
    pub play: fn(PlayArgs<T>) -> bool,
    pub target_is_invalid: Option<fn(TargetIsInvalidArgs<T>) -> bool>,
}

impl<T> SpecificPlayFn<T> {
    pub fn new(play: fn(PlayArgs<T>) -> bool) -> Self {
        SpecificPlayFn {
            play,
            target_is_invalid: None,
        }
    }
    pub fn with_target_is_invalid(
        mut self,
        target_is_invalid: fn(TargetIsInvalidArgs<T>) -> bool,
    ) -> Self {
        self.target_is_invalid = Some(target_is_invalid);
        self
    }
    pub fn target_is_invalid(&self, target_is_invalid_args: TargetIsInvalidArgs<T>) -> bool {
        match self.target_is_invalid {
            Some(f) => f(target_is_invalid_args),
            None => false,
        }
    }
    pub fn exec(&self, args: PlayArgs<T>) -> bool {
        match self.target_is_invalid {
            Some(f) => f(args.to_target_is_invalid_args()),
            None => false,
        };
        (self.play)(args)
    }
}

pub enum PlayFn {
    WorldPos(SpecificPlayFn<WorldPosTarget>),
    UnitSpawnPoint(SpecificPlayFn<UnitSpawnpointTarget>),
    BuildingLocation(SpecificPlayFn<BuildingLocationTarget>),
    Entity(SpecificPlayFn<EntityTarget>),
}

impl PlayFn {
    pub fn exec(&self, play_args: PlayArgs<PlayTarget>) -> bool {
        match self._exec(play_args) {
            Ok(result) => result,
            Err(e) => {
                println!("Error: {}", e);
                debug_assert!(false);
                false
            }
        }
    }
    fn _exec(&self, play_args: PlayArgs<PlayTarget>) -> Result<bool, &'static str> {
        match self {
            PlayFn::WorldPos(specific_play_fn) => {
                Ok(specific_play_fn.exec(PlayArgs::try_from(play_args)?))
            }
            PlayFn::UnitSpawnPoint(specific_play_fn) => {
                Ok(specific_play_fn.exec(PlayArgs::try_from(play_args)?))
            }
            PlayFn::BuildingLocation(specific_play_fn) => {
                Ok(specific_play_fn.exec(PlayArgs::try_from(play_args)?))
            }
            PlayFn::Entity(specific_play_fn) => {
                Ok(specific_play_fn.exec(PlayArgs::try_from(play_args)?))
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
