use common::{
    card::{Card, CardInstance},
    rect_transform::RectTransform,
};
use macroquad::{
    math::Vec2,
    window::{screen_height, screen_width},
};

use crate::draw::GOLDEN_RATIO;

pub const CARD_BORDER: f32 = 5.0;
pub const CARD_VISIBLE_HEIGHT: f32 = 0.8;

#[derive(Debug, Clone)]
pub struct PhysicalCard {
    pub card: Card,
    pub transform: RectTransform,
    pub target_transform: RectTransform,
}

pub struct PhysicalCardInstance {
    pub card_instance: CardInstance,
    pub transform: RectTransform,
    pub target_transform: RectTransform,
}

impl PhysicalCardInstance {
    pub fn new(card_instance: CardInstance) -> Self {
        Self {
            card_instance,
            transform: RectTransform::default(),
            target_transform: RectTransform::default(),
        }
    }
}

pub fn card_transform_in_hand(
    card_idx: usize,
    hand_size: usize,
    relative_splay_radius: f32,
    card_delta_angle: f32,
) -> RectTransform {
    let w = screen_width() / 12.0;
    let h = w * GOLDEN_RATIO;
    RectTransform {
        w,
        h,
        x: screen_width() / 2.0,
        y: screen_height() + (relative_splay_radius * h) - (CARD_VISIBLE_HEIGHT * h),
        rotation: (card_idx as f32 - ((hand_size - 1) as f32 / 2.0)) * card_delta_angle,
        offset: Vec2 {
            x: 0.5,
            y: relative_splay_radius,
        },
    }
}

pub fn card_transform_outside_hand(x: f32, y: f32) -> RectTransform {
    let w = screen_width() / 10.0;
    RectTransform {
        w,
        h: w * GOLDEN_RATIO,
        x,
        y,
        rotation: 0.0,
        offset: 0.5 * Vec2::ONE,
    }
}

pub fn card_transform_hovered(
    card_idx: usize,
    hand_size: usize,
    relative_splay_radius: f32,
    card_delta_angle: f32,
) -> RectTransform {
    let w = screen_width() / 10.0;
    let h = w * GOLDEN_RATIO;
    let x = screen_width() / 2.0
        + ((relative_splay_radius * h) - (CARD_VISIBLE_HEIGHT * h))
            * f32::sin((card_idx as f32 - ((hand_size - 1) as f32 / 2.0)) * card_delta_angle);
    let y = screen_height();

    RectTransform {
        w,
        h: w * GOLDEN_RATIO,
        x,
        y,
        rotation: 0.0,
        offset: Vec2 { x: 0.5, y: 1.0 },
    }
}
