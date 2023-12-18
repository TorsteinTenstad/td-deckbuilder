use common::{card::Card, play_target::UnitSpawnpointTarget, *};
use itertools::Itertools;
use macroquad::{
    color::{Color, BLACK, BLUE, GRAY, LIGHTGRAY, RED, WHITE, YELLOW},
    math::Vec2,
    shapes::{
        draw_circle, draw_circle_lines, draw_hexagon, draw_line, draw_rectangle, draw_rectangle_ex,
        DrawRectangleParams,
    },
    text::{camera_font_scale, draw_text_ex, measure_text, TextDimensions, TextParams},
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    window::{clear_background, screen_height, screen_width},
};
use std::collections::HashMap;

use crate::{input::mouse_position_vec, ClientGameState};

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

const GOLDEN_RATIO: f32 = 1.61803398875;

const PATH_COLOR: Color = Color {
    r: 0.843,
    g: 0.803,
    b: 0.627,
    a: 1.0,
};

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

pub fn main_draw(state: &ClientGameState) {
    // board
    clear_background(BLACK);

    draw_texture_ex(
        state.textures.get("concept").unwrap(),
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2 {
                x: screen_width(),
                y: screen_height(),
            }),
            ..Default::default()
        },
    );

    for (_, path) in state.static_game_state.paths.iter() {
        for ((x1, y1), (x2, y2)) in path.iter().tuple_windows() {
            let x1 = to_screen_x(*x1 as f32);
            let y1 = to_screen_y(*y1 as f32);
            let x2 = to_screen_x(*x2 as f32);
            let y2 = to_screen_y(*y2 as f32);
            draw_line(x1, y1, x2, y2, 5.0, PATH_COLOR);
        }
    }

    for (_id, loc) in state.dynamic_game_state.building_locations.iter() {
        let x = to_screen_x(loc.position.0 as f32);
        let y = to_screen_y(loc.position.1 as f32);
        draw_circle(x, y, 20.0, WHITE);
    }

    // entities
    for entity in state.dynamic_game_state.entities.values() {
        let player = state.dynamic_game_state.players.get(&entity.owner);
        let color = if entity.damage_animation > 0.0 {
            RED
        } else {
            player.map_or(WHITE, |player| player.color)
        };
        let pos_x = to_screen_x(entity.pos.x);
        let pos_y = to_screen_y(entity.pos.y);

        match entity.tag {
            EntityTag::Tower | EntityTag::Base => {
                draw_hexagon(pos_x, pos_y, 20.0, 0.0, false, color, color);
            }
            EntityTag::Unit => {
                draw_circle(pos_x, pos_y, entity.radius, color);
            }
            EntityTag::Bullet => {
                draw_circle(pos_x, pos_y, to_screen_size(PROJECTILE_RADIUS), GRAY);
            }
        }
    }

    // range_circle_preview
    let mut range_circle_preview: Option<(f32, f32, f32, Color)> = None;
    if let Some((x, y)) = state.preview_tower_pos {
        if state.input.mouse_in_world {
            let color = if state.input.mouse_over_occupied_tile {
                RED
            } else {
                BLUE
            };
            draw_hexagon(
                to_screen_x(x),
                to_screen_y(y),
                20.0,
                0.0,
                false,
                GRAY,
                Color { a: 0.5, ..color },
            );
            range_circle_preview = Some((x as f32, y as f32, 3.0, color));
        }
    } else if let Some(entity) = state
        .selected_entity_id
        .and_then(|id| state.dynamic_game_state.entities.get(&id))
    {
        if let Some(RangedAttack { range, .. }) = entity.ranged_attack {
            range_circle_preview = Some((entity.pos.x, entity.pos.y, range, BLUE));
        }
    }
    if let Some((x, y, range, color)) = range_circle_preview {
        let x = to_screen_x(x);
        let y = to_screen_y(y);
        let r = to_screen_size(range);

        draw_circle(x, y, r, Color { a: 0.2, ..color });
        draw_circle_lines(x, y, r, 2.0, color);
    }

    // Progress bars
    if let Some(player) = state.dynamic_game_state.players.get(&state.player_id) {
        let margin = 10.0;
        let outline_w = 5.0;
        let w = 25.0;
        let h = 100.0;
        let draw_progress = player.hand.card_draw_counter;
        draw_progress_bar(
            screen_width() - w - margin,
            screen_height() - h - margin,
            w,
            h,
            outline_w,
            draw_progress,
            state.physical_hand.cards.len() as i32,
            YELLOW,
            WHITE,
            BLACK,
        );
        let energy_progress = player.hand.energy_counter;
        draw_progress_bar(
            screen_width() - 2.0 * w - 2.0 * margin,
            screen_height() - h - margin,
            w,
            h,
            outline_w,
            energy_progress,
            state
                .dynamic_game_state
                .players
                .get(&state.player_id)
                .unwrap()
                .hand
                .energy,
            BLUE,
            WHITE,
            BLACK,
        );
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

const CARD_BORDER: f32 = 5.0;
const CARD_VISIBLE_HEIGHT: f32 = 0.8;

pub fn unit_spawnpoint_gui_indicator_transform(
    target: &UnitSpawnpointTarget,
    static_game_state: &StaticGameState,
) -> RectTransform {
    let UnitSpawnpointTarget {
        path_id,
        path_idx,
        direction: _,
    } = target;

    let Vec2 { x, y } = get_path_pos(&static_game_state, *path_id, *path_idx);
    RectTransform {
        x: to_screen_x(x),
        y: to_screen_y(y),
        w: 50.0,
        h: 50.0,
        offset: Vec2::splat(0.5),
        ..Default::default()
    }
}

pub fn curser_is_inside(transform: &RectTransform) -> bool {
    let local_mouse_pos = Vec2::from_angle(-transform.rotation).rotate(
        mouse_position_vec()
            - Vec2 {
                x: transform.x,
                y: transform.y,
            },
    ) + transform.offset
        * Vec2 {
            x: transform.w,
            y: transform.h,
        };

    local_mouse_pos.cmpgt(Vec2::ZERO).all()
        && local_mouse_pos
            .cmplt(Vec2::new(transform.w, transform.h))
            .all()
}

pub fn draw_card(
    card: &Card,
    transform: &RectTransform,
    alpha: f32,
    textures: &HashMap<String, Texture2D>,
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

    let icons: Vec<(&str, f32)> = Vec::new();

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
            .get(*texture_id)
            .expect(format!("Texture \"{}\" not found", texture_id).as_str());
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

#[derive(Debug, Default, Clone)]
pub struct RectTransform {
    pub w: f32,
    pub h: f32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub offset: Vec2,
}

impl RectTransform {
    pub fn animate_towards(&mut self, target: &RectTransform, snap: f32) {
        let self_weight = (0.5f32).powf(snap);
        let target_weight = 1.0 - self_weight;
        self.x = target_weight * target.x + self_weight * self.x;
        self.y = target_weight * target.y + self_weight * self.y;
        self.rotation = target_weight * target.rotation + self_weight * self.rotation;
        self.offset = target_weight * target.offset + self_weight * self.offset;
        self.w = target_weight * target.w + self_weight * self.w;
        self.h = target_weight * target.h + self_weight * self.h;
    }
}

pub fn card_transform(
    card_idx: usize,
    hand_size: usize,
    relative_splay_radius: f32,
    card_delta_angle: f32,
) -> RectTransform {
    let w = screen_width() / 12.0;
    let h = w * GOLDEN_RATIO;
    RectTransform {
        w,
        h,
        x: screen_width() / 2.0,
        y: screen_height() + (relative_splay_radius * h) - (CARD_VISIBLE_HEIGHT * h),
        rotation: (card_idx as f32 - ((hand_size - 1) as f32 / 2.0)) * card_delta_angle,
        offset: Vec2 {
            x: 0.5,
            y: relative_splay_radius,
        },
    }
}

pub fn out_of_hand_card_transform(x: f32, y: f32) -> RectTransform {
    let w = screen_width() / 10.0;
    RectTransform {
        w,
        h: w * GOLDEN_RATIO,
        x,
        y,
        rotation: 0.0,
        offset: 0.5 * Vec2::ONE,
    }
}

pub fn hovered_card_transform(
    card_idx: usize,
    hand_size: usize,
    relative_splay_radius: f32,
    card_delta_angle: f32,
) -> RectTransform {
    let w = screen_width() / 10.0;
    let h = w * GOLDEN_RATIO;
    let x = screen_width() / 2.0
        + ((relative_splay_radius * h) - (CARD_VISIBLE_HEIGHT * h))
            * f32::sin((card_idx as f32 - ((hand_size - 1) as f32 / 2.0)) * card_delta_angle);
    let y = screen_height();

    RectTransform {
        w,
        h: w * GOLDEN_RATIO,
        x,
        y,
        rotation: 0.0,
        offset: Vec2 { x: 0.5, y: 1.0 },
    }
}

pub async fn load_textures() -> HashMap<String, Texture2D> {
    let mut textures: HashMap<String, Texture2D> = HashMap::new();
    for texture_id in vec![
        "hourglass",
        "hourglass_bow",
        "hourglass_sword",
        "range",
        "shield",
        "sword",
        "bow",
        "concept",
    ] {
        textures.insert(
            texture_id.to_string(),
            load_texture(format!("assets/textures/{}.png", texture_id).as_str())
                .await
                .unwrap(),
        );
    }
    return textures;
}
