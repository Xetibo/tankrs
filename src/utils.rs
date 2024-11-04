use std::fmt::{Display, Write};

use bevy::{
    asset::Assets,
    prelude::{Commands, Component, Mesh, ResMut},
    sprite::ColorMaterial,
    utils::HashMap,
};

use crate::{
    bullets::{BulletBundle, BulletCount, BulletInfo, BulletType},
    inputs::KeyMap,
};

#[derive(Component)]
pub struct Inventory {
    //
}

#[derive(Component, Clone)]
pub struct Player {
    pub inventory: HashMap<BulletType, BulletCount>,
    pub selected_bullet: (
        BulletType,
        fn(
            &mut Commands,
            &mut ResMut<Assets<Mesh>>,
            &mut ResMut<Assets<ColorMaterial>>,
            &BulletInfo,
        ),
    ),
    pub health: u32,
    pub fuel: u32,
    pub key_map: KeyMap,
}

impl Player {
    pub fn selected_bullet(
        &self,
    ) -> fn(
        &mut Commands<'_, '_>,
        &mut ResMut<'_, Assets<Mesh>>,
        &mut ResMut<'_, Assets<ColorMaterial>>,
        &BulletInfo,
    ) {
        self.selected_bullet.1
    }
}

pub fn polynomial(x: i32, rand: f32) -> f32 {
    let x = x as f32;
    //(f32::consts::E - x) * (x * f32::consts::E) *
    //(rand * power(x, 4)) + (rand * power(x, 3)) - (rand * power(x, 2)) - (rand * x)
    //power(x, 5) - power(x, 4) - (5.0 * power(x, 3)) + (3.0 * power(x, 2)) + (4.0 * x) - 3.0
    //((rand * x) * 0.00005).cos() * 1000.0 * rand
    (x * rand * 0.005).cos() * 100. * rand + 100.0
}

pub fn power(num: f32, pow: i32) -> f32 {
    if pow > 0 {
        power(num, pow - 1)
    } else {
        1.0
    }
}
