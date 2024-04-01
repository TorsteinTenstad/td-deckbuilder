use std::collections::HashMap;

use macroquad::texture::{load_texture, Texture2D};

use crate::{card::Card, sprite_id::SpriteId, world::Direction};

#[derive(Debug, Default)]
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

    pub fn get_card_texture(&self, card: &Card) -> &Texture2D {
        self.card_textures
            .get(card)
            .expect("Card texture not found")
    }
}
