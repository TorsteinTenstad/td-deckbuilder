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

    pub fn set_gameplay_camera(&mut self, normalized_draw_area: Rect) {
        let level_width = get_prototype_level_config().level_width as f32;
        let level_height = get_prototype_level_config().level_height as f32;

        let screen_width = screen_width();
        let screen_height = screen_height();
        let draw_width = screen_width * normalized_draw_area.w;
        let draw_height = screen_height * normalized_draw_area.h;
        let draw_area_aspect = draw_width / draw_height;

        let height = level_width / draw_area_aspect;
        let top = if height > level_height {
            -(height - level_height) / 2.0
        } else {
            -self.normalized_scroll_y * level_height + level_height / 2.0
        };

        let camera_rect_w = level_width / normalized_draw_area.w;
        let camera_rect_h = height / normalized_draw_area.h;
        let camera_rect = Rect {
            x: -normalized_draw_area.x * camera_rect_w,
            y: top - normalized_draw_area.y * camera_rect_h,
            w: camera_rect_w,
            h: camera_rect_h,
        };

        let camera = Camera2D {
            target: Vec2::new(
                camera_rect.x + camera_rect.w / 2.0,
                camera_rect.y + camera_rect.h / 2.0,
            ),
            zoom: Vec2::new(1.0 / camera_rect.w * 2.0, 1.0 / camera_rect.h * 2.0),
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
