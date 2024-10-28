use bevy::{
    asset::Assets,
    color::Color,
    math::{Vec2, Vec3},
    prelude::{default, Bundle, Circle, Commands, Component, Mesh, ResMut, Transform},
    sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::tank::Angle;

#[derive(Component)]
pub struct BulletCollider {}

#[derive(Component)]
pub struct Bullet {
    pub velocity_shot: Vec2,
    pub velocity_gravity: Vec2,
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
    &Vec3,
) = |meshes: &mut ResMut<Assets<Mesh>>,
     materials: &mut ResMut<Assets<ColorMaterial>>,
     commands: &mut Commands,
     direction: &Angle,
     velocity: &Vec2,
     origin: &Vec3| {
    let offset_origin = Vec3 {
        x: origin.x,
        y: origin.y + 20.0,
        z: 0.0,
    };
    commands.spawn(BulletBundle {
        bullet: Bullet {
            velocity_shot: *velocity,
            velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
            direction: *direction,
        },
        mesh_bundle: MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle { radius: 1.0 })),
            material: materials.add(Color::BLACK),
            transform: Transform {
                translation: offset_origin,
                scale: Vec3 {
                    x: 10.0,
                    y: 10.0,
                    z: 1.0,
                },
                ..default()
            },
            ..default()
        },
    });
};
