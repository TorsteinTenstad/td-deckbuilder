use crate::ClientGameState;
use common::config::SCROLL_SENSITIVITY;
use macroquad::{
    input::{is_key_down, is_key_pressed, is_mouse_button_released, mouse_position, mouse_wheel},
    math::Vec2,
    miniquad::MouseButton,
};

#[derive(Default)]
pub struct GameInput {}

pub fn mouse_screen_pos_vec() -> Vec2 {
    let (x, y) = mouse_position();
    Vec2 { x, y }
}

pub fn main_input(state: &mut ClientGameState) {
    let (_, wheel_y) = mouse_wheel();
    state.view_state.normalized_scroll_y += wheel_y * SCROLL_SENSITIVITY;
    state.view_state.normalized_scroll_y = state.view_state.normalized_scroll_y.clamp(0.0, 1.0);
    if is_mouse_button_released(MouseButton::Left) {
        state.selected_entity_id = state
            .server_controlled_game_state
            .dynamic_game_state
            .entities
            .iter()
            .find_map(|entity_instance| {
                ((entity_instance.pos - mouse_screen_pos_vec()).length()
                    < entity_instance.entity.radius)
                    .then_some(entity_instance.id)
            });
    }

    if is_key_pressed(macroquad::miniquad::KeyCode::F3) {
        state.debug_draw_config.draw_paths = !state.debug_draw_config.draw_paths;
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
