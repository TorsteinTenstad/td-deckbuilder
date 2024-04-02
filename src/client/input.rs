use crate::ClientGameState;
use common::config::SCROLL_SENSITIVITY;
use macroquad::{
    input::{
        is_key_down, is_key_pressed, is_mouse_button_released, mouse_position, mouse_wheel, KeyCode,
    },
    math::Vec2,
    miniquad::MouseButton,
};

#[derive(Debug, Default)]
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
                ((entity_instance.pos - state.view_state.get_mouse_world_pos()).length()
                    < entity_instance.entity.radius)
                    .then_some(entity_instance.id)
            });
    }
    if is_key_pressed(KeyCode::M) {
        if state.view_state.ui_bar_width == 0.0 {
            state.view_state.ui_bar_width = 0.2;
        } else {
            state.view_state.ui_bar_width = 0.0;
        }
    }

    if is_key_pressed(KeyCode::F3) {
        state.debug_draw_config.draw_paths = !state.debug_draw_config.draw_paths;
    }

    // TODO, Magne: this is temp
    // Card drawing parameter adjustment
    {
        if is_key_down(KeyCode::L) {
            state.card_delta_angle += 0.05 * state.dt;
            dbg!(state.card_delta_angle);
        }
        if is_key_down(KeyCode::J) {
            state.card_delta_angle -= 0.05 * state.dt;
            dbg!(state.card_delta_angle);
        }
        if is_key_down(KeyCode::I) {
            state.relative_splay_radius += 0.5 * state.dt;
            dbg!(state.relative_splay_radius);
        }
        if is_key_down(KeyCode::K) {
            state.relative_splay_radius -= 0.5 * state.dt;
            dbg!(state.relative_splay_radius);
        }
    }
}
