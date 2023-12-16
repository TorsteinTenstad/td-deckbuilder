use common::{Entity, EntityTag};
use macroquad::{
    input::{is_key_down, is_mouse_button_released, mouse_position},
    math::Vec2,
    miniquad::MouseButton,
    window::{screen_height, screen_width},
};

use crate::{
    config::CARD_PANEL_RELATIVE_HEIGHT,
    draw::{to_world_x, to_world_y},
    ClientGameState,
};

#[derive(Default)]
pub struct GameInput {
    pub mouse_in_world: bool,
    pub mouse_over_occupied_tile: bool,
}

pub fn mouse_position_vec() -> Vec2 {
    let (x, y) = mouse_position();
    Vec2 { x, y }
}

pub fn mouse_world_position() -> Vec2 {
    let Vec2 { x, y } = mouse_position_vec();
    Vec2 {
        x: to_world_x(x),
        y: to_world_y(y),
    }
}

pub fn pos_in_rect(pos: Vec2, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
    x0 <= pos.x && pos.x <= x1 && y0 <= pos.y && pos.y <= y1
}

pub fn tower_at_tile(state: &ClientGameState, pos: Vec2) -> Option<&Entity> {
    state.dynamic_game_state.entities.values().find(|entity| {
        entity.tag == EntityTag::Tower
            && entity.pos.x as i32 == pos.x as i32
            && entity.pos.y as i32 == pos.y as i32
    })
}

pub fn main_input(state: &mut ClientGameState) {
    state.input.mouse_in_world = pos_in_rect(
        mouse_position_vec(),
        0.0,
        0.0,
        screen_width(),
        screen_height() * (1.0 - CARD_PANEL_RELATIVE_HEIGHT),
    );

    state.input.mouse_over_occupied_tile = tower_at_tile(&state, mouse_world_position()).is_some();

    if is_mouse_button_released(MouseButton::Left) {
        state.selected_entity_id =
            state
                .dynamic_game_state
                .entities
                .iter()
                .find_map(|(id, entity)| {
                    ((entity.pos - mouse_world_position()).length() < entity.radius).then(|| *id)
                });
    }

    //Card drawing parameter adjustment
    {
        if is_key_down(macroquad::prelude::KeyCode::L) {
            state.card_delta_angle += 0.05 * state.dt;
            dbg!(state.card_delta_angle);
        }
        if is_key_down(macroquad::prelude::KeyCode::J) {
            state.card_delta_angle -= 0.05 * state.dt;
            dbg!(state.card_delta_angle);
        }
        if is_key_down(macroquad::prelude::KeyCode::I) {
            state.relative_splay_radius += 0.5 * state.dt;
            dbg!(state.relative_splay_radius);
        }
        if is_key_down(macroquad::prelude::KeyCode::K) {
            state.relative_splay_radius -= 0.5 * state.dt;
            dbg!(state.relative_splay_radius);
        }
    }
}
