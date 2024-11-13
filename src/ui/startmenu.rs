use bevy::{
    asset::Assets,
    prelude::{Commands, Entity, EventWriter, Mesh, Query, Res, ResMut, Transform},
    sprite::{ColorMaterial, Sprite},
};
use bevy_iced::{
    iced::widget::{row, text},
    IcedContext,
};

use crate::{
    tank::Tank,
    utils::{GameState, Player, ResetEvent},
    UiMessage,
};

#[derive(Clone, Copy)]
pub enum StartMenuMessage {
    ChoosePlayerCount(u32),
    Start,
}

pub fn update_startmenu_ui<'a>(
    messages: impl Iterator<Item = &'a UiMessage>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
    mut state: ResMut<GameState>,
    mut reset_writer: EventWriter<ResetEvent>,
) {
}

pub fn view_startmenu_ui(
    state: Res<GameState>,
    player_query: Query<(&Player, &Tank)>,
    mut ctx: IcedContext<UiMessage>,
) {
    ctx.display(row![text("Hello")])
}
