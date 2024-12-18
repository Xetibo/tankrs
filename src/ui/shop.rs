use bevy::{
    prelude::{Entity, EventWriter, Query, Res, ResMut, Transform},
    sprite::Sprite,
};
use bevy_iced::{
    iced::{
        widget::{button, column, container, row, text, Container},
        Alignment, Theme,
    },
    IcedContext, Renderer,
};
use enum_iterator::all;

use crate::{
    bullets::{BulletCount, BulletType},
    tank::Tank,
    utils::{EndTurnEvent, GameState, Player},
    UiMessage,
};

use super::utils::black_background::get_custom_container_style;

#[derive(Clone, Copy)]
pub enum ShopMessage {
    BuyItem(BulletType),
    EndTurn,
}

pub fn update_shop_ui<'a>(
    messages: impl Iterator<Item = &'a UiMessage>,
    state: ResMut<GameState>,
    mut query: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
    mut end_turn_writer: EventWriter<EndTurnEvent>,
) {
    let msgs: Vec<&ShopMessage> = messages
        .filter_map(|val| match val {
            UiMessage::ShopMessage(message) => Some(message),
            _ => None,
        })
        .collect();
    let mut current_player_opt = None;
    for (_, player, _, _, _) in &mut query {
        if state.active_player == player.player_number {
            current_player_opt = Some(player);
        }
    }
    if let Some(mut player) = current_player_opt {
        for msg in msgs {
            match msg {
                ShopMessage::BuyItem(bullet_type) => {
                    let cost = bullet_type.get_cost();
                    player.money -= cost;
                    let old = *player
                        .inventory
                        .get(bullet_type)
                        .unwrap_or(&BulletCount::Count(0));
                    player.inventory.insert(*bullet_type, old.increment());
                }
                ShopMessage::EndTurn => {
                    end_turn_writer.send(EndTurnEvent {});
                }
            }
        }
    }
}

pub fn view_shop_ui(
    state: Res<GameState>,
    player_query: Query<(&Player, &Tank)>,
    mut ctx: IcedContext<UiMessage>,
) {
    let wrap = UiMessage::ShopMessage;
    let mut current_player_opt = None;
    for (player, _) in player_query.iter() {
        if state.active_player == player.player_number {
            current_player_opt = Some(player);
        }
    }
    if let Some(player) = current_player_opt {
        let item_container = |elem: &BulletType| -> Option<Container<UiMessage, Theme, Renderer>> {
            let current_count = player.inventory.get(elem).unwrap_or(&BulletCount::Count(0));
            let cost = elem.get_cost();
            let max_count = elem.get_max_count();
            if let BulletCount::Count(count) = current_count {
                Some(container(column![
                    text(format!("Cost: {}, You currently have: {}", cost, count)),
                    button("buy").on_press_maybe(if cost <= player.money && *count < max_count {
                        Some(wrap(ShopMessage::BuyItem(*elem)))
                    } else {
                        None
                    }),
                ]))
            } else {
                None
            }
        };
        let battle_button = button(text("confirm")).on_press(wrap(ShopMessage::EndTurn));
        let bullets = all::<BulletType>().collect::<Vec<_>>();
        let bullet_items: Vec<Container<UiMessage, Theme, Renderer>> =
            bullets.iter().filter_map(item_container).collect();
        let mut bullet_container = column![];
        for bullet in bullet_items {
            bullet_container = bullet_container.push(bullet);
        }
        ctx.display(
            container(column![
                row![battle_button].padding(5),
                column![
                    text(format!("Player: {}", player.player_number)),
                    text(format!("Money: {}", player.money))
                ]
                .padding(5),
                row![container(bullet_container)]
                    .align_y(Alignment::Center)
                    .padding(5)
            ])
            .padding(10)
            //.width(5000)
            //.height(5000)
            .center_x(5000)
            .center_y(5000)
            .style(get_custom_container_style),
        )
    }
}
