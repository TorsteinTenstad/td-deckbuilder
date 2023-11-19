use macroquad::{
    color::Color,
    math::Vec2,
    text::{camera_font_scale, draw_text_ex, measure_text, TextDimensions, TextParams},
};

pub enum TextOriginX {
    Left,
    Center,
    Right,
}

pub enum TextOriginY {
    Top,
    Center,
    AbsoluteCenter,
    Base,
    Bottom,
}

pub fn draw_text_with_origin(
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    rotation: f32,
    color: Color,
    origin_x: TextOriginX,
    origin_y: TextOriginY,
) {
    let (font_size, font_scale, font_scale_aspect) = camera_font_scale(font_size);
    let TextDimensions {
        width,
        height,
        offset_y,
    } = measure_text(text, None, font_size, font_scale);
    let origin_correction = Vec2::from_angle(rotation).rotate(Vec2 {
        x: match origin_x {
            TextOriginX::Left => 0.0,
            TextOriginX::Center => -width / 2.0,
            TextOriginX::Right => -width,
        },
        y: match origin_y {
            TextOriginY::Top => offset_y,
            TextOriginY::Center => offset_y / 2.0,
            TextOriginY::AbsoluteCenter => offset_y - height / 2.0,
            TextOriginY::Base => 0.0,
            TextOriginY::Bottom => offset_y - height,
        },
    });
    draw_text_ex(
        text,
        x + origin_correction.x,
        y + origin_correction.y,
        TextParams {
            font_size,
            font_scale,
            font_scale_aspect,
            rotation,
            color,
            ..Default::default()
        },
    )
}
