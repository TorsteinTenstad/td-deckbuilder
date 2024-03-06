use crate::{
    entity::EntityInstance,
    game_state::{DynamicGameState, SemiStaticGameState, StaticGameState},
};

pub struct UpdateArgs<'a> {
    pub static_game_state: &'a StaticGameState,
    pub semi_static_game_state: &'a mut SemiStaticGameState,
    pub dynamic_game_state: &'a mut DynamicGameState,
    pub entity_instance: &'a mut EntityInstance,
    pub dt: f32,
}
