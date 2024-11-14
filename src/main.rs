use core::f32;

use bevy::{
    math::Vec2,
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use bevy_iced::{IcedContext, IcedPlugin};
use bullets::{Bullet, BulletType, NORMAL_BULLET};
use inputs::{handle_keypress, KeyMap};
use tank::{Tank, TankBundle};
use ui::{
    battle::{update_battle_ui, view_battle_ui, BattleMessage},
    shop::{update_shop_ui, view_shop_ui, ShopMessage},
    startmenu::{update_startmenu_ui, view_startmenu_ui, StartMenuMessage},
};
use utils::{
    get_current_player_props, polynomial, EndTurnEvent, FireEvent, GameMode, GameState, Player,
    ResetEvent,
};

pub mod bullets;
pub mod inputs;
pub mod tank;
pub mod ui;
pub mod utils;

#[derive(Event, Clone)]
pub enum UiMessage {
    StartMenuMessage(StartMenuMessage),
    BattleMessage(BattleMessage),
    ShopMessage(ShopMessage),
    SetSceneMessage(GameMode),
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(IcedPlugin::default())
        .add_event::<UiMessage>()
        .add_event::<FireEvent>()
        .add_event::<EndTurnEvent>()
        .add_event::<ResetEvent>()
        .insert_resource::<GameState>(GameState {
            firing: false,
            mode: GameMode::StartMenu,
            player_count_input: "2".into(),
            player_count: 2,
            player_count_parse_error: false,
            wind: 0,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_ui)
        .add_systems(Update, reset_players)
        .add_systems(Update, view_ui)
        .add_systems(Update, collision_handler)
        .add_systems(Update, bullet_collision)
        .add_systems(Update, gravity)
        .add_systems(Update, move_bullets)
        .add_systems(Update, swap_player)
        .add_systems(Update, handle_keypress)
        .run();
}

#[derive(Component)]
struct Wall {}

#[allow(clippy::too_many_arguments)]
pub fn update_ui(
    mut messages: EventReader<UiMessage>,
    commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
    mut state: ResMut<GameState>,
    reset_writer: EventWriter<ResetEvent>,
    asset_server: Res<AssetServer>,
) {
    let mut new_messages = messages.read().peekable();
    if let Some(UiMessage::SetSceneMessage(mode)) = new_messages.peek() {
        state.mode = *mode;
    }

    match state.mode {
        utils::GameMode::Battle => update_battle_ui(
            new_messages,
            commands,
            materials,
            meshes,
            query,
            state,
            reset_writer,
            asset_server,
        ),
        utils::GameMode::Shop => update_shop_ui(new_messages, query),
        utils::GameMode::StartMenu => update_startmenu_ui(new_messages, state, reset_writer),
    }
}

pub fn view_ui(
    state: Res<GameState>,
    player_query: Query<(&Player, &Tank)>,
    ctx: IcedContext<UiMessage>,
) {
    match state.mode {
        utils::GameMode::Battle => view_battle_ui(state, player_query, ctx),
        utils::GameMode::Shop => view_shop_ui(player_query, ctx),
        utils::GameMode::StartMenu => view_startmenu_ui(state, ctx),
    }
}

fn setup(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut writer: EventWriter<ResetEvent>,
) {
    //let rand: f32 = rand::random::<f32>().clamp(0.0, 5.0);
    let rand: f32 = 0.5;
    let mut vertices = Vec::new();
    let mut i = -1920;
    //for _ in -1000..1000 {
    for _ in -1920..1920 {
        vertices.push([i as f32, 0.0, 0.0]);
        let two = [i as f32, polynomial(i, rand), 0.0];
        let three = [(i + 1) as f32, 0.0, 0.0];
        vertices.push(two);
        vertices.push(three);
        vertices.push(three);
        vertices.push(two);
        vertices.push([(i + 1) as f32, polynomial(i + 1, rand), 0.0]);
        //    x: i as f32,
        //    y: 0.0,
        //    z: 0.0,
        //});
        //let two = Vec3 {
        //    x: i as f32,
        //    y: polynomial(i, rand),
        //    z: 0.0,
        //};
        //vertices.push(two);
        //let three = Vec3 {
        //    x: (i + 1) as f32,
        //    y: 0.0,
        //    z: 0.0,
        //};
        //vertices.push(three);
        //vertices.push(three);
        //vertices.push(two);
        //vertices.push(Vec3 {
        //    x: (i + 1) as f32,
        //    y: polynomial(i + 1, rand),
        //    z: 0.0,
        //});
        i += 1;
    }
    //let mut indices = Vec::new();
    //for i in 0..(6 * 10) {
    //    indices.push(i);
    //}
    let poly = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    //poly.insert_indices(Indices::U32(indices));
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(poly)),
            material: materials.add(Color::BLACK),
            transform: Transform {
                translation: Vec3 {
                    x: 0.0,
                    y: -650.0,
                    z: 0.0,
                },
                ..default()
            },
            ..default()
        },
        Wall {},
    ));
    //commands.spawn((
    //    SpriteBundle {
    //        sprite: Sprite {
    //            rect: Some(Rect {
    //                min: Vec2::new(-2000.0, 0.0),
    //                max: Vec2::new(2000.0, 10.0),
    //            }),
    //            color: Color::BLACK,
    //            ..default()
    //        },
    //        transform: Transform {
    //            translation: Vec3 {
    //                x: 0.0,
    //                y: -400.0,
    //                z: 0.0,
    //            },
    //            ..default()
    //        },
    //        ..default()
    //    },
    //    Wall {},
    //));
    //generate_terrain(materials, meshes, commands);
    writer.send(ResetEvent {});
}

fn reset_players(
    state: Res<GameState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Player)>,
    mut reader: EventReader<ResetEvent>,
) {
    if reader.read().next().is_some() {
        for (entity, _) in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for i in 0..state.player_count {
            commands.spawn(TankBundle {
                sprite: SpriteBundle {
                    texture: asset_server.load("greentank_rechts.png"),
                    transform: Transform {
                        scale: Vec3 {
                            x: 0.3333,
                            y: 0.3333,
                            z: 1.0,
                        },
                        translation: Vec3 {
                            x: -200.0 + i as f32 * 150.0,
                            y: -300.0,
                            z: 1.0,
                        },
                        ..default()
                    },
                    ..default()
                },
                tank: Tank {
                    blocked_direction: Vec2::default(),
                    scale: Vec3 {
                        x: 100.0,
                        y: 10.0,
                        z: 0.0,
                    },
                    // top right
                    shooting_direction: tank::Angle::default(),
                    shooting_velocity: Vec2::new(100.0, 600.0),
                },
                player: Player {
                    player_number: i,
                    inventory: BulletType::init_bullets(),
                    health: 100,
                    fuel: 100,
                    money: 0,
                    key_map: KeyMap::default_keymap(),
                    selected_bullet: (BulletType::RegularBullet, NORMAL_BULLET),
                    is_active: i == 0,
                    fire_velocity: 0,
                },
            });
        }
    }
}

fn move_bullets(time: Res<Time>, mut query: Query<(&mut Bullet, &mut Transform)>) {
    for (mut bullet, mut transform) in &mut query {
        // TODO move this away from here -> calculated every frame for no reason
        let direction = bullet.direction.get();
        let direction_y = direction.sin();
        let direction_x = direction.cos();

        // calculate next positions
        transform.translation.x += bullet.velocity_shot.x * direction_x * time.delta_seconds();
        transform.translation.y += bullet.velocity_shot.y * direction_y * time.delta_seconds()
            + bullet.velocity_gravity.y * -1.0 * time.delta_seconds();

        // calculate new velocities
        bullet.velocity_shot.y += time.delta_seconds() * -50.0;
        bullet.velocity_gravity.y += time.delta_seconds() * 50.0;
        if bullet.velocity_shot.x > 0.0 {
            bullet.velocity_shot.x =
                (time.delta_seconds() * -10.0 + bullet.velocity_shot.x).clamp(0.0, 1000.0);
        } else {
            bullet.velocity_shot.x =
                (time.delta_seconds() * 10.0 + bullet.velocity_shot.x).clamp(-1000.0, 0.0);
        }
    }
}

fn gravity(mut query: Query<(&Tank, &mut Transform)>) {
    for (_, mut transform) in &mut query {
        transform.translation.y = (transform.translation.y - 9.81).clamp(
            polynomial(transform.translation.x as i32, 0.5) - 550.0,
            1000.0,
        );
    }
}

fn collision_handler(
    mut tanks: Query<&mut Tank, Without<Wall>>,
    mut walls: Query<(&Wall, &mut Transform)>,
) {
    for mut tank in &mut tanks {
        for (_, wall_transform) in &mut walls {
            let wall_y = wall_transform.translation.y;
            let wall_size = 5.0;
            let tank_size = 166.0 / 2.0;
            let min_y = wall_y + wall_size / 2.0 + tank_size;

            tank.blocked_direction.y = min_y;
        }
    }
}

fn bullet_collision(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    bullets: Query<(Entity, &mut Bullet, &Transform)>,
    walls: Query<(&Wall, &Transform)>,
    tanks: Query<(Entity, &Tank, &Transform)>,
    mut writer: EventWriter<EndTurnEvent>,
) {
    if bullets.iter().len() == 0 {
        state.firing = false;
        if state.firing {
            writer.send(EndTurnEvent {});
            return;
        }
    }
    for (entity, _, bullet_transform) in &bullets {
        for (_, _) in &walls {
            if bullet_transform.translation.y
                < polynomial(bullet_transform.translation.x as i32, 0.5) - 650.0
            {
                commands.entity(entity).despawn_recursive();
            }
        }
        for (tank_entity, tank, tank_transform) in &tanks {
            if bullet_transform.translation.y <= tank_transform.translation.y + (tank.scale.y / 2.0)
                && bullet_transform.translation.y
                    >= tank_transform.translation.y - (tank.scale.y / 2.0)
                && bullet_transform.translation.x
                    <= tank_transform.translation.x + (tank.scale.x / 2.0)
                && bullet_transform.translation.x
                    >= tank_transform.translation.x - (tank.scale.x / 2.0)
            {
                commands.entity(tank_entity).despawn_recursive();
            }
        }
    }
}

fn swap_player(
    state: Res<GameState>,
    mut reader: EventReader<EndTurnEvent>,
    mut players: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
) {
    for _ in reader.read() {
        let (_, mut player, _, _, _) = if let Some(props) = get_current_player_props(&mut players) {
            props
        } else {
            return;
        };
        player.is_active = false;
        let is_highest = player.player_number == state.player_count - 1;
        let previous = player.player_number;
        for (_, mut player, _, _, _) in &mut players {
            if is_highest && player.player_number == 0 || player.player_number == previous + 1 {
                player.is_active = true;
            }
        }
    }
}
