use client_game_state::ClientGameState;
use common::component_attack::{Attack, AttackVariant};
use common::entity::EntityTag;
use common::play_target::{unit_spawnpoint_target_transform, PlayFn};
use common::rect_transform::point_inside;
use common::*;
use macroquad::color::{Color, BLACK, BLUE, GRAY, RED, WHITE, YELLOW};
use macroquad::math::Vec2;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_hexagon};
use macroquad::texture::{draw_texture_ex, DrawTextureParams};
use macroquad::window::{clear_background, screen_height, screen_width};
use macroquad::{window::next_frame, window::request_new_screen_size};
use physical_hand::{hand_step, PhysicalHand};
pub mod config;
mod draw;
use draw::*;
mod input;
use input::*;
mod network;
use network::*;
mod client_game_state;
mod physical_card;
mod physical_hand;

#[macroquad::main("Client")]
async fn main() {
    request_new_screen_size(1280.0, 720.0);

    let mut state = ClientGameState::new().await;

    loop {
        udp_update_game_state(&mut state);
        main_step(&mut state);
        udp_send_commands(&mut state);
        main_draw(&state);

        next_frame().await;
    }
}

fn main_step(state: &mut ClientGameState) {
    state.step();
    main_input(state);
    hand_step(state)
}

fn main_draw(state: &ClientGameState) {
    // for (_, path) in state.static_game_state.paths.iter() {
    //     for ((x1, y1), (x2, y2)) in path.iter().tuple_windows() {
    //         let x1 = to_screen_x(*x1 as f32);
    //         let y1 = to_screen_y(*y1 as f32);
    //         let x2 = to_screen_x(*x2 as f32);
    //         let y2 = to_screen_y(*y2 as f32);
    //         draw_line(x1, y1, x2, y2, 5.0,
    //            Color {
    //               r: 0.843,
    //               g: 0.803,
    //               b: 0.627,
    //               a: 1.0,
    //               },
    //            );
    //     }
    // }

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

    // hand
    for physical_card in state.physical_hand.cards.iter() {
        draw_card(
            &physical_card.card_instance.card,
            &physical_card.transform,
            1.0,
            &state.textures,
        )
    }

    // locations
    for (_id, loc) in state.dynamic_game_state.building_locations.iter() {
        let x = to_screen_x(loc.pos.x as f32);
        let y = to_screen_y(loc.pos.y as f32);
        draw_circle(x, y, 20.0, WHITE);
    }

    // entities
    for entity in state.dynamic_game_state.entities.iter() {
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
                draw_hexagon(
                    pos_x,
                    pos_y,
                    to_screen_size(entity.radius),
                    0.0,
                    false,
                    color,
                    color,
                );
            }
            EntityTag::Unit => {
                draw_circle(pos_x, pos_y, to_screen_size(entity.radius), color);
            }
            EntityTag::Bullet => {
                draw_circle(pos_x, pos_y, to_screen_size(entity.radius), GRAY);
            }
        }
    }

    // range_circle_preview
    let mut range_circle_preview: Vec<(f32, f32, f32, Color)> = Vec::new();
    if let Some(entity) = state.selected_entity_id.and_then(|id| {
        state
            .dynamic_game_state
            .entities
            .iter()
            .find(|entity| entity.id == id)
    }) {
        if let Some(Attack { range, .. }) = entity
            .attacks
            .iter()
            .find(|attack| attack.variant == AttackVariant::RangedAttack)
        {
            range_circle_preview.push((entity.pos.x, entity.pos.y, *range, BLUE));
        }
        range_circle_preview.push((entity.pos.x, entity.pos.y, entity.hitbox_radius, RED));
    }
    for (x, y, range, color) in range_circle_preview {
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

    // hover building location
    if state
        .physical_hand
        .card_idx_being_held
        .filter(|idx| {
            matches!(
                state.physical_hand.cards[*idx]
                    .card_instance
                    .card
                    .get_card_data()
                    .play_fn,
                PlayFn::BuildingSpot(_)
            )
        })
        .is_some()
    {
        for (_id, loc) in state.dynamic_game_state.building_locations.iter() {
            let x = to_screen_x(loc.pos.x);
            let y = to_screen_y(loc.pos.y);
            let r = 20.0;
            let hovering = (mouse_screen_position() - Vec2 { x, y }).length() < r;
            draw_circle_lines(
                x,
                y,
                r,
                3.0,
                Color {
                    a: if hovering { 0.8 } else { 0.5 },
                    ..RED
                },
            );
        }
    }

    // spawnpoint indicators
    for target in state.unit_spawnpoint_targets.iter() {
        let transform = &unit_spawnpoint_target_transform(target, &state.static_game_state);
        let hovering = point_inside(mouse_world_position(), transform);
        draw_rect_transform(
            &to_screen_transform(transform),
            Color {
                a: if hovering { 0.8 } else { 0.5 },
                ..RED
            },
        );
    }
}
