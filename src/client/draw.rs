use crate::rect_transform::RectTransform;
use common::{card::Card, textures::SpriteId, world::Direction, *};
use macroquad::{
    color::{Color, WHITE},
    math::Vec2,
    miniquad::FilterMode,
    shapes::{draw_rectangle, draw_rectangle_ex, DrawRectangleParams},
    text::{camera_font_scale, draw_text_ex, measure_text, Font, TextDimensions, TextParams},
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    window::{screen_height, screen_width},
};
use std::collections::HashMap;
use strum::IntoEnumIterator;

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

#[derive(Default)]
pub struct Sprites {
    sprites: HashMap<SpriteId, Texture2D>,
    sprites_red: HashMap<SpriteId, Texture2D>,
    sprites_blue: HashMap<SpriteId, Texture2D>,
    card_textures: HashMap<Card, Texture2D>,
}

impl Sprites {
    pub async fn load() -> Sprites {
        let mut sprites = Sprites::default();

        for sprite_id in SpriteId::iter() {
            if let Ok(texture) =
                load_texture(format!("assets/textures/{}", sprite_id.to_path()).as_str()).await
            {
                sprites.sprites.insert(sprite_id.clone(), texture);
            }
        }
        for (color, sprites) in [
            ("red", &mut sprites.sprites_red),
            ("blue", &mut sprites.sprites_blue),
        ] {
            for sprite_id in SpriteId::iter() {
                if let Ok(texture) = load_texture(
                    format!("assets/textures/{}/{}", color, sprite_id.to_path()).as_str(),
                )
                .await
                {
                    sprites.insert(sprite_id.clone(), texture);
                }
            }
        }

        for card in Card::iter() {
            if let Ok(texture) = load_texture(card.get_texture_path().as_str()).await {
                sprites.card_textures.insert(card, texture);
            }
        }

        sprites
            .sprites
            .entry(SpriteId::Empty)
            .or_insert_with(Texture2D::empty);

        sprites
    }
    pub fn get_texture(&self, sprite_id: &SpriteId) -> &Texture2D {
        self.get_team_texture(sprite_id, None)
    }
    pub fn get_team_texture(&self, sprite_id: &SpriteId, team: Option<Direction>) -> &Texture2D {
        if let Some(sprite) = self.sprites.get(sprite_id) {
            return sprite;
        }
        match team {
            Some(Direction::Positive) => self.sprites_red.get(sprite_id),
            Some(Direction::Negative) => self.sprites_blue.get(sprite_id),
            _ => None,
        }
        .unwrap_or(self.sprites.get(&SpriteId::Empty).unwrap())
    }

    fn get_card_texture(&self, card: &Card) -> &Texture2D {
        self.card_textures.get(card).unwrap()
    }
}
