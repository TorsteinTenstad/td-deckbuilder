use macroquad::{
    input::{is_key_down, is_mouse_button_released, mouse_position},
    math::Vec2,
    miniquad::MouseButton,
};

use crate::{
    draw::{cell_h, cell_w},
    tower_at_tile, ClientGameState,
};

#[derive(Default)]
pub struct GameInput {
    pub mouse_in_world: bool,
    pub mouse_over_occupied_tile: bool,
}

pub fn mouse_position_vec() -> Vec2 {
    let pos = mouse_position();
    Vec2 { x: pos.0, y: pos.1 }
}

pub fn mouse_world_position() -> Vec2 {
    let pos = mouse_position_vec();
    Vec2 {
        x: pos.x / cell_w(),
        y: pos.y / cell_h(),
    }
}

pub fn pos_in_rect(pos: Vec2, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
    x0 <= pos.x && pos.x <= x1 && y0 <= pos.y && pos.y <= y1
}

pub fn main_input(state: &mut ClientGameState) {
    state.input.mouse_in_world = pos_in_rect(
        mouse_position_vec(),
        0.0,
        0.0,
        state.static_game_state.grid_w as f32,
        state.static_game_state.grid_h as f32,
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
