use core::f32;

use bevy::{
    math::{Vec2, Vec3},
    prelude::{Bundle, Component},
    sprite::Sprite,
};

use crate::utils::Player;

#[derive(Clone, Copy)]
pub struct Angle {
    value: f32,
}

impl Default for Angle {
    fn default() -> Self {
        Angle {
            value: f32::consts::FRAC_PI_2,
        }
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

    //pub fn get_degrees(&self) -> f32 {
    //    self.value * (180.0 / f32::consts::PI)
    //}

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

impl Default for Tank {
    fn default() -> Tank {
        Tank {
            blocked_direction: Vec2::default(),
            scale: Vec3 {
                x: 100.0,
                y: 10.0,
                z: 0.0,
            },
            shooting_direction: Angle::default(),
            shooting_velocity: Vec2::new(1.0, 1.0),
        }
    }
}

#[derive(Bundle)]
pub struct TankBundle {
    pub sprite: Sprite,
    pub player: Player,
    pub tank: Tank,
}
