use core::f32;
use std::ops::RangeInclusive;

use bevy::{
    asset::Assets,
    prelude::{Commands, Entity, EventReader, EventWriter, Mesh, Query, ResMut, Transform},
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
    utils::{get_current_player_props, EndTurnEvent, Player, ResetEvent},
    UiMessage,
};

pub fn view_ui(player_query: Query<(&Player, &Tank)>, mut ctx: IcedContext<UiMessage>) {
    let button = button(text("Reset")).on_press(UiMessage::Reset);
    let (mut current_player_opt, mut player_tank_opt) = (None, None);
    for (player, tank) in player_query.iter() {
        if player.is_active {
            current_player_opt = Some(player);
            player_tank_opt = Some(tank);
        }
    }
    if let (Some(player), Some(tank)) = (current_player_opt, player_tank_opt) {
        ctx.display(row![
            button,
            text(format!("Player: {}", player.player_number)),
            bullet_picker(player).into(),
            fuel(player).into(),
            firing(player, tank).into()
        ]);
    } else {
        ctx.display(row![button,]);
    }
}

/// help text
pub fn update_ui(
    mut messages: EventReader<UiMessage>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
    mut writer: EventWriter<EndTurnEvent>,
    mut reset_writer: EventWriter<ResetEvent>,
) {
    let (_, mut player, mut tank, mut transform, _) =
        if let Some(props) = get_current_player_props(&mut query) {
            props
        } else {
            return;
        };
    for msg in messages.read() {
        match msg {
            UiMessage::Reset => {
                reset_writer.send(ResetEvent {});
            }
            UiMessage::MoveRight => {
                // TODO deduplicate form inputs
                transform.translation.x += 10.0;
            }
            UiMessage::MoveLeft => {
                // TODO deduplicate form inputs
                transform.translation.x -= 10.0;
            }
            UiMessage::Fire => {
                // TODO deduplicate form inputs
                let info = BulletInfo {
                    direction: &tank.shooting_direction,
                    velocity: &tank.shooting_velocity,
                    origin: &transform.translation,
                };
                (player.selected_bullet.1)(&mut commands, &mut meshes, &mut materials, &info);
                // TODO make this depend on other logic
                writer.send(EndTurnEvent {});
            }
            UiMessage::SetVelocity(velocity) => {
                player.fire_velocity = *velocity;
            }
            UiMessage::SetAngle(angle) => {
                tank.shooting_direction.set(*angle);
            }
            UiMessage::SelectBullet(bullet) => {
                let bullet_fn = bullet.get_bullet_from_type();
                player.selected_bullet = (bullet.clone(), bullet_fn);
            }
        }
    }
}

type IcedElement = bevy_iced::iced::Element<'static, UiMessage, Theme, Renderer>;

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
        UiMessage::SelectBullet
    )]
}

fn fuel(player: &Player) -> impl Into<IcedElement> {
    column![
        button(text("left")).on_press(UiMessage::MoveLeft),
        text(format!("Fuel: {}", player.fuel)),
        button(text("right")).on_press(UiMessage::MoveRight)
    ]
}

fn firing(player: &Player, tank: &Tank) -> impl Into<IcedElement> {
    // TODO deduplicate
    let angle_range = RangeInclusive::new(0.0, f32::consts::PI);
    let current_angle = tank.shooting_direction.get();

    let velocity_range = RangeInclusive::new(0, 100);
    let current_velocity = player.fire_velocity;
    column![
        text("Set Angle"),
        slider(angle_range, current_angle, UiMessage::SetAngle),
        text("Set Velocity"),
        slider(velocity_range, current_velocity, UiMessage::SetVelocity),
        button(text("fire")).on_press(UiMessage::Fire),
    ]
}
