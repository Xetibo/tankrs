use std::{fmt::Display, hash::Hash};

use bevy::{
    asset::{AssetServer, Assets},
    color::Color,
    math::{Vec2, Vec3},
    prelude::{default, Bundle, Circle, Commands, Component, Mesh, Res, ResMut, Transform},
    sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle, SpriteBundle},
    utils::HashMap,
};
use enum_iterator::Sequence;

use crate::{tank::Angle, utils::BulletFn};

#[derive(Component)]
pub struct BulletCollider {}

#[derive(Component, Eq, PartialEq, Clone, Copy, Sequence)]
pub enum BulletType {
    RegularBullet,
    FireBullet,
    Nuke,
}

impl Display for BulletType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = match self {
            BulletType::RegularBullet => "RegularBullet",
            BulletType::FireBullet => "FireBullet",
            BulletType::Nuke => "Nuke",
        };
        f.write_str(type_str)
    }
}

impl Hash for BulletType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let value = self.get_int_value();
        state.write(&value.to_le_bytes())
    }
}

impl BulletType {
    pub fn get_int_value(&self) -> u32 {
        match self {
            BulletType::RegularBullet => 0,
            BulletType::FireBullet => 1,
            BulletType::Nuke => 2,
        }
    }

    pub fn get_from_int(value: u32) -> BulletType {
        match value {
            0 => BulletType::RegularBullet,
            1 => BulletType::FireBullet,
            2 => BulletType::Nuke,
            _ => panic!("Bullet with this ID doesn't exist"),
        }
    }

    pub fn get_bullet_from_type(&self) -> BulletFn {
        match self {
            BulletType::RegularBullet => NORMAL_BULLET,
            BulletType::FireBullet => FIRE_BULLET,
            BulletType::Nuke => NUKE,
        }
    }

    pub fn init_bullets() -> HashMap<BulletType, BulletCount> {
        let mut map = HashMap::new();
        map.insert(BulletType::RegularBullet, BulletCount::Unlimited);
        // TODO remove
        map.insert(BulletType::FireBullet, BulletCount::Count(10));
        map.insert(BulletType::Nuke, BulletCount::Count(10));
        map
    }
}

#[derive(Component, Clone)]
pub enum BulletCount {
    Unlimited,
    Count(u32),
}

#[derive(Component)]
pub struct Bullet {
    pub velocity_shot: Vec2,
    pub velocity_gravity: Vec2,
    pub direction: Angle,
    pub damage: u32,
    pub radius: u32,
}

#[derive(Bundle)]
pub struct BulletMeshBundle {
    pub bullet: Bullet,
    pub mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Bundle)]
pub struct BulletSpriteBundle {
    pub bullet: Bullet,
    pub sprite_bundle: SpriteBundle,
}

pub struct BulletInfo<'a> {
    pub direction: &'a Angle,
    pub velocity: &'a Vec2,
    pub origin: &'a Vec3,
}

impl<'a> BulletInfo<'a> {
    pub fn new(direction: &'a Angle, velocity: &'a Vec2, origin: &'a Vec3) -> Self {
        Self {
            direction,
            velocity,
            origin,
        }
    }
}

pub const NORMAL_BULLET: BulletFn = |commands: &mut Commands,
                                     meshes: &mut ResMut<Assets<Mesh>>,
                                     materials: &mut ResMut<Assets<ColorMaterial>>,
                                     _: &Res<AssetServer>,
                                     info: &BulletInfo| {
    let offset_origin = Vec3 {
        x: info.origin.x,
        y: info.origin.y + 20.0,
        z: 0.0,
    };
    commands.spawn((
        BulletMeshBundle {
            bullet: Bullet {
                velocity_shot: *info.velocity,
                velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
                direction: *info.direction,
                // TODO implement
                damage: 10,
                radius: 10,
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
        },
        BulletType::RegularBullet,
    ));
};

pub const FIRE_BULLET: BulletFn = |commands: &mut Commands,
                                   meshes: &mut ResMut<Assets<Mesh>>,
                                   materials: &mut ResMut<Assets<ColorMaterial>>,
                                   _: &Res<AssetServer>,
                                   info: &BulletInfo| {
    let offset_origin = Vec3 {
        x: info.origin.x,
        y: info.origin.y + 20.0,
        z: 0.0,
    };
    commands.spawn((
        BulletMeshBundle {
            bullet: Bullet {
                velocity_shot: *info.velocity,
                velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
                direction: *info.direction,
                // TODO implement
                damage: 10,
                radius: 10,
            },
            mesh_bundle: MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle { radius: 2.0 })),
                material: materials.add(Color::srgb(1.0, 0.0, 0.0)),
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
        },
        BulletType::FireBullet,
    ));
};

pub const NUKE: BulletFn = |commands: &mut Commands,
                            _: &mut ResMut<Assets<Mesh>>,
                            _: &mut ResMut<Assets<ColorMaterial>>,
                            asset_server: &Res<AssetServer>,
                            info: &BulletInfo| {
    let offset_origin = Vec3 {
        x: info.origin.x,
        y: info.origin.y + 20.0,
        z: 0.0,
    };
    commands.spawn((
        BulletSpriteBundle {
            bullet: Bullet {
                velocity_shot: *info.velocity,
                velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
                direction: *info.direction,
                // TODO implement
                damage: 10,
                radius: 10,
            },
            sprite_bundle: SpriteBundle {
                texture: asset_server.load("../assets/nuke.gif"),
                transform: Transform {
                    translation: offset_origin,
                    ..default()
                },
                ..default()
            },
        },
        BulletType::Nuke,
    ));
};
