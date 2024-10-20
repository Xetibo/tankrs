use bevy::{
    math::Vec2,
    prelude::{Bundle, Component},
    sprite::SpriteBundle,
};

#[derive(Component)]
pub struct Tank {
    pub blocked_direction: Vec2,
    pub shooting_direction: Vec2,
    pub shooting_velocity: Vec2,
}

#[derive(Bundle)]
pub struct TankBundle {
    pub sprite: SpriteBundle,
    pub tank: Tank,
}
