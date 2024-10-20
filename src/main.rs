use bevy::prelude::*;
use bullets::{Bullet, NORMAL_BULLET};
use tank::{Tank, TankBundle};

pub mod bullets;
pub mod tank;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_keypress)
        .add_systems(Update, collision_handler)
        .add_systems(Update, gravity)
        .add_systems(Update, move_bullets)
        .run();
}

#[derive(Component)]
struct Wall {}

fn setup(
    mut _materials: ResMut<Assets<StandardMaterial>>,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TankBundle {
        sprite: SpriteBundle {
            texture: asset_server.load("greentank_rechts.png"),
            ..default()
        },
        tank: Tank {
            blocked_direction: Vec2::default(),
            // top right
            shooting_direction: Vec2::new(1.0, 1.0),
            shooting_velocity: Vec2::new(30.0, 250.0),
        },
    });
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                rect: Some(Rect {
                    min: Vec2::new(-2000.0, 0.0),
                    max: Vec2::new(2000.0, 10.0),
                }),
                color: Color::BLACK,
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    x: 0.0,
                    y: -400.0,
                    z: 0.0,
                },
                ..default()
            },
            ..default()
        },
        Wall {},
    ));
    //generate_terrain(materials, meshes, commands);
}

fn handle_keypress(
    mut query: Query<(&mut Tank, &mut Transform, &mut Sprite)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (mut tank, mut transform, mut sprite) in &mut query {
        if keys.pressed(KeyCode::ArrowRight) {
            transform.translation.x += 10.0;
            sprite.flip_x = false;
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= 10.0;
            sprite.flip_x = true;
        }
        if keys.pressed(KeyCode::ArrowUp) {
            transform.translation.y =
                (transform.translation.y + 10.0).clamp(tank.blocked_direction.y, 1000.0);
        }
        if keys.pressed(KeyCode::ArrowDown) {
            transform.translation.y =
                (transform.translation.y - 10.0).clamp(tank.blocked_direction.y, 1000.0);
        }
        if keys.pressed(KeyCode::KeyA) {
            tank.shooting_direction.x -= 1.0;
            tank.shooting_direction.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            tank.shooting_direction.x += 1.0;
            tank.shooting_direction.y += 1.0;
        }
        if keys.pressed(KeyCode::Space) {
            (NORMAL_BULLET)(
                &mut meshes,
                &mut materials,
                &mut commands,
                &tank.shooting_direction,
                &tank.shooting_velocity,
            );
        }
    }
}

fn move_bullets(time: Res<Time>, mut query: Query<(&mut Bullet, &mut Transform)>) {
    for (mut bullet, mut transform) in &mut query {
        // i mean ofc, if you can't understand velocity anymore..
        transform.translation.x += bullet.velocity.x * bullet.direction.x * time.delta_seconds();
        // TODO make this use actual acceleration
        let grav = 2.0;

        transform.translation.y += bullet.velocity.y * bullet.direction.y * time.delta_seconds();
        bullet.velocity.y = time.delta_seconds() * -80.0 + bullet.velocity.y;
        if transform.translation.x > 0.0 {
            bullet.velocity.x =
                (time.delta_seconds() * -1.0 + bullet.velocity.x).clamp(0.0, 1000.0);
        } else {
            bullet.velocity.x =
                (time.delta_seconds() * 1.0 + bullet.velocity.x).clamp(-1000.0, 0.0);
        }
    }
}

fn generate_random(min: i32, max: i32) -> i32 {
    rand::random::<i32>().clamp(min, max)
}

fn gravity(mut query: Query<(&Tank, &mut Transform)>) {
    for (tank, mut transform) in &mut query {
        transform.translation.y =
            (transform.translation.y - 9.81).clamp(tank.blocked_direction.y, 1000.0);
    }
}

fn collision_handler(
    mut tanks: Query<(&mut Tank, &mut Transform), Without<Wall>>,
    mut walls: Query<(&Wall, &mut Transform)>,
) {
    for (mut tank, mut tank_transform) in &mut tanks {
        for (_, mut wall_transform) in &mut walls {
            let wall_y = wall_transform.translation.y;
            let wall_size = 5.0;
            let tank_size = 166.0 / 2.0;
            let min_y = wall_y + wall_size / 2.0 + tank_size;

            tank.blocked_direction.y = min_y;
        }
    }
}

fn generate_terrain(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    //let mut initial = generate_random(5, 10);
    //for i in 0..10 {
    //    let shape = Mesh2dHandle(meshes.add(Triangle2d::new(
    //        Vec2::Y * 10.0 * initial as f32,
    //        Vec2::new(-50.0, -50.0),
    //        Vec2::new(50.0, -50.0),
    //    )));
    //    commands.spawn(MaterialMesh2dBundle {
    //        mesh: shape,
    //        material: materials.add(Color::BLACK),
    //        transform: Transform::from_xyz(
    //            // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
    //            i as f32 * 100.0,
    //            0.0,
    //            0.0,
    //        ),
    //        ..default()
    //    });
    //    initial = generate_random(initial - 2, initial + 2);
    //}
}
