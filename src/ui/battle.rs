use core::f32;
use std::ops::RangeInclusive;

use bevy::{
    asset::{AssetServer, Assets},
    math::Vec2,
    prelude::{Commands, Entity, EventWriter, Mesh, Query, Res, ResMut, Transform},
    sprite::{ColorMaterial, Sprite},
    time::Time,
};
use bevy_iced::{
    iced::{
        self,
        widget::{column, row, text},
        Theme,
    },
    IcedContext, Renderer,
};
use oxiced::widgets::{
    oxi_button::{self, ButtonVariant},
    oxi_picklist, oxi_slider,
};

use crate::{
    bullets::{BulletCount, BulletInfo, BulletType},
    tank::Tank,
    utils::{get_current_player_props, polynomial, BulletHelpers, GameState, Player, ResetEvent},
    UiMessage,
};

#[derive(Clone)]
pub enum BattleMessage {
    Reset,
    MoveRight,
    MoveLeft,
    Fire,
    SetVelocity(f32),
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

pub fn view_battle_ui(
    state: Res<GameState>,
    player_query: Query<(&Player, &Tank)>,
    mut ctx: IcedContext<UiMessage>,
) {
    let reset_button = oxi_button::button::<UiMessage, Theme, iced::Renderer>(
        text("Reset"),
        ButtonVariant::Primary,
    )
    .on_press(wrap(BattleMessage::Reset));
    let (mut current_player_opt, mut player_tank_opt) = (None, None);
    for (player, tank) in player_query.iter() {
        if state.active_player == player.player_number {
            current_player_opt = Some(player);
            player_tank_opt = Some(tank);
        }
    }
    if let (Some(player), Some(tank)) = (current_player_opt, player_tank_opt) {
        let current_bullet = player.selected_bullet.bullet_type;
        let current_bullet_count_opt = player.inventory.get(&current_bullet);
        let current_bullet_count_str =
            if let Some(BulletCount::Count(count)) = current_bullet_count_opt {
                format!("{}", count)
            } else {
                "unlimited".into()
            };
        ctx.display(
            row![
                reset_button,
                bullet_picker(player).into(),
                text(current_bullet_count_str),
                fuel(player).into(),
                firing(player, tank).into(),
                info_box(state.wind, player).into()
            ]
            .spacing(20)
            .padding(10)
            .width(1300),
        );
    } else {
        ctx.display(row![reset_button,]);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_battle_ui<'a, 'w, 's>(
    messages: impl Iterator<Item = &'a UiMessage>,
    time: Res<Time>,
    mut commands: Commands<'w, 's>,
    mut materials: ResMut<'w, Assets<ColorMaterial>>,
    mut meshes: ResMut<'w, Assets<Mesh>>,
    mut query: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
    mut state: ResMut<'w, GameState>,
    mut reset_writer: EventWriter<ResetEvent>,
    asset_server: Res<'w, AssetServer>,
) {
    let msgs: Vec<&BattleMessage> = messages
        .filter_map(|val| match val {
            UiMessage::BattleMessage(message) => Some(message),
            _ => None,
        })
        .collect();
    if state
        .active_bullets
        .load(std::sync::atomic::Ordering::Relaxed)
        > 0
    {
        for msg in msgs {
            if let BattleMessage::Reset = msg {
                reset_writer.send(ResetEvent {});
            }
        }
        return;
    }
    let (_, mut player, mut tank, mut transform, _) =
        if let Some(props) = get_current_player_props(state.active_player, &mut query) {
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
    let delta = time.delta_secs();
    for msg in msgs {
        match msg {
            BattleMessage::Reset => {
                reset_writer.send(ResetEvent {});
            }
            BattleMessage::MoveRight => {
                transform.translation.x += player.drive(10) * delta;
                transform.translation.y =
                    polynomial(transform.translation.x as i32, &state) - 695.0;
            }
            BattleMessage::MoveLeft => {
                transform.translation.x -= player.drive(10) * delta;
                transform.translation.y =
                    polynomial(transform.translation.x as i32, &state) - 695.0;
            }
            BattleMessage::Fire => {
                let bullet_type = player.selected_bullet.bullet_type;
                let count_type = *player
                    .inventory
                    .get(&bullet_type)
                    .unwrap_or(&BulletCount::Count(0));
                match count_type {
                    BulletCount::Unlimited => (),
                    BulletCount::Count(count) => {
                        if count == 0 {
                            return;
                        } else {
                            player.inventory.insert(bullet_type, count_type.decrement());
                        }
                    }
                }
                let angle = &tank.shooting_direction.get();
                let x_unit_vec = (angle).cos() * -1.0;
                let info = BulletInfo {
                    velocity: &Vec2 {
                        x: x_unit_vec * player.fire_velocity,
                        y: (angle).sin() * player.fire_velocity,
                    },
                    origin: &transform.translation,
                    owner: player.player_number,
                };

                let mut helpers = BulletHelpers {
                    commands: &mut commands,
                    state: &mut state,
                    meshes: &mut meshes,
                    materials: &mut materials,
                    assetserver: &asset_server,
                };
                (player.selected_bullet.firefn)(&mut helpers, &info);
            }
            BattleMessage::SetVelocity(velocity) => {
                player.fire_velocity = *velocity;
            }
            BattleMessage::SetAngle(angle) => {
                tank.shooting_direction.set(*angle);
            }
            BattleMessage::SelectBullet(bullet) => {
                let bullet_fn = bullet.get_bullet_from_type();
                player.selected_bullet = bullet_fn;
            }
        }
    }
}

type IcedElement = bevy_iced::iced::Element<'static, UiMessage, Theme, Renderer>;

fn wrap(msg: BattleMessage) -> UiMessage {
    UiMessage::BattleMessage(msg)
}

fn bullet_picker(player: &Player) -> impl Into<IcedElement> {
    let options: Vec<BulletType> = player.inventory.keys().copied().collect();
    let selected = Some(player.selected_bullet.bullet_type);
    column![oxi_picklist::pick_list(options, selected, |val| wrap(
        BattleMessage::SelectBullet(val)
    ))]
}

fn fuel(player: &Player) -> impl Into<IcedElement> {
    row![
        // TODO make this work continuously
        oxi_button::button::<UiMessage, Theme, iced::Renderer>(
            text("left"),
            ButtonVariant::Primary
        )
        .on_press(wrap(BattleMessage::MoveLeft))
        .padding(5),
        text(format!("Fuel: {}", player.fuel)),
        oxi_button::button::<UiMessage, Theme, iced::Renderer>(
            text("right"),
            ButtonVariant::Primary
        )
        .on_press(wrap(BattleMessage::MoveRight))
        .padding(5)
    ]
    .spacing(10)
}

fn firing(player: &Player, tank: &Tank) -> impl Into<IcedElement> {
    let angle_range = RangeInclusive::new(0.0, f32::consts::PI);
    let current_angle = tank.shooting_direction.get();

    let velocity_range = RangeInclusive::new(0.0, 10.0);
    let current_velocity = player.fire_velocity;
    row![
        column![
            text(format!(
                "Angle: {:.0}",
                current_angle * 180.0 / f32::consts::PI
            )),
            oxi_slider::slider(angle_range, current_angle, |val| wrap(
                BattleMessage::SetAngle(val)
            ))
            .step(0.01),
        ],
        column![
            text(format!("Velocity: {:.0}", current_velocity)),
            oxi_slider::slider(velocity_range, current_velocity, |val| wrap(
                BattleMessage::SetVelocity(val)
            ))
            .step(0.1),
        ],
        oxi_button::button::<UiMessage, Theme, iced::Renderer>(
            text("fire"),
            ButtonVariant::Primary
        )
        .on_press(wrap(BattleMessage::Fire)),
    ]
    .spacing(10)
}

fn info_box(wind: f32, player: &Player) -> impl Into<IcedElement> {
    // TODO display properly
    column![
        text(format!("Wind: {:.2}", wind)),
        text(format!("Player: {}", player.player_number)),
        text(format!("Health: {}", player.health)),
        text(format!("Money: {}", player.money)),
    ]
}
