use core::f32;
use std::ops::RangeInclusive;

use bevy::{
    asset::Assets,
    prelude::{Commands, Entity, EventReader, EventWriter, Mesh, Query, Res, ResMut, Transform},
    sprite::{ColorMaterial, Sprite},
};
use bevy_iced::{
    iced::{
        widget::{button, column, row, slider, text},
        Theme,
    },
    IcedContext, Renderer,
};

use crate::{
    bullets::{BulletInfo, BulletType},
    tank::Tank,
    utils::{get_current_player_props, GameState, Player, ResetEvent},
    UiMessage,
};

#[derive(Clone)]
pub enum BattleMessage {
    Reset,
    MoveRight,
    MoveLeft,
    Fire,
    SetVelocity(u32),
    SetAngle(f32),
    SelectBullet(BulletType),
    // UseRepair,
    // Teleport,
    // Parachute,
}

impl From<BattleMessage> for UiMessage {
    fn from(val: BattleMessage) -> Self {
        UiMessage::BattleMessage(val)
    }
}

pub fn view_ui(
    state: Res<GameState>,
    player_query: Query<(&Player, &Tank)>,
    mut ctx: IcedContext<UiMessage>,
) {
    let button = button(text("Reset")).on_press(wrap(BattleMessage::Reset));
    let (mut current_player_opt, mut player_tank_opt) = (None, None);
    for (player, tank) in player_query.iter() {
        if player.is_active {
            current_player_opt = Some(player);
            player_tank_opt = Some(tank);
        }
    }
    if let (Some(player), Some(tank)) = (current_player_opt, player_tank_opt) {
        ctx.display(
            row![
                button,
                bullet_picker(player).into(),
                fuel(player).into(),
                firing(player, tank).into(),
                info_box(state.wind, player).into()
            ]
            .spacing(20)
            .padding(10)
            .width(1300),
        );
    } else {
        ctx.display(row![button,]);
    }
}

pub fn update_ui(
    mut messages: EventReader<UiMessage>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
    mut state: ResMut<GameState>,
    mut reset_writer: EventWriter<ResetEvent>,
) {
    let msgs: Vec<&BattleMessage> = messages
        .read()
        .filter_map(|val| match val {
            UiMessage::BattleMessage(message) => Some(message),
            _ => None,
        })
        .collect();
    if state.firing {
        for msg in msgs {
            if let BattleMessage::Reset = msg {
                reset_writer.send(ResetEvent {});
            }
        }
        return;
    }
    let (_, mut player, mut tank, mut transform, _) =
        if let Some(props) = get_current_player_props(&mut query) {
            props
        } else {
            // TODO this is not good
            for msg in msgs {
                if let BattleMessage::Reset = msg {
                    reset_writer.send(ResetEvent {});
                }
            }
            return;
        };
    for msg in msgs {
        match msg {
            BattleMessage::Reset => {
                reset_writer.send(ResetEvent {});
            }
            BattleMessage::MoveRight => {
                // TODO deduplicate form inputs
                transform.translation.x += 10.0;
            }
            BattleMessage::MoveLeft => {
                // TODO deduplicate form inputs
                transform.translation.x -= 10.0;
            }
            BattleMessage::Fire => {
                // TODO deduplicate form inputs
                let info = BulletInfo {
                    direction: &tank.shooting_direction,
                    velocity: &tank.shooting_velocity,
                    origin: &transform.translation,
                };
                (player.selected_bullet.1)(&mut commands, &mut meshes, &mut materials, &info);
                state.firing = true;
            }
            BattleMessage::SetVelocity(velocity) => {
                player.fire_velocity = *velocity;
            }
            BattleMessage::SetAngle(angle) => {
                tank.shooting_direction.set(*angle);
            }
            BattleMessage::SelectBullet(bullet) => {
                let bullet_fn = bullet.get_bullet_from_type();
                player.selected_bullet = (bullet.clone(), bullet_fn);
            }
        }
    }
}

type IcedElement = bevy_iced::iced::Element<'static, UiMessage, Theme, Renderer>;

fn wrap(msg: BattleMessage) -> UiMessage {
    UiMessage::BattleMessage(msg)
}

fn bullet_picker(player: &Player) -> impl Into<IcedElement> {
    let options: Vec<BulletType> = player
        .inventory
        .keys()
        .map(|elem| (*elem).clone())
        .collect();
    let selected = Some(player.selected_bullet.0.clone());
    column![bevy_iced::iced::widget::pick_list(
        options,
        selected,
        |val| wrap(BattleMessage::SelectBullet(val))
    )]
}

fn fuel(player: &Player) -> impl Into<IcedElement> {
    row![
        button(text("left"))
            .on_press(wrap(BattleMessage::MoveLeft))
            .padding(5),
        text(format!("Fuel: {}", player.fuel)),
        button(text("right"))
            .on_press(wrap(BattleMessage::MoveRight))
            .padding(5)
    ]
    .spacing(10)
}

fn firing(player: &Player, tank: &Tank) -> impl Into<IcedElement> {
    // TODO deduplicate
    let angle_range = RangeInclusive::new(0.0, f32::consts::PI);
    let current_angle = tank.shooting_direction.get();

    let velocity_range = RangeInclusive::new(0, 100);
    let current_velocity = player.fire_velocity;
    row![
        column![
            text("Set Angle"),
            slider(angle_range, current_angle, |val| wrap(
                BattleMessage::SetAngle(val)
            )),
        ],
        column![
            text("Set Velocity"),
            slider(velocity_range, current_velocity, |val| wrap(
                BattleMessage::SetVelocity(val)
            )),
        ],
        button(text("fire")).on_press(wrap(BattleMessage::Fire)),
    ]
    .spacing(10)
}

fn info_box(wind: i32, player: &Player) -> impl Into<IcedElement> {
    // TODO display properly
    column![
        text(format!("wind: {}", wind)),
        text(format!("Player: {}", player.player_number)),
        text(format!("health: {}", player.health)),
        text(format!("health: {}", player.money)),
    ]
}
