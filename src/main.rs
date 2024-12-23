use bevy::{
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
    utils::HashMap,
};
use core::f32;
use oxiced::theme::get_theme;

use bevy_iced::{
    iced::{self, Style},
    IcedContext, IcedPlugin, IcedSettings,
};
use bullets::{BulletCount, BulletEntity, BulletInfo, BulletType};
use inputs::handle_keypress;
use tank::{Tank, TankBundle, Turret, TurretBundle};
use ui::{
    battle::{update_battle_ui, view_battle_ui, BattleMessage},
    shop::{update_shop_ui, view_shop_ui, ShopMessage},
    startmenu::{update_startmenu_ui, view_startmenu_ui, StartMenuMessage},
};
use utils::{
    get_current_player_props, next_random, polynomial, BulletHelpers, EndTurnEvent, FireEvent,
    GameMode, GameState, Player, PlayerKillEvent, RedrawTerrainEvent, ResetEvent, TurretMoveEvent,
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
        .add_event::<TurretMoveEvent>()
        .add_event::<FireEvent>()
        .add_event::<EndTurnEvent>()
        .add_event::<ResetEvent>()
        .add_event::<RedrawTerrainEvent>()
        .add_event::<PlayerKillEvent>()
        .insert_resource(IcedSettings {
            scale_factor: None,
            theme: get_theme(),
            style: Style {
                text_color: iced::Color::from_rgb(1.0, 0.0, 0.5),
            },
        })
        .insert_resource::<GameState>(GameState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update_ui)
        .add_systems(Update, reset_players)
        .add_systems(Update, view_ui)
        .add_systems(Update, collision_handler)
        .add_systems(Update, bullet_collision_wrapper)
        .add_systems(Update, gravity)
        .add_systems(Update, move_bullets)
        .add_systems(Update, move_turret_handler)
        .add_systems(Update, swap_player)
        .add_systems(Update, handle_keypress)
        .add_systems(Update, kill_handler)
        .add_systems(Update, redraw_terrain)
        .add_systems(Update, animate_sprite)
        .run();
}

#[derive(Component)]
struct Wall {}

#[allow(clippy::too_many_arguments)]
pub fn update_ui(
    mut messages: EventReader<UiMessage>,
    time: Res<Time>,
    commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
    mut state: ResMut<GameState>,
    mut reset_writer: EventWriter<ResetEvent>,
    end_turn_writer: EventWriter<EndTurnEvent>,
    turret_move_writer: EventWriter<TurretMoveEvent>,
    asset_server: Res<AssetServer>,
    atlas: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut new_messages = messages.read().peekable();
    if let Some(UiMessage::SetSceneMessage(mode)) = new_messages.peek() {
        reset_writer.send(ResetEvent {});
        state.mode = *mode;
    }

    match state.mode {
        utils::GameMode::Battle => update_battle_ui(
            new_messages,
            time,
            commands,
            materials,
            meshes,
            query,
            state,
            reset_writer,
            turret_move_writer,
            asset_server,
            atlas,
        ),
        utils::GameMode::Shop => update_shop_ui(new_messages, state, query, end_turn_writer),
        utils::GameMode::StartMenu => update_startmenu_ui(new_messages, state, reset_writer),
    }
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (entity, indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = if atlas.index == indices.last {
                    commands.entity(entity).despawn_recursive();
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

pub fn view_ui(
    state: Res<GameState>,
    player_query: Query<(&Player, &Tank)>,
    ctx: IcedContext<UiMessage>,
) {
    match state.mode {
        utils::GameMode::Battle => view_battle_ui(state, player_query, ctx),
        utils::GameMode::Shop => view_shop_ui(state, player_query, ctx),
        utils::GameMode::StartMenu => view_startmenu_ui(state, ctx),
    }
}

fn setup(mut commands: Commands, mut writer: EventWriter<ResetEvent>) {
    commands.spawn(Camera2d);
    writer.send(ResetEvent {});
}

fn redraw_terrain(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    state: Res<GameState>,
    mut commands: Commands,
    walls: Query<(Entity, &Wall)>,
    mut reader: EventReader<RedrawTerrainEvent>,
) {
    for _ in reader.read() {
        for (entity, _) in walls.iter() {
            commands.entity(entity).despawn_recursive();
        }
        let mut vertices = Vec::new();
        let mut i = -960;
        // TODO make this use a proper curve
        for _ in -960..960 {
            vertices.push([i as f32, 0.0, 0.0]);
            let two = [i as f32, polynomial(i, &state), 0.0];
            let three = [(i + 1) as f32, 0.0, 0.0];
            vertices.push(two);
            vertices.push(three);
            vertices.push(three);
            vertices.push(two);
            vertices.push([(i + 1) as f32, polynomial(i + 1, &state), 0.0]);
            i += 1;
        }
        let poly = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

        commands.spawn((
            Mesh2d(meshes.add(poly)),
            MeshMaterial2d(materials.add(Color::BLACK)),
            Transform {
                translation: Vec3 {
                    x: 0.0,
                    y: -540.0,
                    z: 0.0,
                },
                ..default()
            },
            Wall {},
        ));
    }
}

fn reset_players(
    mut state: ResMut<GameState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Player)>,
    mut reader: EventReader<ResetEvent>,
    mut redraw_writer: EventWriter<RedrawTerrainEvent>,
) {
    if reader.read().next().is_some() {
        let (wind, poly_rand) = next_random();
        state.wind = wind;
        state.rand = poly_rand;
        state.active_player = 0;
        state.damage = [0.0; 1921];
        state.players.clear();

        let mut previous_player_states = Vec::<(u32, HashMap<BulletType, BulletCount>)>::new();
        for (entity, player) in query.iter() {
            previous_player_states.push((player.money, player.inventory.clone()));
            commands.entity(entity).despawn_recursive();
        }
        for i in 0..state.set_player_count {
            let tank = Tank::default();
            let angle = tank.shooting_direction;
            let x_cord = -520.0 * state.rand + i as f32 * state.rand * 300.0;
            state.players.insert(i, true);
            let transform = Transform {
                scale: Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 1.0,
                },
                translation: Vec3 {
                    x: x_cord,
                    y: polynomial(x_cord as i32, &state) - 520.0,
                    z: 1.0,
                },
                ..default()
            };

            commands
                .spawn((
                    TankBundle {
                        sprite: Sprite {
                            image: asset_server.load(Tank::sprite_str_for_index(i)),
                            ..default()
                        },
                        tank,
                        player: Player::from_previous_or_initial(
                            i,
                            previous_player_states.get(i as usize),
                        ),
                    },
                    transform,
                ))
                .with_children(|builder| {
                    builder.spawn((
                        TurretBundle {
                            sprite: Sprite {
                                image: asset_server.load("turret.png"),
                                ..default()
                            },
                            turret: Turret {},
                        },
                        Transform {
                            rotation: Quat::from_axis_angle(
                                Vec3 {
                                    x: 1.0,
                                    y: 1.0,
                                    z: 0.0,
                                },
                                angle.get(),
                            ),
                            translation: Vec3 {
                                x: 0.0,
                                y: 5.0,
                                z: -1.0,
                            },
                            ..default()
                        },
                    ));
                });
        }
        redraw_writer.send(RedrawTerrainEvent {});
    }
}

fn move_bullets(
    time: Res<Time>,
    state: Res<GameState>,
    mut query: Query<(&mut BulletEntity, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (mut bullet, mut transform) in &mut query {
        let wind = state.wind;

        // y
        transform.translation.y =
            transform.translation.y + bullet.velocity_shot.y + 0.5 * -5.0 * delta * delta;
        bullet.velocity_shot.y += delta * -5.0;

        // x
        transform.translation.x =
            transform.translation.x + bullet.velocity_shot.x + 0.5 * wind * delta * delta;
    }
}

fn gravity(state: Res<GameState>, mut query: Query<(&mut Tank, &mut Transform)>) {
    for (mut tank, mut transform) in &mut query {
        let min = polynomial(transform.translation.x as i32, &state) - 520.0;
        let diff = transform.translation.y - min;
        if diff > 20.0 {
            tank.fall_damage = (diff - 20.0).clamp(0.0, 1000.0) as u32;
        }
        transform.translation.y = (transform.translation.y - 9.81).clamp(min, 1000.0);
    }
}

fn collision_handler(
    mut commands: Commands,
    mut tanks: Query<(Entity, &mut Player, &mut Tank), Without<Wall>>,
    mut walls: Query<(&Wall, &mut Transform)>,
    mut state: ResMut<GameState>,
) {
    for (entity, mut player, mut tank) in &mut tanks {
        for (_, wall_transform) in &mut walls {
            let wall_y = wall_transform.translation.y;
            let wall_size = 5.0;
            let tank_size = 166.0 / 2.0;
            let min_y = wall_y + wall_size / 2.0 + tank_size;
            tank.blocked_direction.y = min_y;
        }
        player.health -= tank.fall_damage as i32;
        if player.health < 0 {
            commands.entity(entity).despawn_recursive();
            let player_num = state.active_player;
            state.players.insert(player_num, false);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn bullet_collision_wrapper(
    commands: Commands,
    state: ResMut<GameState>,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    bullets: Query<(Entity, &mut BulletEntity, &Transform, &BulletType)>,
    walls: Query<(&Wall, &Transform)>,
    query: Query<(Entity, &mut Player, &Tank, &Transform)>,
    writer: EventWriter<EndTurnEvent>,
    battle_writer: EventWriter<PlayerKillEvent>,
    redraw_writer: EventWriter<RedrawTerrainEvent>,
    atlas: ResMut<Assets<TextureAtlasLayout>>,
) {
    bullet_collision(
        commands,
        state,
        materials,
        meshes,
        asset_server,
        bullets,
        walls,
        query,
        writer,
        battle_writer,
        redraw_writer,
        atlas,
    )
}

fn move_turret_handler(
    state: ResMut<GameState>,
    mut reader: EventReader<TurretMoveEvent>,
    mut turrets: Query<(&Parent, &Turret, &mut Transform)>,
    mut query: Query<(Entity, &mut Player, &Tank)>,
) {
    for _ in reader.read() {
        for (tank_entity, player, tank) in &mut query {
            for (parent, _, mut turret_transform) in &mut turrets {
                if parent.get() == tank_entity && player.player_number == state.active_player {
                    turret_transform.rotation = Quat::from_axis_angle(
                        Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 1.0,
                        },
                        f32::consts::PI - tank.shooting_direction.get(),
                    )
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn bullet_collision<'w>(
    mut commands: Commands<'w, '_>,
    mut state: ResMut<'w, GameState>,
    mut materials: ResMut<'w, Assets<ColorMaterial>>,
    mut meshes: ResMut<'w, Assets<Mesh>>,
    asset_server: Res<'w, AssetServer>,
    bullets: Query<(Entity, &mut BulletEntity, &Transform, &BulletType)>,
    walls: Query<(&Wall, &Transform)>,
    mut query: Query<(Entity, &mut Player, &Tank, &Transform)>,
    mut writer: EventWriter<EndTurnEvent>,
    mut battle_writer: EventWriter<PlayerKillEvent>,
    mut redraw_writer: EventWriter<RedrawTerrainEvent>,
    mut atlas: ResMut<'w, Assets<TextureAtlasLayout>>,
) {
    for (bullet_entity, bullet, bullet_transform, bullet_type) in &bullets {
        let bullet_info = bullet_type.get_bullet_from_type();
        let mut hit = false;
        for (_, _) in &walls {
            if bullet_transform.translation.y
                < polynomial(bullet_transform.translation.x as i32, &state) - 540.0
            {
                hit = true;
                commands.entity(bullet_entity).despawn_recursive();
                // TODO handle this in a separate function -> add damage there
                let mut helpers = BulletHelpers {
                    commands: &mut commands,
                    state: &mut state,
                    meshes: &mut meshes,
                    materials: &mut materials,
                    assetserver: &asset_server,
                    atlas: &mut atlas,
                };
                let info = BulletInfo {
                    velocity: &Vec2 { x: 1.0, y: 1.0 },
                    origin: &bullet_transform.translation,
                    owner: bullet.owner,
                    radius: bullet.radius,
                    dmg: bullet.damage,
                };
                (bullet_info.groundhitfn)(&mut helpers, &mut writer, &info);
            }
            if hit {
                break;
            }
        }
        for (tank_entity, mut player, tank, tank_transform) in &mut query {
            if bullet_transform.translation.y <= tank_transform.translation.y + (tank.scale.y / 2.0)
                && bullet_transform.translation.y
                    >= tank_transform.translation.y - (tank.scale.y / 2.0)
                && bullet_transform.translation.x
                    <= tank_transform.translation.x + (tank.scale.x / 2.0)
                && bullet_transform.translation.x
                    >= tank_transform.translation.x - (tank.scale.x / 2.0)
            {
                hit = true;
                player.health -= bullet.damage as i32;
                if player.health < 0 {
                    state.players.insert(player.player_number, false);
                    battle_writer.send(PlayerKillEvent {
                        killer: bullet.owner,
                        killed: player.player_number,
                    });
                    commands.entity(tank_entity).despawn_recursive();
                }
                let mut helpers = BulletHelpers {
                    commands: &mut commands,
                    state: &mut state,
                    meshes: &mut meshes,
                    materials: &mut materials,
                    assetserver: &asset_server,
                    atlas: &mut atlas,
                };
                let info = BulletInfo {
                    velocity: &Vec2 { x: 1.0, y: 1.0 },
                    origin: &bullet_transform.translation,
                    owner: bullet.owner,
                    radius: bullet.radius,
                    dmg: bullet.damage,
                };
                (bullet_info.playerhitfn)(&mut helpers, &mut writer, &info);
            }
        }
        if hit {
            commands.entity(bullet_entity).despawn_recursive();
            redraw_writer.send(RedrawTerrainEvent {});
        }
    }
}

fn kill_handler(mut reader: EventReader<PlayerKillEvent>, mut players: Query<&mut Player>) {
    for event in reader.read() {
        if event.killer != event.killed {
            for mut player in &mut players {
                if player.player_number == event.killer {
                    player.money += 1000;
                }
            }
        }
    }
}

fn swap_player(
    mut state: ResMut<GameState>,
    mut reader: EventReader<EndTurnEvent>,
    mut ui_writer: EventWriter<UiMessage>,
    mut reset_writer: EventWriter<ResetEvent>,
    players: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
) {
    let mut last = false;
    if state.mode == GameMode::Battle && players.iter().len() < 2 {
        reset_writer.send(ResetEvent {});
        ui_writer.send(UiMessage::SetSceneMessage(GameMode::Shop));
        state
            .active_bullets
            .store(0, std::sync::atomic::Ordering::Relaxed);
        last = true;
    }
    for _ in reader.read() {
        let (wind, _) = next_random();
        state.wind = wind;
        if state.mode == GameMode::Shop && state.active_player == state.set_player_count - 1 {
            ui_writer.send(UiMessage::SetSceneMessage(GameMode::Battle));
        }
        if last {
            return;
        }
        state.increment_player();
    }
}
