use common::{card::Card, *};
use macroquad::{
    color::{Color, BLACK, BLUE, GRAY, LIGHTGRAY, RED, WHITE, YELLOW},
    math::Vec2,
    shapes::{
        draw_circle, draw_circle_lines, draw_hexagon, draw_poly, draw_rectangle, draw_rectangle_ex,
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

const GRASS_COLOR: Color = Color {
    r: 0.686,
    g: 0.784,
    b: 0.490,
    a: 1.0,
};

const PATH_COLOR: Color = Color {
    r: 0.843,
    g: 0.803,
    b: 0.627,
    a: 1.0,
};

// TODO overloading?
pub fn cell_w() -> f32 {
    screen_width() / 16.0 // state.static_game_state.grid_w as f32; TODO
}
pub fn cell_h() -> f32 {
    (7.0 / 9.0) * screen_height() / 7.0 // state.static_game_state.grid_h as f32; TODO
}
pub fn u32_to_screen_x(x: u32) -> f32 {
    (x as f32) * cell_w()
}
pub fn u32_to_screen_y(y: u32) -> f32 {
    (y as f32) * cell_h()
}
pub fn f32_to_screen_x(x: f32) -> f32 {
    (x as f32) * cell_w()
}
pub fn f32_to_screen_y(y: f32) -> f32 {
    (y as f32) * cell_h()
}
pub fn to_screen_size(x: f32) -> f32 {
    x * cell_w()
}

pub fn main_draw(state: &ClientGameState) {
    // board
    clear_background(BLACK);
    for x in 0..state.static_game_state.grid_w {
        for y in 0..state.static_game_state.grid_h {
            draw_rectangle_ex(
                u32_to_screen_x(x),
                u32_to_screen_y(y),
                cell_w(),
                cell_h(),
                DrawRectangleParams {
                    color: if state.static_game_state.path.contains(&(x as i32, y as i32)) {
                        PATH_COLOR
                    } else {
                        GRASS_COLOR
                    },
                    ..Default::default()
                },
            );
        }
    }

    // entities
    for entity in state.dynamic_game_state.entities.values() {
        let player = state.dynamic_game_state.players.get(&entity.owner);
        let color = if entity.damage_animation > 0.0 {
            RED
        } else {
            player.map_or(WHITE, |player| player.color)
        };
        let pos_x = f32_to_screen_x(entity.pos.x);
        let pos_y = f32_to_screen_y(entity.pos.y);
        let r = to_screen_size(entity.radius);

        match entity.tag {
            EntityTag::Tower => {
                draw_hexagon(pos_x, pos_y, 20.0, 0.0, false, color, color);
            }
            EntityTag::Unit => {
                draw_circle(pos_x, pos_y, r, color);
            }
            EntityTag::Drone => {
                let rotation = if let Behavior::Drone(DroneBehavior {
                    target_entity_id, ..
                }) = entity.behavior
                {
                    if let Some(target_entity) =
                        target_entity_id.and_then(|id| state.dynamic_game_state.entities.get(&id))
                    {
                        (target_entity.pos - entity.pos).angle_between(Vec2::NEG_X)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                draw_poly(pos_x, pos_y, 3, r, 360.0 * rotation, color);
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
                f32_to_screen_x(x),
                f32_to_screen_y(y),
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
        let x = f32_to_screen_x(x);
        let y = f32_to_screen_y(y);
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
        let draw_progress = player.card_draw_counter;
        draw_progress_bar(
            screen_width() - w - margin,
            screen_height() - h - margin,
            w,
            h,
            outline_w,
            draw_progress,
            state.hand.hand.len() as i32,
            YELLOW,
            WHITE,
            BLACK,
        );
        let energy_progress = player.energy_counter;
        draw_progress_bar(
            screen_width() - 2.0 * w - 2.0 * margin,
            screen_height() - h - margin,
            w,
            h,
            outline_w,
            energy_progress,
            state.hand.energy,
            BLUE,
            WHITE,
            BLACK,
        );
    }
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

pub fn card_is_hovering(transform: &RectTransform) -> bool {
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
    draw_rectangle_ex(
        transform.x,
        transform.y,
        transform.w,
        transform.h,
        DrawRectangleParams {
            color: Color { a: alpha, ..GRAY },
            rotation: transform.rotation,
            offset: transform.offset,
            ..Default::default()
        },
    );
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
        BLACK,
        TextOriginX::Right,
        TextOriginY::Top,
    );

    let width_relative_margin = 0.1;
    let energy_indicator_pos = get_on_card_pos(
        width_relative_margin,
        width_relative_margin * transform.w / transform.h,
    );

    let mut icons: Vec<(&str, f32)> = Vec::new();
    let entity = card.to_entity(
        0,
        &ServerPlayer::new(Direction::Positive, Vec2::ZERO, BLACK),
        0.0,
        0.0,
    );

    icons.push(("shield", entity.health));
    if let Some(RangedAttack {
        range,
        damage,
        fire_interval,
        ..
    }) = entity.ranged_attack
    {
        icons.push(("bow", damage));
        icons.push(("range", range));
        icons.push(("hourglass_bow", fire_interval));
    }
    if let Some(MeleeAttack {
        damage,
        attack_interval,
        ..
    }) = entity.melee_attack
    {
        icons.push(("sword", damage));
        icons.push(("hourglass_sword", attack_interval));
    };

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
            WHITE,
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
            BLACK,
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

#[derive(Default)]
pub struct RectTransform {
    w: f32,
    h: f32,
    x: f32,
    y: f32,
    rotation: f32,
    offset: Vec2,
}

pub fn card_transform(
    i: usize,
    n: usize,
    relative_splay_radius: f32,
    card_delta_angle: f32,
) -> RectTransform {
    let w = screen_width() / 12.0;
    let h = w * GOLDEN_RATIO;
    return RectTransform {
        w,
        h,
        x: screen_width() / 2.0,
        y: screen_height() + (relative_splay_radius * h) - (CARD_VISIBLE_HEIGHT * h),
        rotation: (i as f32 - ((n - 1) as f32 / 2.0)) * card_delta_angle,
        offset: Vec2 {
            x: 0.5,
            y: relative_splay_radius,
        },
    };
}

pub fn draw_out_of_hand_card(card: &Card, x: f32, y: f32, textures: &HashMap<String, Texture2D>) {
    let w = screen_width() / 10.0;
    let transform = RectTransform {
        w,
        h: w * GOLDEN_RATIO,
        x,
        y,
        rotation: 0.0,
        offset: 0.5 * Vec2::ONE,
    };
    draw_card(card, &transform, 1.0, textures)
}

pub fn draw_highlighted_card(
    card: &Card,
    i: usize,
    relative_splay_radius: f32,
    card_delta_angle: f32,
    textures: &HashMap<String, Texture2D>,
    hand_size: usize,
) {
    let w = screen_width() / 10.0;
    let h = w * GOLDEN_RATIO;
    let x = screen_width() / 2.0
        + ((relative_splay_radius * h) - (CARD_VISIBLE_HEIGHT * h))
            * f32::sin((i as f32 - ((hand_size - 1) as f32 / 2.0)) * card_delta_angle);
    let y = screen_height();

    let transform = RectTransform {
        w,
        h: w * GOLDEN_RATIO,
        x,
        y,
        rotation: 0.0,
        offset: Vec2 { x: 0.5, y: 1.0 },
    };

    draw_card(card, &transform, 1.0, textures);
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
