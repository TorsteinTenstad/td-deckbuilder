use crate::{rect_transform::RectTransform, sprites::Sprites};
use crate::card::Card;
use macroquad::{
    color::{Color, WHITE},
    math::Vec2,
    shapes::{draw_rectangle, draw_rectangle_ex, DrawRectangleParams},
    text::{camera_font_scale, draw_text_ex, measure_text, Font, TextDimensions, TextParams},
    texture::{draw_texture_ex, DrawTextureParams},
};


#[allow(dead_code)]
pub enum TextOriginX {
    Left,
    Center,
    Right,
}

#[allow(dead_code)]
pub enum TextOriginY {
    Top,
    Center,
    AbsoluteCenter,
    Base,
    Bottom,
}

pub fn draw_rect_transform(transform: &RectTransform, color: Color) {
    draw_rectangle_ex(
        transform.x,
        transform.y,
        transform.w,
        transform.h,
        DrawRectangleParams {
            rotation: transform.rotation,
            offset: transform.offset,
            color,
        },
    );
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
    font: Option<&Font>,
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
            font,
        },
    )
}

pub fn draw_progress_bar(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    outline_w: f32,
    bar_progress: f32,
    count: i32,
    fill_color: Color,
    outline_color: Color,
    background_color: Color,
    font: Option<&Font>,
) {
    let inner_w = w - 2.0 * outline_w;
    let inner_h = h - 2.0 * outline_w;
    let filled_h = inner_h * bar_progress.fract();
    draw_rectangle(x, y, w, h, outline_color);
    draw_rectangle(
        x + outline_w,
        y + outline_w,
        inner_w,
        inner_h,
        background_color,
    );
    draw_rectangle(
        x + outline_w,
        y + outline_w + (inner_h - filled_h),
        inner_w,
        filled_h,
        fill_color,
    );
    draw_text_with_origin(
        format!("{}", count).as_str(),
        x + w / 2.0,
        y - outline_w,
        24.0,
        0.0,
        fill_color,
        TextOriginX::Center,
        TextOriginY::Bottom,
        font,
    )
}

pub fn draw_card(card: &Card, transform: &RectTransform, alpha: f32, sprites: &Sprites) {
    let texture = sprites.get_card_texture(card);
    #[rustfmt::skip]
    let offset_x 
        = transform.offset.x * transform.w * f32::cos(-transform.rotation)
        + transform.offset.y * transform.h * f32::sin(-transform.rotation);
    #[rustfmt::skip]
    let offset_y 
        = transform.offset.x * transform.w * f32::sin(-transform.rotation)
        + transform.offset.y * transform.h * f32::cos(-transform.rotation);
    draw_texture_ex(
        texture,
        transform.x - offset_x,
        transform.y - offset_y,
        Color { a: alpha, ..WHITE },
        DrawTextureParams {
            dest_size: Some(Vec2::new(transform.w, transform.h)),
            rotation: transform.rotation,
            ..Default::default()
        },
    )
}