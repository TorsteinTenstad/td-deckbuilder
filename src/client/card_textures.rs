use std::collections::HashMap;

use common::card::{Card, CardData};
use macroquad::{
    camera::{self, set_camera, set_default_camera, Camera2D},
    color::{BLACK, PINK, WHITE},
    math::Vec2,
    texture::{draw_texture_ex, load_texture, render_target, DrawTextureParams, Texture2D},
    window::clear_background,
};

use crate::{draw_text_with_origin, Sprites, GOLDEN_RATIO};

const CARD_IMAGE_WIDTH: u32 = 1024;
const CARD_IMAGE_HEIGHT: u32 = ((CARD_IMAGE_WIDTH as f32) * GOLDEN_RATIO) as u32;

pub struct CardTextures {
    textures: HashMap<String, Texture2D>,
}

fn card_data_to_image_path(card_data: &CardData) -> String {
    let attack_string = match card_data.attack {
        Some(attack) => attack.to_string(),
        None => String::from("_"),
    };
    let health_string = match card_data.health {
        Some(health) => health.to_string(),
        None => String::from("_"),
    };
    format!(
        "assets/cards/{}-{}-{}-{}-{}-{}-{}.png",
        card_data.name,
        card_data.energy_cost,
        attack_string,
        health_string,
        card_data.description,
        CARD_IMAGE_WIDTH,
        CARD_IMAGE_HEIGHT
    )
}

impl CardTextures {
    const FORCE_REGENERATION: bool = true;

    pub async fn load(sprites: &Sprites) -> Self {
        let mut inst = Self {
            textures: HashMap::new(),
        };
        for card in Card::iter() {
            let card_data = card.get_card_data();
            let image_path = card_data_to_image_path(card_data);
            if !Self::FORCE_REGENERATION {
                if let Ok(texture) = load_texture(&image_path).await {
                    inst.textures.insert(image_path, texture);
                    continue;
                }
            }
            let render_target = render_target(CARD_IMAGE_WIDTH, CARD_IMAGE_HEIGHT);
            let camera = Camera2D {
                render_target: Some(render_target),
                ..Default::default()
            };
            set_camera(&camera);
            clear_background(BLACK);

            let card_art_texture = sprites.get_texture(&card_data.sprite_id);
            let card_art_aspect_ratio =
                card_art_texture.height() as f32 / card_art_texture.width() as f32;
            draw_texture_ex(
                card_art_texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(
                        CARD_IMAGE_WIDTH as f32,
                        CARD_IMAGE_WIDTH as f32 * card_art_aspect_ratio,
                    )),
                    ..Default::default()
                },
            );
            draw_text_with_origin(
                card_data.name,
                CARD_IMAGE_WIDTH as f32 / 2.0,
                50.0,
                20.0,
                0.0,
                PINK,
                crate::TextOriginX::Center,
                crate::TextOriginY::Center,
                None,
            );

            let texture = camera.render_target.unwrap().texture;
            texture.get_texture_data().export_png(image_path.as_str());
            inst.textures.insert(image_path.clone(), texture);
        }
        set_default_camera();
        inst
    }

    pub fn get_card_texture(&mut self, card_data: &CardData) -> &Texture2D {
        let path = card_data_to_image_path(card_data);
        self.textures.get(&path).unwrap()
    }
}
