use std::{cell::RefCell, rc::Rc};

use bevy::{
    asset::Assets,
    input::ButtonInput,
    prelude::{Commands, Component, Entity, KeyCode, Mesh, Query, Res, ResMut, Transform},
    sprite::{ColorMaterial, Sprite},
};

use crate::{
    bullets::{BulletInfo, BulletType},
    tank::Tank,
    utils::{get_current_player_props, Player},
};

#[derive(Component, Clone)]
pub struct KeyMap {
    tank_left: Rc<RefCell<KeyCode>>,
    tank_right: Rc<RefCell<KeyCode>>,
    aim_left: Rc<RefCell<KeyCode>>,
    aim_right: Rc<RefCell<KeyCode>>,
    fire: Rc<RefCell<KeyCode>>,
    switch_bullet: Rc<RefCell<KeyCode>>,
}

unsafe impl Send for KeyMap {}
unsafe impl Sync for KeyMap {}

impl KeyMap {
    pub fn default_keymap() -> KeyMap {
        KeyMap {
            tank_left: Rc::new(RefCell::new(KeyCode::ArrowLeft)),
            tank_right: Rc::new(RefCell::new(KeyCode::ArrowRight)),
            aim_left: Rc::new(RefCell::new(KeyCode::KeyA)),
            aim_right: Rc::new(RefCell::new(KeyCode::KeyD)),
            fire: Rc::new(RefCell::new(KeyCode::Space)),
            switch_bullet: Rc::new(RefCell::new(KeyCode::ShiftLeft)),
        }
    }
}

pub fn handle_keypress(
    mut query: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (_, mut player, mut tank, mut transform, mut sprite) =
        if let Some(props) = get_current_player_props(&mut query) {
            props
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
