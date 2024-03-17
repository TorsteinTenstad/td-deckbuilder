use crate::input::mouse_screen_position;
use common::{
    draw::{draw_rect_transform, draw_text_with_origin, TextOriginX, TextOriginY},
    rect_transform::{point_inside, RectTransform},
};
use macroquad::{
    color::{GRAY, WHITE},
    input::{get_char_pressed, is_mouse_button_pressed},
    miniquad::{date::now, MouseButton},
    text::Font,
};

pub struct TextBox {
    pub transform: RectTransform,
    pub text: String,
    pub active: bool,
}

impl TextBox {
    pub fn new(rect_transform: RectTransform) -> Self {
        Self {
            transform: rect_transform,
            text: "".to_string(),
            active: false,
        }
    }

    pub fn step(&mut self) {
        if self.active {
            while let Some(c) = get_char_pressed() {
                if c == '\n' {
                    self.active = false;
                    return;
                }
                if c == '\u{8}' {
                    self.text.pop();
                    return;
                }
                self.text.push(c);
            }
        } else {
            self.active = point_inside(mouse_screen_position(), &self.transform)
                && is_mouse_button_pressed(MouseButton::Left);
        }
    }

    pub fn draw(&self, font: Option<&Font>) {
        draw_rect_transform(&self.transform, WHITE);
        let cursor_blink_rate = 0.9;
        let text_with_cursor = self.text.clone()
            + if self.active && (now() % cursor_blink_rate < cursor_blink_rate / 2.0) {
                "|"
            } else {
                ""
            };
        draw_text_with_origin(
            &text_with_cursor,
            self.transform.x,
            self.transform.y + self.transform.h / 2.0,
            28.0,
            0.0,
            GRAY,
            TextOriginX::Left,
            TextOriginY::Center,
            font,
        )
    }
}
