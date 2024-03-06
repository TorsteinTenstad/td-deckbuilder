use serde::{Deserialize, Serialize};

use crate::{buff::ExtraHealthBuff, entity::EntityState, update_args::UpdateArgs};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Health {
    pub max_health: f32,
    pub health: f32,
    pub extra_health_buffs: Vec<ExtraHealthBuff>,
    pub damage_animation: f32,
}
impl Health {
    pub fn new(max_health: f32) -> Self {
        Self {
            max_health,
            health: max_health,
            ..Default::default()
        }
    }

    pub fn deal_damage(&mut self, damage: f32) {
        let mut damage = damage;
        for buff in self.extra_health_buffs.iter_mut() {
            if buff.health <= 0.0 {
                break;
            }
            let damage_to_take = damage.min(buff.health);
            damage -= damage_to_take;
            buff.health -= damage_to_take;
        }
        self.health -= damage;
        self.damage_animation = 0.1;
    }

    pub fn heal(&mut self, damage: f32) {
        self.health += damage;
        self.health = self.health.min(self.max_health);
    }
}

impl Health {
    pub fn update(update_args: &mut UpdateArgs) {
        update_args.entity_instance.entity.health.damage_animation -= update_args.dt;
        if let Some(seconds_left_to_live) =
            &mut update_args.entity_instance.entity.seconds_left_to_live
        {
            *seconds_left_to_live -= update_args.dt;
            if seconds_left_to_live < &mut 0.0 {
                update_args.entity_instance.state = EntityState::Dead;
            }
        }
        if update_args.entity_instance.entity.health.health <= 0.0
            && update_args.entity_instance.entity.health.damage_animation < 0.0
        {
            update_args.entity_instance.state = EntityState::Dead;
        }
    }
}
