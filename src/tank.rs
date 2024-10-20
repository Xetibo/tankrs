use bevy::{
    math::Vec2,
    prelude::{Bundle, Component},
    sprite::SpriteBundle,
};

#[derive(Clone, Copy)]
pub struct Angle {
    value: f32,
}

impl Default for Angle {
    fn default() -> Self {
        Angle { value: 90.0 }
    }
}

impl Angle {
    fn check(value: f32) -> bool {
        if value >= 0.0 && value <= 180.0 {
            true
        } else {
            false
        }
    }

    pub fn new(value: f32) -> Option<Angle> {
        if Self::check(value) {
            Some(Angle { value })
        } else {
            None
        }
    }

    pub fn get(&self) -> f32 {
        self.value
    }

    pub fn set(&mut self, value: f32) {
        if Self::check(value) {
            self.value = value;
        } else {
            println!(
                "The value of the angle was outside of the range 0.0-180.0: {}",
                value
            );
        }
    }
}

#[derive(Component)]
pub struct Tank {
    pub blocked_direction: Vec2,
    pub shooting_direction: Angle,
    pub shooting_velocity: Vec2,
}

#[derive(Bundle)]
pub struct TankBundle {
    pub sprite: SpriteBundle,
    pub tank: Tank,
}
