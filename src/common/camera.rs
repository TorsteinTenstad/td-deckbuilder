use crate::level_config::get_prototype_level_config;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    math::{Rect, Vec2},
    window::{screen_height, screen_width},
};

pub fn set_ui_overlay_camera() {
    set_default_camera()
}

pub fn set_gameplay_camera(top: f32) {
    let level_width = get_prototype_level_config().level_width as f32;
    let level_height = get_prototype_level_config().level_height as f32;

    let window_aspect = screen_width() / screen_height();

    let height = level_width / window_aspect;
    let top = if height > level_height {
        -(height - level_height) / 2.0
    } else {
        top
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
}
