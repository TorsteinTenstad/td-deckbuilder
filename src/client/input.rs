use crate::{
    draw::{to_world_x, to_world_y},
    ClientGameState,
};
use macroquad::{
    input::{is_key_down, is_key_pressed, is_mouse_button_released, mouse_position},
    math::Vec2,
    miniquad::MouseButton,
};

#[derive(Default)]
pub struct GameInput {}

pub fn mouse_screen_position() -> Vec2 {
    let (x, y) = mouse_position();
    Vec2 { x, y }
}

pub fn mouse_world_position() -> Vec2 {
    let Vec2 { x, y } = mouse_screen_position();
    Vec2 {
        x: to_world_x(x),
        y: to_world_y(y),
    }
}

pub fn main_input(state: &mut ClientGameState) {
    if is_mouse_button_released(MouseButton::Left) {
        state.selected_entity_id = state.dynamic_game_state.entities.iter().find_map(|entity| {
            ((entity.pos - mouse_world_position()).length() < entity.radius).then(|| entity.id)
        });
    }

    if is_key_pressed(macroquad::miniquad::KeyCode::F3) {
        state.show_debug_info = !state.show_debug_info;
    }

    // TODO, Magne: this is temp
    // Card drawing parameter adjustment
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
