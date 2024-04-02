use crate::debug_draw_config::DebugDrawConfig;
use crate::entity::{EntityInstance, EntityTag};
use crate::game_state::{
    DynamicGameState, SemiStaticGameState, ServerControlledGameState, StaticGameState,
};
use crate::sprites::Sprites;
use crate::world::{BuildingLocation, Zoning};
use itertools::Itertools;
use macroquad::color::{Color, GRAY, LIGHTGRAY, PINK, RED, WHITE};
use macroquad::math::Vec2;
use macroquad::shapes::{draw_circle, draw_line, draw_poly};
use macroquad::texture::{draw_texture_ex, DrawTextureParams};

pub fn draw_minimap(server_controlled_game_state: &ServerControlledGameState) {
    draw_minimap_entities(&server_controlled_game_state.dynamic_game_state, 75.0);
}

fn draw_minimap_entities(dynamic_game_state: &DynamicGameState, entity_scale: f32) {
    let get_color = |entity_instance: &EntityInstance| {
        let player = dynamic_game_state
            .players
            .get(&entity_instance.owner)
            .unwrap();
        let color = player.color;
        let damage_animation_color =
            (entity_instance.entity.health.damage_animation > 0.0).then_some(RED);
        damage_animation_color.unwrap_or(color)
    };
    fn draw_poly_with_border(x: f32, y: f32, sides: u8, radius: f32, rotation: f32, color: Color) {
        let relative_border_thickness: f32 = 0.4;
        let border_color: Color = WHITE;
        draw_poly(x, y, sides, radius, rotation, border_color);
        draw_poly(
            x,
            y,
            sides,
            radius * (1.0 - relative_border_thickness),
            rotation,
            color,
        );
    }
    for entity_instance in dynamic_game_state.entities.iter() {
        let color = get_color(entity_instance);
        match entity_instance.entity.tag {
            EntityTag::None => {
                debug_assert!(false);
            }
            EntityTag::Bullet => {}
            EntityTag::Tower => {
                draw_poly_with_border(
                    entity_instance.pos.x,
                    entity_instance.pos.y,
                    20,
                    entity_scale,
                    0.0,
                    color,
                );
            }
            EntityTag::Base => {
                draw_poly_with_border(
                    entity_instance.pos.x,
                    entity_instance.pos.y,
                    4,
                    entity_scale * 0.75,
                    45.0,
                    color,
                );
                draw_poly_with_border(
                    entity_instance.pos.x,
                    entity_instance.pos.y - entity_scale * 0.5,
                    3,
                    entity_scale,
                    30.0,
                    color,
                );
            }
            EntityTag::Unit | EntityTag::FlyingUnit => {
                draw_poly_with_border(
                    entity_instance.pos.x,
                    entity_instance.pos.y,
                    4,
                    entity_scale,
                    0.0,
                    color,
                );
            }
        };
    }
}

pub fn draw_server_controlled_game_state(
    server_controlled_game_state: &ServerControlledGameState,
    sprites: &Sprites,
    debug_draw_config: &DebugDrawConfig,
) {
    if debug_draw_config.draw_paths {
        draw_path_lines(&server_controlled_game_state.static_game_state, 5.0);
        draw_path_nodes(&server_controlled_game_state.static_game_state);
    }
    draw_building_locations(&server_controlled_game_state.semi_static_game_state);
    draw_entities(&server_controlled_game_state.dynamic_game_state, sprites);
}

fn draw_building_locations(semi_static_game_state: &SemiStaticGameState) {
    for (_id, BuildingLocation { pos, zoning, .. }) in
        semi_static_game_state.building_locations().iter()
    {
        let (poly_sides, color, radius) = match zoning {
            Zoning::Normal => (20, LIGHTGRAY, 16.0),
            Zoning::Commerce => (6, WHITE, 20.0),
        };
        draw_poly(pos.x, pos.y, poly_sides, radius, 0., color);
    }
}

fn draw_entities(dynamic_game_state: &DynamicGameState, sprites: &Sprites) {
    for entity_instance in dynamic_game_state.entities.iter() {
        let Some(player) = dynamic_game_state.players.get(&entity_instance.owner) else {
            continue;
        };
        let damage_animation_color =
            (entity_instance.entity.health.damage_animation > 0.0).then_some(RED);

        match entity_instance.entity.tag {
            EntityTag::None => {
                debug_assert!(false);
            }
            EntityTag::Tower | EntityTag::Base | EntityTag::Unit | EntityTag::FlyingUnit => {
                let texture = sprites.get_team_texture(
                    &entity_instance.entity.sprite_id,
                    Some(player.direction.clone()),
                );

                let flip_x = entity_instance
                    .entity
                    .movement
                    .as_ref()
                    .is_some_and(|movement| movement.movement_towards_target.velocity.x < 0.0);

                let height = 2.0 * entity_instance.entity.radius;
                let width = height * texture.width() / texture.height();

                draw_texture_ex(
                    texture,
                    entity_instance.pos.x - entity_instance.entity.radius,
                    entity_instance.pos.y - entity_instance.entity.radius,
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
                draw_circle(
                    entity_instance.pos.x,
                    entity_instance.pos.y,
                    entity_instance.entity.radius,
                    GRAY,
                );
            }
        }
    }
}

fn draw_path_lines(static_game_state: &StaticGameState, line_width: f32) {
    for (_, path) in static_game_state.paths.iter() {
        for ((x1, y1), (x2, y2)) in path.iter().tuple_windows() {
            draw_line(
                *x1,
                *y1,
                *x2,
                *y2,
                line_width,
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
fn draw_path_nodes(static_game_state: &StaticGameState) {
    for (_, path) in static_game_state.paths.iter() {
        for ((x1, y1), (x2, y2)) in path.iter().tuple_windows() {
            draw_circle(*x1, *y1, 10.0, PINK);
            draw_circle(*x2, *y2, 10.0, PINK);
        }
    }
}
