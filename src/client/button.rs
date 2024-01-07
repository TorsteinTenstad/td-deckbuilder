

pub struct Button{
    pub transform: RectTransform,
    pub text: String,
}

impl Button{
    pub fn new(rect_transform: RectTransform, text: String) -> Self{
        Self{
            transform: rect_transform,
            text,
        }
    }

    pub fn step(&mut self){
        if point_inside(mouse_screen_position(), &self.transform) && is_mouse_button_pressed(MouseButton::Left){
            println!("Button pressed!");
        }
    }

    pub fn draw(&self, font: Option<&Font>){
        draw_rect_transform(&self.transform, WHITE);
        draw_text_with_origin(
            &self.text,
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