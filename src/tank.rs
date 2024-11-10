use bevy::{
    math::Vec2,
    prelude::{Bundle, Component},
    sprite::SpriteBundle,
};

use crate::utils::Player;

#[derive(Clone, Copy)]
pub struct Angle {
    value: f32,
}

impl Default for Angle {
    fn default() -> Self {
        Angle { value: 1.5 }
    }
}

impl Angle {
    fn check(value: f32) -> bool {
        (0.0..=std::f32::consts::PI).contains(&value)
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

#[derive(Component, Clone)]
pub struct Tank {
    pub blocked_direction: Vec2,
    pub shooting_direction: Angle,
    pub shooting_velocity: Vec2,
    pub(crate) scale: bevy::prelude::Vec3,
}

#[derive(Bundle)]
pub struct TankBundle {
    pub sprite: SpriteBundle,
    pub player: Player,
    pub tank: Tank,
}
