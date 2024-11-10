use std::{cell::RefCell, sync::Arc};

use bevy::{
    asset::Assets,
    input::ButtonInput,
    prelude::{Commands, Component, KeyCode, Mesh, Query, Res, ResMut, Transform},
    sprite::{ColorMaterial, Sprite},
};

use crate::{
    bullets::{BulletInfo, BulletType},
    tank::Tank,
    utils::Player,
};

#[derive(Component, Clone)]
pub struct KeyMap {
    tank_left: Arc<RefCell<KeyCode>>,
    tank_right: Arc<RefCell<KeyCode>>,
    aim_left: Arc<RefCell<KeyCode>>,
    aim_right: Arc<RefCell<KeyCode>>,
    fire: Arc<RefCell<KeyCode>>,
    switch_bullet: Arc<RefCell<KeyCode>>,
}

unsafe impl Send for KeyMap {}
unsafe impl Sync for KeyMap {}

impl KeyMap {
    pub fn default_keymap() -> KeyMap {
        KeyMap {
            tank_left: Arc::new(RefCell::new(KeyCode::ArrowLeft)),
            tank_right: Arc::new(RefCell::new(KeyCode::ArrowRight)),
            aim_left: Arc::new(RefCell::new(KeyCode::KeyA)),
            aim_right: Arc::new(RefCell::new(KeyCode::KeyD)),
            fire: Arc::new(RefCell::new(KeyCode::Space)),
            switch_bullet: Arc::new(RefCell::new(KeyCode::ShiftLeft)),
        }
    }
}

pub fn handle_keypress(
    mut query: Query<(&mut Tank, &mut Player, &mut Transform, &mut Sprite)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // TODO deduplicate
    let (mut tank_opt, mut player_opt, mut transform_opt, mut sprite_opt) =
        (None, None, None, None);
    for (tank, player, transform, sprite) in &mut query {
        if player.is_active {
            tank_opt = Some(tank);
            player_opt = Some(player);
            transform_opt = Some(transform);
            sprite_opt = Some(sprite);
        }
    }
    let (mut tank, mut player, mut transform, mut sprite) =
        if let (Some(tank), Some(player), Some(transform), Some(sprite)) =
            (tank_opt, player_opt, transform_opt, sprite_opt)
        {
            (tank, player, transform, sprite)
        } else {
            return;
        };
    if keys.pressed(*player.key_map.tank_right.borrow()) {
        transform.translation.x += 10.0;
        sprite.flip_x = false;
    }
    if keys.pressed(*player.key_map.tank_left.borrow()) {
        transform.translation.x -= 10.0;
        sprite.flip_x = true;
    }

    // x 1.0 y 0.0
    // x 0.0 y 1.0
    // x -1.0 y 0.0
    let current = tank.shooting_direction.get();
    if keys.pressed(*player.key_map.aim_right.borrow()) {
        tank.shooting_direction.set(current - 0.01);
    }
    if keys.pressed(*player.key_map.aim_left.borrow()) {
        tank.shooting_direction.set(current + 0.01);
    }
    if keys.pressed(*player.key_map.fire.borrow()) {
        let info = BulletInfo {
            direction: &tank.shooting_direction,
            velocity: &tank.shooting_velocity,
            origin: &transform.translation,
        };
        (player.selected_bullet.1)(&mut commands, &mut meshes, &mut materials, &info);
    }
    if keys.pressed(*player.key_map.switch_bullet.borrow()) {
        let previous_bullet = player.selected_bullet.0.get_int_value();
        let new_bullet = BulletType::get_from_int(previous_bullet + 1);
        let bullet_count_opt = player.inventory.get_mut(&new_bullet);

        if let Some(count_type) = bullet_count_opt {
            match count_type {
                crate::bullets::BulletCount::Unlimited => (),
                crate::bullets::BulletCount::Count(count) => {
                    // TODO limit count on shooting
                    if *count == 0 {
                        player.inventory.remove(&new_bullet);
                    }
                    let new_bullet_func = new_bullet.get_bullet_from_type();
                    player.selected_bullet = (new_bullet, new_bullet_func);
                }
            }
        }
    }
}
