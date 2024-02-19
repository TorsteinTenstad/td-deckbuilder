use client_game_state::ClientGameState;
use common::component_attack::{Attack, AttackVariant};
use common::component_movement::get_detection_range;
use common::entity::EntityTag;
use common::entity_blueprint::DEFAULT_UNIT_DETECTION_RADIUS;
use common::network::ClientMessage;
use common::play_target::{unit_spawnpoint_target_transform, PlayFn};
use common::rect_transform::{point_inside, RectTransform};
use common::textures::SpriteId;
use common::world::find_entity;
use common::*;
use itertools::Itertools;
use macroquad::color::{Color, BLACK, BLUE, GRAY, PINK, RED, WHITE, YELLOW};
use macroquad::input::is_key_pressed;
use macroquad::math::Vec2;
use macroquad::miniquad::KeyCode;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line};
use macroquad::texture::{draw_texture_ex, DrawTextureParams};
use macroquad::window::{clear_background, screen_height, screen_width};
use macroquad::{window::next_frame, window::request_new_screen_size};
use physical_hand::hand_step;
pub mod config;
mod draw;
use draw::*;
mod input;
use input::*;
mod network;
use network::*;
mod client_game_state;
mod hit_numbers;
mod physical_card;
mod physical_hand;
mod text_box;
use text_box::*;
mod deck_builder;

#[macroquad::main("Client")]
async fn main() {
    request_new_screen_size(1280.0, 720.0);

    let mut state = ClientGameState::new().await;
    let mut text_box = TextBox::new(RectTransform {
        w: 200.0,
        h: 50.0,
        ..Default::default()
    });
    text_box.text = state.client_network_state.server_addr.to_string();

    loop {
        if state.in_deck_builder {
            state.step();
            state.deck_builder.step(state.dt);
            text_box.step();
            text_box.transform.x = screen_width() - text_box.transform.w - 10.0;
            text_box.transform.y = screen_height() - text_box.transform.h - 10.0;
            state.deck_builder.draw(&state.sprites);
            text_box.draw(Some(&state.font));

            next_frame().await;

            if is_key_pressed(KeyCode::Enter) {
                state.client_network_state.server_addr = text_box
                    .text
                    .parse()
                    .unwrap_or(state.client_network_state.server_addr);
                state.deck_builder.save();
                state.in_deck_builder = false;
            }
        } else {
            state
                .client_network_state
                .ensure_joined(ClientMessage::JoinGame(
                    state
                        .deck_builder
                        .deck
                        .iter()
                        .map(|physical_card| physical_card.card.clone())
                        .collect_vec(),
                ));
            while let Some(server_message) = state.client_network_state.receive() {
                state.update_server_controled_game_state_with_server_message(server_message);
            }
            main_step(&mut state);
            state.client_network_state.send_queued();
            main_draw(&state);

            next_frame().await;

            if is_key_pressed(KeyCode::Escape) {
                state.in_deck_builder = true;
            }
        }
    }
}

fn main_step(state: &mut ClientGameState) {
    state.step();
    main_input(state);
    hand_step(state);
    state.hit_numbers.step(
        &state
            .server_controlled_game_state
            .dynamic_game_state
            .entities,
        state.dt,
    );
}

fn main_draw(state: &ClientGameState) {
    if !state.has_player() {
        return;
    }

    // board
    clear_background(BLACK);
    draw_texture_ex(
        state.sprites.get_texture(&SpriteId::Map),
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

    //paths
    if state.show_debug_info {
        for building_location in state
            .server_controlled_game_state
            .semi_static_game_state
            .building_locations()
            .values()
        {
            draw_circle(
                to_screen_x(building_location.pos.x),
                to_screen_y(building_location.pos.y),
                to_screen_size(DEFAULT_UNIT_DETECTION_RADIUS),
                Color { a: 0.2, ..PINK },
            );
        }
        for (_, path) in state
            .server_controlled_game_state
            .static_game_state
            .paths
            .iter()
        {
            for ((x1, y1), (x2, y2)) in path.iter().tuple_windows() {
                let x1 = to_screen_x(*x1);
                let y1 = to_screen_y(*y1);
                let x2 = to_screen_x(*x2);
                let y2 = to_screen_y(*y2);
                draw_circle(x1, y1, 10.0, PINK);
                draw_circle(x2, y2, 10.0, PINK);
                draw_line(
                    x1,
                    y1,
                    x2,
                    y2,
                    5.0,
                    Color {
                        r: 0.843,
                        g: 0.803,
                        b: 0.627,
                        a: 1.0,
                    },
                );
            }
        }
    }

    // hand
    let mut physical_cards = state.physical_hand.cards.clone();
    physical_cards.sort_by(|a, b| {
        a.transform
            .w
            .partial_cmp(&b.transform.w)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for physical_card in physical_cards.iter() {
        draw_card(
            &physical_card.card_instance.card,
            &physical_card.transform,
            1.0,
            &state.sprites,
        )
    }

    // locations
    for (_id, loc) in state
        .server_controlled_game_state
        .semi_static_game_state
        .building_locations()
        .iter()
    {
        let x = to_screen_x(loc.pos.x);
        let y = to_screen_y(loc.pos.y);
        draw_circle(x, y, 20.0, WHITE);
    }

    // entities
    for entity in state
        .server_controlled_game_state
        .dynamic_game_state
        .entities
        .iter()
    {
        let Some(player) = state
            .server_controlled_game_state
            .dynamic_game_state
            .players
            .get(&entity.owner)
        else {
            continue;
        };
        let damage_animation_color = (entity.health.damage_animation > 0.0).then_some(RED);
        let pos_x = to_screen_x(entity.pos.x);
        let pos_y = to_screen_y(entity.pos.y);
        let radius = to_screen_size(entity.radius);

        match entity.tag {
            EntityTag::Tower | EntityTag::Base | EntityTag::Unit | EntityTag::FlyingUnit => {
                let texture = state
                    .sprites
                    .get_team_texture(&entity.sprite_id, Some(player.direction.clone()));

                let flip_x = entity
                    .movement
                    .as_ref()
                    .is_some_and(|movement| movement.movement_towards_target.velocity.x < 0.0);

                let height = 2.0 * radius;
                let width = height * texture.width() / texture.height();

                draw_texture_ex(
                    texture,
                    pos_x - radius,
                    pos_y - radius,
                    damage_animation_color.unwrap_or(WHITE),
                    DrawTextureParams {
                        dest_size: Some(Vec2 {
                            x: width,
                            y: height,
                        }),
                        flip_x,
                        ..Default::default()
                    },
                )
            }
            EntityTag::Bullet => {
                draw_circle(pos_x, pos_y, radius, GRAY);
            }
        }
    }

    // range_circle_preview
    let mut range_circle_preview: Vec<(f32, f32, f32, Color)> = Vec::new();
    if let Some(entity) = find_entity(
        &state
            .server_controlled_game_state
            .dynamic_game_state
            .entities,
        state.selected_entity_id,
    ) {
        if let Some(Attack { range, .. }) = entity
            .attacks
            .iter()
            .find(|attack| attack.variant == AttackVariant::RangedAttack)
        {
            range_circle_preview.push((
                entity.pos.x,
                entity.pos.y,
                range.to_f32(entity.radius),
                BLUE,
            ));
        }

        if let Some(detection_range) = get_detection_range(entity) {
            range_circle_preview.push((entity.pos.x, entity.pos.y, detection_range, YELLOW));
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
    if let Some(player) = state
        .server_controlled_game_state
        .dynamic_game_state
        .players
        .get(&state.player_id)
    {
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
            Some(&state.font),
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
                .server_controlled_game_state
                .dynamic_game_state
                .players
                .get(&state.player_id)
                .unwrap()
                .hand
                .energy,
            BLUE,
            WHITE,
            BLACK,
            Some(&state.font),
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
        for (_id, loc) in state
            .server_controlled_game_state
            .semi_static_game_state
            .building_locations()
            .iter()
        {
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
        let transform = &unit_spawnpoint_target_transform(
            target,
            &state.server_controlled_game_state.static_game_state,
        );
        let hovering = point_inside(mouse_world_position(), transform);
        draw_rect_transform(
            &to_screen_transform(transform),
            Color {
                a: if hovering { 0.8 } else { 0.5 },
                ..RED
            },
        );
    }

    // hit numbers
    state.hit_numbers.draw(Some(&state.font));
}
