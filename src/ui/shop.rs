use bevy::{
    asset::Assets,
    prelude::{Commands, Entity, EventWriter, Mesh, Query, Res, ResMut, Transform},
    sprite::{ColorMaterial, Sprite},
};
use bevy_iced::{
    iced::{
        alignment::{Horizontal, Vertical},
        widget::{button, column, container, row, text, Container},
        Alignment, Background, Color, Theme,
    },
    IcedContext, Renderer,
};
use enum_iterator::all;

use crate::{
    bullets::BulletType,
    tank::Tank,
    utils::{GameMode, GameState, Player, ResetEvent},
    UiMessage,
};

#[derive(Clone, Copy)]
pub enum ShopMessage {
    BuyItem(BulletType),
    EndTurn,
}

pub struct BlackBackgroundContainer;

impl container::StyleSheet for BlackBackgroundContainer {
    type Style = bevy_iced::iced::Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let palette = style.palette();
        container::Appearance {
            text_color: Some(palette.text),
            background: Some(Background::Color(Color::BLACK)),
            ..Default::default()
        }
    }
}

pub fn get_custom_container_style() -> bevy_iced::iced::theme::Container {
    bevy_iced::iced::theme::Container::Custom(Box::new(BlackBackgroundContainer))
}

pub fn update_shop_ui<'a>(
    messages: impl Iterator<Item = &'a UiMessage>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
    mut state: ResMut<GameState>,
    mut reset_writer: EventWriter<ResetEvent>,
) {
    let msgs: Vec<&ShopMessage> = messages
        .filter_map(|val| match val {
            UiMessage::ShopMessage(message) => Some(message),
            _ => None,
        })
        .collect();
    for msg in msgs {
        match msg {
            ShopMessage::BuyItem(bullet_type) => println!("bought {}", bullet_type),
            ShopMessage::EndTurn => println!("end turn"),
        }
    }
}

pub fn view_shop_ui(
    state: Res<GameState>,
    player_query: Query<(&Player, &Tank)>,
    mut ctx: IcedContext<UiMessage>,
) {
    let wrap = UiMessage::ShopMessage;
    let (mut current_player_opt, mut player_tank_opt) = (None, None);
    for (player, tank) in player_query.iter() {
        if player.is_active {
            current_player_opt = Some(player);
            player_tank_opt = Some(tank);
        }
    }
    if let (Some(player), Some(tank)) = (current_player_opt, player_tank_opt) {
        let battle_button =
            button(text("shop")).on_press(UiMessage::SetSceneMessage(GameMode::Battle));
        let bullets = all::<BulletType>().collect::<Vec<_>>();
        let bullet_items: Vec<Container<UiMessage, Theme, Renderer>> = bullets
            .iter()
            .map(|elem| -> Container<UiMessage, Theme, Renderer> {
                container(column![
                    text(format!("Cost: {}", 10 /*TODO add cost*/)), /*button("buy").on_press(on_press)*/
                    button("buy").on_press(wrap(ShopMessage::BuyItem(*elem))),
                ])
            })
            .collect();
        let mut bullet_container = column![];
        for bullet in bullet_items {
            bullet_container = bullet_container.push(bullet);
        }
        ctx.display(
            container(column![
                row![battle_button, text("Hello")],
                row![container(bullet_container)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Top)]
            ])
            .padding(10)
            .width(1920)
            .height(1080)
            .style(get_custom_container_style()),
        )
    }
}
