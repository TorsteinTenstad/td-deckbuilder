use common::card::{Card, CardData};
use macroquad::{
    color::{Color, BLACK, WHITE},
    math::Vec2,
    text::{
        camera_font_scale, draw_text_ex, load_ttf_font, measure_text, Font, TextDimensions,
        TextParams,
    },
    texture::{
        draw_texture, draw_texture_ex, get_screen_data, load_texture, DrawTextureParams, Texture2D,
    },
    window::{clear_background, next_frame, request_new_screen_size},
};

#[macroquad::main("Card Generator")]
async fn main() {
    let template_texture = load_texture("assets/textures/card_template.png")
        .await
        .unwrap();
    let font = load_ttf_font("assets\\fonts\\shaky-hand-some-comic.bold.ttf")
        .await
        .unwrap();
    request_new_screen_size(template_texture.width(), template_texture.height());
    next_frame().await;
    next_frame().await;
    for card in Card::iter() {
        clear_background(Color::new(0., 0., 0., 0.));
        draw_card(&template_texture, &font, &card.get_card_data()).await;
        get_screen_data().export_png(card.get_texture_path().as_str());
        next_frame().await;
    }
}

async fn draw_card(template_texture: &Texture2D, font: &Font, card_data: &CardData) {
    let card_art_texture =
        load_texture(format!("assets/textures/card_art/{}", card_data.card_art_path).as_str())
            .await;
    let card_art_texture = match card_art_texture {
        Ok(texture) => texture,
        Err(_) => Texture2D::empty(),
    };

    let card_art_margin = 0.05 * template_texture.width();
    let card_art_w = template_texture.width() - 2.0 * card_art_margin;
    let card_art_h = card_art_w * card_art_texture.height() / card_art_texture.width();
    draw_texture_ex(
        &card_art_texture,
        card_art_margin,
        card_art_margin,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2::new(card_art_w, card_art_h)),
            ..Default::default()
        },
    );

    draw_texture(template_texture, 0.0, 0.0, WHITE);

    draw_centered_text(
        card_data.name,
        template_texture.width() / 2.0,
        460.0,
        60.0,
        font,
    );

    draw_centered_text(
        card_data.description,
        template_texture.width() / 2.0,
        560.0,
        36.0,
        font,
    );

    draw_centered_text(
        card_data.energy_cost.to_string().as_str(),
        75.0,
        98.0,
        120.0,
        font,
    );

    let attack_number_str = match card_data.attack {
        Some(attack) => attack.to_string(),
        None => String::from(""),
    };

    draw_centered_text(attack_number_str.as_str(), 75.0, 750.0, 70.0, font);

    let health_number_str = match card_data.health {
        Some(health) => health.to_string(),
        None => String::from(""),
    };

    draw_centered_text(health_number_str.as_str(), 570.0, 750.0, 70.0, font);
}
fn draw_centered_text(text: &str, x: f32, y: f32, font_size: f32, font: &Font) {
    let (font_size, font_scale, font_scale_aspect) = camera_font_scale(font_size);
    let TextDimensions {
        width,
        height: _,
        offset_y,
    } = measure_text(text, None, font_size, font_scale);
    draw_text_ex(
        text,
        x - width / 2.0,
        y + offset_y / 2.0,
        TextParams {
            font_size,
            font_scale,
            font_scale_aspect,
            color: BLACK,
            font: Some(font),
            ..Default::default()
        },
    )
}
