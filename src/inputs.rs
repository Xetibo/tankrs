use std::{cell::RefCell, rc::Rc};

use bevy::{
    input::ButtonInput,
    prelude::{Component, EventWriter, KeyCode, Query, Res},
};

use crate::{
    bullets::BulletType,
    tank::Tank,
    ui::battle::BattleMessage,
    utils::{GameMode, GameState, Player},
    UiMessage,
};

#[derive(Component, Clone)]
pub struct KeyMap {
    tank_left: Rc<RefCell<KeyCode>>,
    tank_right: Rc<RefCell<KeyCode>>,
    aim_left: Rc<RefCell<KeyCode>>,
    aim_right: Rc<RefCell<KeyCode>>,
    fire: Rc<RefCell<KeyCode>>,
    switch_bullet: Rc<RefCell<KeyCode>>,
    velocity_up: Rc<RefCell<KeyCode>>,
    velocity_down: Rc<RefCell<KeyCode>>,
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
            velocity_up: Rc::new(RefCell::new(KeyCode::KeyQ)),
            velocity_down: Rc::new(RefCell::new(KeyCode::KeyE)),
        }
    }
}

pub fn handle_keypress(
    query: Query<(&Player, &Tank)>,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    mut writer: EventWriter<UiMessage>,
) {
    if state.firing || state.mode != GameMode::Battle {
        return;
    }
    let (mut player_opt, mut tank_opt) = (None, None);
    for (player, tank) in query.iter() {
        if state.active_player == player.player_number {
            player_opt = Some(player);
            tank_opt = Some(tank);
        }
    }
    let (player, tank) = if let (Some(player), Some(tank)) = (player_opt, tank_opt) {
        (player, tank)
    } else {
        return;
    };
    let wrap = UiMessage::BattleMessage;
    if keys.pressed(*player.key_map.tank_right.borrow()) {
        writer.send(wrap(BattleMessage::MoveRight));
    }
    if keys.pressed(*player.key_map.tank_left.borrow()) {
        writer.send(wrap(BattleMessage::MoveLeft));
    }

    let current_angle = tank.shooting_direction.get();
    let current_velocity = player.fire_velocity;
    if keys.pressed(*player.key_map.aim_right.borrow()) {
        writer.send(wrap(BattleMessage::SetAngle(current_angle - 0.01)));
    }
    if keys.pressed(*player.key_map.aim_left.borrow()) {
        writer.send(wrap(BattleMessage::SetAngle(current_angle + 0.01)));
    }
    if keys.just_released(*player.key_map.fire.borrow()) {
        writer.send(wrap(BattleMessage::Fire));
    }
    if keys.just_released(*player.key_map.velocity_up.borrow()) {
        writer.send(wrap(BattleMessage::SetVelocity(current_velocity + 0.1)));
    }
    if keys.just_released(*player.key_map.velocity_down.borrow()) {
        writer.send(wrap(BattleMessage::SetVelocity(current_velocity - 0.1)));
    }
    if keys.pressed(*player.key_map.switch_bullet.borrow()) {
        let previous_bullet = player.selected_bullet.bullet_type.get_int_value();
        let new_bullet = BulletType::get_from_int(previous_bullet + 1);
        writer.send(wrap(BattleMessage::SelectBullet(new_bullet)));
    }
}
