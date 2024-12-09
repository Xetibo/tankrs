use std::{fmt::Display, hash::Hash};

use bevy::{
    asset::{AssetServer, Assets},
    color::Color,
    math::{Vec2, Vec3},
    prelude::{
        default, Bundle, Circle, Commands, Component, EventWriter, Mesh, Res, ResMut, Transform,
    },
    sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle, SpriteBundle},
    utils::HashMap,
};
use enum_iterator::Sequence;

use crate::utils::{BulletFn, CollisionFn, EndTurnEvent, GameState};

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

    pub fn get_bullet_from_type(&self) -> Bullet {
        match self {
            BulletType::RegularBullet => NORMAL_BULLET,
            BulletType::FireBullet => FIRE_BULLET,
            BulletType::Nuke => NUKE,
        }
    }

    pub fn get_cost(&self) -> u32 {
        match self {
            BulletType::RegularBullet => 0,
            BulletType::FireBullet => 10,
            BulletType::Nuke => 100,
        }
    }

    pub fn get_max_count(&self) -> u32 {
        match self {
            BulletType::RegularBullet => u32::MAX,
            BulletType::FireBullet => 20,
            BulletType::Nuke => 3,
        }
    }

    pub fn get_radius(&self) -> u32 {
        // TODO get this from somewhere else
        match self {
            BulletType::RegularBullet => 10,
            BulletType::FireBullet => 20,
            BulletType::Nuke => 150,
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

#[derive(Component, Clone, Copy)]
pub enum BulletCount {
    Unlimited,
    Count(u32),
}

impl BulletCount {
    pub fn increment(&self) -> BulletCount {
        match self {
            BulletCount::Unlimited => BulletCount::Unlimited,
            BulletCount::Count(count) => BulletCount::Count(count + 1),
        }
    }

    pub fn decrement(&self) -> BulletCount {
        match self {
            BulletCount::Unlimited => BulletCount::Unlimited,
            BulletCount::Count(count) => BulletCount::Count(count - 1),
        }
    }
}

#[derive(Component)]
pub struct BulletEntity {
    pub velocity_shot: Vec2,
    pub velocity_gravity: Vec2,
    pub damage: u32,
    pub radius: u32,
    pub owner: u32,
}

#[derive(Bundle)]
pub struct BulletMeshBundle {
    pub bullet: BulletEntity,
    pub mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Bundle)]
pub struct BulletSpriteBundle {
    pub bullet: BulletEntity,
    pub sprite_bundle: SpriteBundle,
}

pub struct BulletInfo<'a> {
    pub velocity: &'a Vec2,
    pub origin: &'a Vec3,
    pub owner: u32,
}

impl<'a> BulletInfo<'a> {
    pub fn new(velocity: &'a Vec2, origin: &'a Vec3, owner: u32) -> Self {
        Self {
            velocity,
            origin,
            owner,
        }
    }
}

#[derive(Component, Clone)]
pub struct Bullet {
    pub firefn: BulletFn,
    pub playerhitfn: CollisionFn,
    pub groundhitfn: CollisionFn,
    pub bullet_type: BulletType,
}

pub const REGULAR_HIT: CollisionFn =
    |_: &mut Commands, state: &mut ResMut<GameState>, writer: &mut EventWriter<EndTurnEvent>| {
        // TODO make use of bullettype
        for i in 0..10 {
            // TODO use index
            state.damage[1920 + i] = i as f32 * 10.0;
        }
        state.firing = false;
        writer.send(EndTurnEvent {});
    };

pub const NORMAL_BULLET: Bullet = Bullet {
    firefn: NORMAL_BULLET_FN,
    playerhitfn: REGULAR_HIT,
    groundhitfn: REGULAR_HIT,
    bullet_type: BulletType::RegularBullet,
};

pub const NORMAL_BULLET_FN: BulletFn = |commands: &mut Commands,
                                        state: &mut ResMut<GameState>,
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
            bullet: BulletEntity {
                velocity_shot: *info.velocity,
                velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
                // TODO implement
                damage: 10,
                radius: 10,
                owner: info.owner,
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
    state.firing = true;
};

pub const FIRE_BULLET: Bullet = Bullet {
    firefn: FIRE_BULLET_FN,
    playerhitfn: REGULAR_HIT,
    groundhitfn: REGULAR_HIT,
    bullet_type: BulletType::Nuke,
};

pub const FIRE_BULLET_FN: BulletFn = |commands: &mut Commands,
                                      state: &mut ResMut<GameState>,
                                      meshes: &mut ResMut<Assets<Mesh>>,
                                      materials: &mut ResMut<Assets<ColorMaterial>>,
                                      // TODO investigate adding an eventwriter for desapwn event ->  this could instead
                                      // be used to handle the state for triggering a new players turn
                                      // ->  each player only fires once, but this bullet can spawn
                                      // more bullets -> cluster.
                                      // This despawn method can handle these effects by allowing per
                                      // bullet overrides. The cluster can delegate the spawning of
                                      // the event for later.
                                      _: &Res<AssetServer>,
                                      info: &BulletInfo| {
    let offset_origin = Vec3 {
        x: info.origin.x,
        y: info.origin.y + 20.0,
        z: 0.0,
    };
    commands.spawn((
        BulletMeshBundle {
            bullet: BulletEntity {
                velocity_shot: *info.velocity,
                velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
                // TODO implement
                damage: 10,
                radius: 10,
                owner: info.owner,
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
    state.firing = true;
};

pub const NUKE: Bullet = Bullet {
    firefn: NUKE_FN,
    playerhitfn: REGULAR_HIT,
    groundhitfn: REGULAR_HIT,
    bullet_type: BulletType::Nuke,
};

pub const NUKE_FN: BulletFn = |commands: &mut Commands,
                               state: &mut ResMut<GameState>,
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
            bullet: BulletEntity {
                velocity_shot: *info.velocity,
                velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
                // TODO implement
                damage: 10,
                radius: 10,
                owner: info.owner,
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
    state.firing = true;
    // TODO add callback for destruction
};
