use crate::level_config::get_prototype_level_config;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    input::mouse_position,
    math::{Rect, Vec2},
    window::{screen_height, screen_width},
};

pub struct ViewState {
    pub normalized_scroll_y: f32,
    pub camera_cash: Option<Camera2D>,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            normalized_scroll_y: 0.5,
            camera_cash: None,
        }
    }
}

impl ViewState {
    pub fn set_ui_overlay_camera(&mut self) {
        set_default_camera();
    }

    pub fn set_gameplay_camera(&mut self) {
        let level_width = get_prototype_level_config().level_width as f32;
        let level_height = get_prototype_level_config().level_height as f32;

        let window_aspect = screen_width() / screen_height();

        let height = level_width / window_aspect;
        let top = if height > level_height {
            -(height - level_height) / 2.0
        } else {
            -self.normalized_scroll_y * level_height + level_height / 2.0
        };

        let level_size = Rect {
            x: 0.0,
            y: top,
            w: level_width,
            h: height,
        };

        let camera = Camera2D {
            target: Vec2::new(
                level_size.x + level_size.w / 2.0,
                level_size.y + level_size.h / 2.0,
            ),
            zoom: Vec2::new(1.0 / level_size.w * 2.0, 1.0 / level_size.h * 2.0),
            offset: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            render_target: None,
            viewport: None,
        };
        set_camera(&camera);
        self.camera_cash = Some(camera);
    }

    pub fn get_mouse_world_pos(&self) -> Vec2 {
        let (x, y) = mouse_position();
        match self.camera_cash.as_ref() {
            Some(camera) => camera.screen_to_world(Vec2::new(x, y)),
            None => {
                debug_assert!(false);
                Vec2::new(x, y)
            }
        }
    }
}
