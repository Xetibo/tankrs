use bevy::prelude::*;
use game_lib::GamePlugin;
use space_editor::prelude::{PrefabBundle, PrefabPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((PrefabPlugin, GamePlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_keypress)
        .run();
}

#[derive(Component)]
struct GG {}

#[derive(Bundle)]
struct Fuck {
    sprite: SpriteBundle,
    gg: GG,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(PrefabBundle::new("scenes/cube.scn.ron"))
        .insert(Name::new("Prefab"));
    commands.spawn(Camera2dBundle::default());

    commands.spawn(Fuck {
        sprite: SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                flip_x: false,
                flip_y: false,
                custom_size: Some(Vec2::new(30.0, 30.0)),
                rect: Some(Rect {
                    min: Vec2::new(30.0, 30.0),
                    max: Vec2::new(30.0, 30.0),
                }),
                anchor: bevy::sprite::Anchor::Center,
            },
            ..default()
        },
        gg: GG {},
    });
    //commands.spawn(SpriteBundle {
    //    texture: asset_server.load("gg.png"),
    //    ..default()
    //});
}

fn handle_keypress(mut query: Query<(&GG, &mut Transform)>, keys: Res<ButtonInput<KeyCode>>) {
    for (tank, mut transform) in &mut query {
        if keys.pressed(KeyCode::ArrowRight) {
            transform.translation.x += 10.0;
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= 10.0;
        }
    }
}
