use bevy::prelude::*;
use space_editor::prelude::{PrefabBundle, PrefabPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PrefabPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(PrefabBundle::new("cube.scn.ron"))
        .insert(Name::new("Prefab"));
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load("gg.png"),
        ..default()
    });
}
