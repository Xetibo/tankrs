use bevy::{
    asset::Assets,
    color::Color,
    math::{Quat, Vec2, Vec3},
    prelude::{default, Bundle, Circle, Commands, Component, Mesh, ResMut, Transform},
    sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::tank::Angle;

#[derive(Component)]
pub struct Bullet {
    pub velocity: Vec2,
    pub direction: Angle,
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
    &Angle,
    &Vec2,
) = |meshes: &mut ResMut<Assets<Mesh>>,
     materials: &mut ResMut<Assets<ColorMaterial>>,
     commands: &mut Commands,
     direction: &Angle,
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
