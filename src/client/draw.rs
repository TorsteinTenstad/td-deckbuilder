use crate::{physical_card::CARD_BORDER, rect_transform::RectTransform};
use common::{card::Card, textures::SpriteId, *};
use macroquad::{
    color::{Color, BLACK, BLUE, GRAY, LIGHTGRAY, WHITE, YELLOW},
    math::Vec2,
    shapes::{draw_circle, draw_rectangle, draw_rectangle_ex, DrawRectangleParams},
    text::{camera_font_scale, draw_text_ex, measure_text, TextDimensions, TextParams},
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    window::{screen_height, screen_width},
};
use std::collections::HashMap;

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

pub const GOLDEN_RATIO: f32 = 1.61803398875;

pub fn to_screen_x<T>(x: T) -> f32
where
    T: Into<f32>,
{
    x.into() * screen_width() / level_config::LEVEL_WIDTH as f32
}
pub fn to_screen_y<T>(y: T) -> f32
where
    T: Into<f32>,
{
    y.into() * screen_height() / level_config::LEVEL_HEIGHT as f32
}

pub fn to_world_x<T>(x: T) -> f32
where
    T: Into<f32>,
{
    x.into() * level_config::LEVEL_WIDTH as f32 / screen_width()
}
pub fn to_world_y<T>(y: T) -> f32
where
    T: Into<f32>,
{
    y.into() * level_config::LEVEL_HEIGHT as f32 / screen_height()
}

pub fn to_screen_size<T>(size: T) -> f32
where
    T: Into<f32>,
{
    size.into() * screen_width() / level_config::LEVEL_WIDTH as f32
}

pub fn to_screen_transform(transform: &RectTransform) -> RectTransform {
    RectTransform {
        x: to_screen_x(transform.x),
        y: to_screen_y(transform.y),
        w: to_screen_size(transform.w),
        h: to_screen_size(transform.h),
        rotation: transform.rotation,
        offset: transform.offset,
    }
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
    )
}

pub fn draw_card(
    card: &Card,
    transform: &RectTransform,
    alpha: f32,
    textures: &HashMap<SpriteId, Texture2D>,
) {
    draw_circle(transform.x, transform.y, 3.0, YELLOW);
    draw_rect_transform(transform, Color { a: alpha, ..GRAY });
    let inner_offset = transform.offset
        + Vec2 {
            x: 0.0,
            y: (2.0 * transform.offset.y - 1.0) * CARD_BORDER / transform.h,
        };

    draw_rectangle_ex(
        transform.x,
        transform.y,
        transform.w - 2.0 * CARD_BORDER,
        transform.h - 2.0 * CARD_BORDER,
        DrawRectangleParams {
            color: Color {
                a: alpha,
                ..LIGHTGRAY
            },
            rotation: transform.rotation,
            offset: inner_offset,
            ..Default::default()
        },
    );

    let get_on_card_pos = |rel_x, rel_y| -> Vec2 {
        Vec2 {
            x: transform.x,
            y: transform.y,
        } + Vec2::from_angle(transform.rotation).rotate(
            Vec2 {
                x: transform.w,
                y: transform.h,
            } * (Vec2 { x: rel_x, y: rel_y } - transform.offset),
        )
    };

    let card_name_pos = get_on_card_pos(0.9, 0.1);
    draw_text_with_origin(
        card.name(),
        card_name_pos.x,
        card_name_pos.y,
        20.0,
        transform.rotation,
        Color { a: alpha, ..BLACK },
        TextOriginX::Right,
        TextOriginY::Top,
    );

    let width_relative_margin = 0.1;
    let energy_indicator_pos = get_on_card_pos(
        width_relative_margin,
        width_relative_margin * transform.w / transform.h,
    );

    let icons: Vec<(SpriteId, f32)> = Vec::new();

    for (i, (texture_id, value)) in icons.iter().filter(|(_, value)| *value > 0.001).enumerate() {
        let width_relative_icon_size = 0.2;
        let on_card_icon_pos = get_on_card_pos(
            width_relative_margin,
            2.0 * width_relative_margin + i as f32 * (width_relative_icon_size),
        );
        let on_card_value_pos = get_on_card_pos(
            2.0 * width_relative_margin + width_relative_icon_size,
            2.0 * width_relative_margin + (i as f32 + 0.25) * (width_relative_icon_size),
        );
        let icon_size = Vec2::splat(transform.w * width_relative_icon_size);
        let texture = textures
            .get(texture_id)
            .expect(format!("Texture \"{}\" not found", sprite_id_to_string(*texture_id)).as_str());
        draw_texture_ex(
            texture,
            on_card_icon_pos.x,
            on_card_icon_pos.y,
            Color { a: alpha, ..WHITE },
            DrawTextureParams {
                rotation: transform.rotation,
                dest_size: Some(icon_size),
                pivot: Some(on_card_icon_pos + icon_size / 2.0),
                ..Default::default()
            },
        );
        draw_text_with_origin(
            format!("{}", value).as_str(),
            on_card_value_pos.x,
            on_card_value_pos.y,
            26.0,
            transform.rotation,
            Color { a: alpha, ..BLACK },
            TextOriginX::Left,
            TextOriginY::AbsoluteCenter,
        );
    }

    draw_circle(
        energy_indicator_pos.x,
        energy_indicator_pos.y,
        transform.w / 8.0,
        BLUE,
    );
    draw_text_with_origin(
        format!("{}", card.energy_cost()).as_str(),
        energy_indicator_pos.x,
        energy_indicator_pos.y,
        24.0,
        transform.rotation,
        WHITE,
        TextOriginX::Center,
        TextOriginY::AbsoluteCenter,
    );
}

pub fn sprite_id_to_string(texture_id: SpriteId) -> &'static str {
    match texture_id {
        SpriteId::Bow => "bow",
        SpriteId::Concept => "concept",
        SpriteId::Hourglass => "hourglass",
        SpriteId::HourglassBow => "hourglass_bow",
        SpriteId::HourglassSword => "hourglass_sword",
        SpriteId::Range => "range",
        SpriteId::Shield => "shield",
        SpriteId::UnitArcher => "unit_archer",
        SpriteId::UnitSoldier => "unit_soldier",
        SpriteId::Sword => "sword",
        SpriteId::X => "x",
    }
}

pub async fn load_sprites() -> HashMap<SpriteId, Texture2D> {
    let mut sprites: HashMap<SpriteId, Texture2D> = HashMap::new();
    for sprite_id in vec![
        SpriteId::Bow,
        SpriteId::Concept,
        SpriteId::Hourglass,
        SpriteId::HourglassBow,
        SpriteId::HourglassSword,
        SpriteId::Range,
        SpriteId::Shield,
        SpriteId::UnitArcher,
        SpriteId::UnitSoldier,
        SpriteId::Sword,
        SpriteId::X,
    ] {
        sprites.insert(
            sprite_id.clone(),
            load_texture(
                format!("assets/textures/{}.png", sprite_id_to_string(sprite_id)).as_str(),
            )
            .await
            .unwrap(),
        );
    }
    return sprites;
}
