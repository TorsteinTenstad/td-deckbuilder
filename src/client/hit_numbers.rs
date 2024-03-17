use std::{collections::HashMap, time::SystemTime};

use common::{entity::EntityInstance, ids::EntityId};
use macroquad::{
    color::{Color, GREEN, RED},
    math::Vec2,
    text::Font,
};

use common::draw::{draw_text_with_origin, to_screen_x, to_screen_y, TextOriginX, TextOriginY};

pub struct PhysicalHitNumber {
    pub number: i32,
    pub pos: Vec2,
    pub vel: Vec2,
    pub creation_time: SystemTime,
}

pub struct HitNumbers {
    pub physical_hit_numbers: Vec<PhysicalHitNumber>,
    pub entity_healths: HashMap<EntityId, f32>,
}

impl HitNumbers {
    const ALIVE_TIME: f32 = 0.8;
    const SPEED: f32 = 150.0;

    pub fn new() -> Self {
        Self {
            physical_hit_numbers: Vec::new(),
            entity_healths: HashMap::new(),
        }
    }
    pub fn step(&mut self, entities: &[EntityInstance], dt: f32) {
        for entity_instance in entities.iter() {
            if let Some(old_health) = self.entity_healths.get(&entity_instance.id) {
                let health_diff = entity_instance.entity.health.health - old_health;
                if health_diff.abs() > 1.0 {
                    self.physical_hit_numbers.push(PhysicalHitNumber {
                        number: health_diff as i32,
                        pos: entity_instance.pos,
                        vel: Vec2::NEG_Y * Self::SPEED,
                        creation_time: SystemTime::now(),
                    });
                }
            }
        }
        self.entity_healths = entities
            .iter()
            .map(|entity_instance| (entity_instance.id, entity_instance.entity.health.health))
            .collect();

        let time = SystemTime::now();
        self.physical_hit_numbers.retain_mut(|physical_hit_number| {
            physical_hit_number.pos += physical_hit_number.vel * dt;
            time.duration_since(physical_hit_number.creation_time)
                .unwrap()
                .as_secs_f32()
                < Self::ALIVE_TIME
        });
    }

    pub fn draw(&self, font: Option<&Font>) {
        for physical_hit_number in &self.physical_hit_numbers {
            let alpha = 1.0
                - (SystemTime::now()
                    .duration_since(physical_hit_number.creation_time)
                    .unwrap()
                    .as_secs_f32()
                    / Self::ALIVE_TIME);
            let rgb = if physical_hit_number.number < 0 {
                RED
            } else {
                GREEN
            };

            draw_text_with_origin(
                format!("{}", physical_hit_number.number).as_str(),
                to_screen_x(physical_hit_number.pos.x),
                to_screen_y(physical_hit_number.pos.y),
                28.0,
                0.0,
                Color { a: alpha, ..rgb },
                TextOriginX::Center,
                TextOriginY::Center,
                font,
            )
        }
    }
}
