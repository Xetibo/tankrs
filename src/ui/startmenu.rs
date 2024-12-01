use bevy::prelude::{EventWriter, Res, ResMut};
use bevy_iced::{
    iced::{
        alignment::{Horizontal, Vertical},
        widget::{button, column, container, row, text, text_input},
    },
    IcedContext,
};

use crate::{
    utils::{GameMode, GameState, ResetEvent},
    UiMessage,
};

use super::utils::black_background::get_custom_container_style;

#[derive(Clone)]
pub enum StartMenuMessage {
    ChoosePlayerCount(String),
    Start,
}

pub fn update_startmenu_ui<'a>(
    messages: impl Iterator<Item = &'a UiMessage>,
    mut state: ResMut<GameState>,
    mut reset_writer: EventWriter<ResetEvent>,
) {
    let msgs: Vec<&StartMenuMessage> = messages
        .filter_map(|val| match val {
            UiMessage::StartMenuMessage(message) => Some(message),
            _ => None,
        })
        .collect();
    for msg in msgs {
        match msg {
            StartMenuMessage::ChoosePlayerCount(str_count) => {
                state.player_count_input = str_count.to_string();
                let parsed_count = str_count.parse::<u32>();
                if let Ok(count) = parsed_count {
                    if count > 0 && count < 20
                    /*TODO move this*/
                    {
                        state.player_count = count;
                        state.player_count_parse_error = false;
                    } else {
                        state.player_count_parse_error = true;
                    }
                }
            }
            StartMenuMessage::Start => {
                state.mode = GameMode::Battle;
                reset_writer.send(ResetEvent {});
            }
        }
    }
}

pub fn view_startmenu_ui(state: Res<GameState>, mut ctx: IcedContext<UiMessage>) {
    let wrap = UiMessage::StartMenuMessage;
    let title = text("Tankrs");
    let start_button = button("Start").on_press_maybe(if state.player_count_parse_error {
        None
    } else {
        Some(UiMessage::SetSceneMessage(GameMode::Battle))
    });
    let input = text_input("Player Count", &state.player_count_input)
        .on_input(|count| wrap(StartMenuMessage::ChoosePlayerCount(count)));
    let content_container =
        container(column![title, row![input, start_button].spacing(5)].spacing(10))
            .width(300)
            .height(600)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);
    ctx.display(
        container(content_container)
            .padding(10)
            .width(5000)
            .height(5000)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(get_custom_container_style()),
    )
}
