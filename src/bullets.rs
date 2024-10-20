use bevy::{
    asset::Assets,
    color::Color,
    math::{Quat, Rect, Vec2, Vec3},
    prelude::{default, Bundle, Circle, Commands, Component, Mesh, ResMut, Transform},
    sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle},
};

#[derive(Component)]
pub struct Bullet {
    pub velocity: Vec2,
    pub direction: Vec2,
}

#[derive(Bundle)]
pub struct BulletBundle {
    pub bullet: Bullet,
    pub mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
}

pub const NORMAL_BULLET: fn(
    &mut ResMut<Assets<Mesh>>,
    &mut ResMut<Assets<ColorMaterial>>,
    &mut Commands,
    &Vec2,
    &Vec2,
) = |meshes: &mut ResMut<Assets<Mesh>>,
     materials: &mut ResMut<Assets<ColorMaterial>>,
     commands: &mut Commands,
     direction: &Vec2,
     velocity: &Vec2| {
    commands.spawn(BulletBundle {
        bullet: Bullet {
            velocity: *velocity,
            direction: *direction,
        },
        mesh_bundle: MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle { radius: 1.0 })),
            material: materials.add(Color::BLACK),
            transform: Transform {
                translation: Vec3 {
                    x: 100.0,
                    y: 100.0,
                    z: 0.0,
                },
                rotation: Quat::IDENTITY,
                scale: Vec3 {
                    x: 10.0,
                    y: 10.0,
                    z: 1.0,
                },
            },
            ..default()
        },
    });
};
