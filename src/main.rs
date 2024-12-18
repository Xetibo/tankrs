use bevy::{
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
    utils::HashMap,
};
use core::f32;

use bevy_iced::{IcedContext, IcedPlugin};
use bullets::{BulletCount, BulletEntity, BulletType};
use inputs::handle_keypress;
use tank::{Tank, TankBundle};
use ui::{
    battle::{update_battle_ui, view_battle_ui, BattleMessage},
    shop::{update_shop_ui, view_shop_ui, ShopMessage},
    startmenu::{update_startmenu_ui, view_startmenu_ui, StartMenuMessage},
};
use utils::{
    get_current_player_props, next_random, polynomial, EndTurnEvent, FireEvent, GameMode,
    GameState, Player, PlayerKillEvent, RedrawTerrainEvent, ResetEvent,
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
        .add_event::<RedrawTerrainEvent>()
        .add_event::<PlayerKillEvent>()
        .insert_resource::<GameState>(GameState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update_ui)
        .add_systems(Update, reset_players)
        .add_systems(Update, view_ui)
        .add_systems(Update, collision_handler)
        .add_systems(Update, bullet_collision)
        //.add_systems(Update, gravity)
        .add_systems(Update, move_bullets)
        .add_systems(Update, swap_player)
        .add_systems(Update, handle_keypress)
        .add_systems(Update, kill_handler)
        .add_systems(Update, redraw_terrain)
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
    reset_writer: EventWriter<ResetEvent>,
    end_turn_writer: EventWriter<EndTurnEvent>,
    asset_server: Res<AssetServer>,
) {
    let mut new_messages = messages.read().peekable();
    if let Some(UiMessage::SetSceneMessage(mode)) = new_messages.peek() {
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
            asset_server,
        ),
        utils::GameMode::Shop => update_shop_ui(new_messages, state, query, end_turn_writer),
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
        utils::GameMode::Shop => view_shop_ui(state, player_query, ctx),
        utils::GameMode::StartMenu => view_startmenu_ui(state, ctx),
    }
}

fn setup(mut writer: EventWriter<ResetEvent>) {
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
        let mut i = -1920;
        // TODO make this use a proper curve
        for _ in -1920..1920 {
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

        //poly.insert_indices(Indices::U32(indices));
        commands.spawn(Camera2d);

        commands.spawn((
            Mesh2d(meshes.add(poly)),
            MeshMaterial2d(materials.add(Color::BLACK)),
            Transform {
                translation: Vec3 {
                    x: 0.0,
                    y: -720.0,
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

        let mut previous_player_states = Vec::<(u32, HashMap<BulletType, BulletCount>)>::new();
        for (entity, player) in query.iter() {
            previous_player_states.push((player.money, player.inventory.clone()));
            commands.entity(entity).despawn_recursive();
        }
        for i in 0..state.player_count {
            let x_cord = -700.0 * state.rand + i as f32 * state.rand * 1050.0;
            commands.spawn((
                TankBundle {
                    sprite: Sprite {
                        image: asset_server.load("greentank_rechts.png"),
                        ..default()
                    },
                    tank: Tank::default(),
                    player: Player::from_previous_or_initial(
                        i,
                        previous_player_states.get(i as usize),
                    ),
                },
                Transform {
                    scale: Vec3 {
                        x: 0.3333,
                        y: 0.3333,
                        z: 1.0,
                    },
                    translation: Vec3 {
                        x: x_cord,
                        y: polynomial(x_cord as i32, &state) - 695.0,
                        z: 1.0,
                    },
                    ..default()
                },
            ));
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

        // s0 + v0 + 0.5 * a * t * t
        // calculate next positions

        // y
        transform.translation.y =
            transform.translation.y + bullet.velocity_shot.y + 0.5 * -5.0 * delta * delta;
        bullet.velocity_shot.y += delta * -5.0;

        // x
        transform.translation.x =
            transform.translation.x + bullet.velocity_shot.x + 0.5 * wind * delta * delta;
        //if bullet.velocity_shot.x > 0.0 {
        //    bullet.velocity_shot.x = (/*delta * wind +*/bullet.velocity_shot.x).clamp(0.0, 1000.0);
        //} else {
        //    bullet.velocity_shot.x = (/*delta * wind +*/bullet.velocity_shot.x).clamp(-1000.0, 0.0);
        //}

        // TODO do we want air resistance?
        //if bullet.velocity_shot.x > 0.0 {
        //transform.translation.x =
        //    transform.translation.x + bullet.velocity_shot.x + 0.5 * wind * delta * delta;
        //bullet.velocity_shot.x = (delta * wind + bullet.velocity_shot.x).clamp(0.0, 1000.0);
        //} else {
        //    transform.translation.x = transform.translation.x
        //        + bullet.velocity_shot.x
        //        + 0.5 * 0.1/*TODO implement wind*/ * delta * delta;
        //    bullet.velocity_shot.x = (delta * 0.1 + bullet.velocity_shot.x).clamp(-1000.0, 0.0);
        //}
    }
}

//fn gravity(mut query: Query<(&Tank, &mut Transform)>) {
//    for (_, mut transform) in &mut query {
//        transform.translation.y = (transform.translation.y - 9.81).clamp(
//            polynomial(transform.translation.x as i32, 0.5) - 550.0,
//            1000.0,
//        );
//    }
//}

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
    bullets: Query<(Entity, &mut BulletEntity, &Transform, &BulletType)>,
    walls: Query<(&Wall, &Transform)>,
    mut query: Query<(Entity, &mut Player, &Tank, &Transform)>,
    mut writer: EventWriter<EndTurnEvent>,
    mut battle_writer: EventWriter<PlayerKillEvent>,
    mut redraw_writer: EventWriter<RedrawTerrainEvent>,
) {
    for (bullet_entity, bullet, bullet_transform, bullet_type) in &bullets {
        let bullet_info = bullet_type.get_bullet_from_type();
        let mut hit = false;
        for (_, _) in &walls {
            if bullet_transform.translation.y
                < polynomial(bullet_transform.translation.x as i32, &state) - 720.0
            {
                hit = true;
                commands.entity(bullet_entity).despawn_recursive();
                // TODO handle this in a separate function -> add damage there
                (bullet_info.groundhitfn)(&mut commands, &mut state, &mut writer);
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
                    battle_writer.send(PlayerKillEvent {
                        killer: bullet.owner,
                        killed: player.player_number,
                    });
                    commands.entity(tank_entity).despawn_recursive();
                }
                (bullet_info.playerhitfn)(&mut commands, &mut state, &mut writer);
                commands.entity(bullet_entity).despawn_recursive();
            }
        }
        if hit {
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
    mut players: Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
) {
    if state.mode == GameMode::Battle && players.iter().len() < 2 {
        ui_writer.send(UiMessage::SetSceneMessage(GameMode::Shop));
        reset_writer.send(ResetEvent {});
        state.firing = false;
    }
    for _ in reader.read() {
        let (wind, _) = next_random();
        state.wind = wind;
        let (_, player, _, _, _) =
            if let Some(props) = get_current_player_props(state.active_player, &mut players) {
                props
            } else {
                return;
            };
        if state.mode == GameMode::Shop && player.player_number == state.player_count - 1 {
            ui_writer.send(UiMessage::SetSceneMessage(GameMode::Battle));
        }
        state.increment_player();
    }
}
