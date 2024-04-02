use crate::level_config::get_prototype_level_config;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    input::mouse_position,
    math::{Rect, Vec2},
    window::{screen_height, screen_width},
};

pub struct ViewState {
    pub normalized_scroll_y: f32,
    pub ui_bar_width: f32,
    pub camera_cash: Option<Camera2D>,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            normalized_scroll_y: 0.0,
            ui_bar_width: 0.0,
            camera_cash: None,
        }
    }
}

pub fn get_level_rect() -> Rect {
    let level_width = get_prototype_level_config().level_width as f32;
    let level_height = get_prototype_level_config().level_height as f32;
    Rect::new(0.0, 0.0, level_width, level_height)
}

pub fn get_level_aspect() -> f32 {
    let level_width = get_prototype_level_config().level_width as f32;
    let level_height = get_prototype_level_config().level_height as f32;
    level_width / level_height
}

pub fn get_screen_aspect() -> f32 {
    screen_width() / screen_height()
}

impl ViewState {
    pub fn set_camera(world_space: Rect, display_space: Rect) -> Camera2D {
        let zoom = 2.0 * display_space.size() / world_space.size();
        let camera = Camera2D {
            target: world_space.point() + (1.0 - 2.0 * display_space.point()) / zoom,
            zoom,
            ..Default::default()
        };
        set_camera(&camera);
        camera
    }

    pub fn set_ui_overlay_camera(&mut self) {
        set_default_camera();
    }

    pub fn set_scrolling_level_camera(&mut self, display_space: Rect) {
        let level_width = get_prototype_level_config().level_width as f32;
        let level_height = get_prototype_level_config().level_height as f32;

        let draw_area_aspect =
            (screen_width() * display_space.w) / (screen_height() * display_space.h);

        let world_space_h = level_width / draw_area_aspect;
        let world_space_y = if world_space_h > level_height {
            -(world_space_h - level_height) / 2.0
        } else {
            -(1.0 - self.normalized_scroll_y) * (world_space_h - level_height)
        };
        let world_space = Rect::new(0.0, world_space_y, level_width, world_space_h);

        let camera = Self::set_camera(world_space, display_space);
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
