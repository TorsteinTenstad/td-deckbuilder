use macroquad::math::Vec2;

#[derive(Debug, Clone, Default)]
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

pub fn point_inside(point: Vec2, transform: &RectTransform) -> bool {
    let transformed_point = Vec2::from_angle(-transform.rotation).rotate(
        point
            - Vec2 {
                x: transform.x,
                y: transform.y,
            },
    ) + transform.offset
        * Vec2 {
            x: transform.w,
            y: transform.h,
        };

    transformed_point.cmpgt(Vec2::ZERO).all()
        && transformed_point
            .cmplt(Vec2::new(transform.w, transform.h))
            .all()
}
