use std::{fmt::Display, hash::Hash, time::Duration};

use bevy::{
    color::Color,
    math::{Vec2, Vec3},
    prelude::{
        default, Bundle, Circle, Component, DespawnRecursiveExt, EventWriter, Mesh2d, Transform,
    },
    sprite::{ColorMaterial, MeshMaterial2d, Sprite, TextureAtlas},
    time::{Timer, TimerMode},
    utils::HashMap,
};
use enum_iterator::Sequence;
use rand::random;

use crate::utils::{BulletFn, BulletHelpers, CollisionFn, EndTurnEvent};

#[derive(Component)]
pub struct BulletCollider {}

#[derive(Component, Eq, PartialEq, Clone, Copy, Sequence)]
pub enum BulletType {
    RegularBullet,
    FireBullet,
    ClusterBullet,
    Nuke,
}

impl Display for BulletType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = match self {
            BulletType::RegularBullet => "RegularBullet",
            BulletType::FireBullet => "FireBullet",
            BulletType::ClusterBullet => "ClusterBullet",
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
            BulletType::ClusterBullet => 2,
            BulletType::Nuke => 3,
        }
    }

    pub fn get_from_int(value: u32) -> BulletType {
        match value {
            0 => BulletType::RegularBullet,
            1 => BulletType::FireBullet,
            2 => BulletType::ClusterBullet,
            3 => BulletType::Nuke,
            _ => panic!("Bullet with this ID doesn't exist"),
        }
    }

    pub fn get_bullet_from_type(&self) -> Bullet {
        match self {
            BulletType::RegularBullet => NORMAL_BULLET,
            BulletType::FireBullet => FIRE_BULLET,
            BulletType::ClusterBullet => CLUSTER_BULLET,
            BulletType::Nuke => NUKE,
        }
    }

    pub fn get_cost(&self) -> u32 {
        match self {
            BulletType::RegularBullet => 0,
            BulletType::FireBullet => 10,
            BulletType::ClusterBullet => 10,
            BulletType::Nuke => 100,
        }
    }

    pub fn get_max_count(&self) -> u32 {
        match self {
            BulletType::RegularBullet => u32::MAX,
            BulletType::FireBullet => 20,
            BulletType::ClusterBullet => 20,
            BulletType::Nuke => 3,
        }
    }

    pub fn get_radius(&self) -> u32 {
        // TODO get this from somewhere else
        match self {
            BulletType::RegularBullet => 10,
            BulletType::FireBullet => 20,
            BulletType::ClusterBullet => 5,
            BulletType::Nuke => 150,
        }
    }

    pub fn init_bullets() -> HashMap<BulletType, BulletCount> {
        let mut map = HashMap::new();
        map.insert(BulletType::RegularBullet, BulletCount::Unlimited);
        // TODO remove
        map.insert(BulletType::FireBullet, BulletCount::Count(10));
        map.insert(BulletType::ClusterBullet, BulletCount::Count(10));
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
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,
}

#[derive(Bundle)]
pub struct BulletSpriteBundle {
    pub bullet: BulletEntity,
    pub sprite_bundle: Sprite,
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
    |helpers: &mut BulletHelpers, writer: &mut EventWriter<EndTurnEvent>, info: &BulletInfo| {
        // TODO make use of bullettype
        let radius = 10;
        for i in -(radius / 2)..(radius / 2) {
            // TODO use index
            let index = info.origin.x as i32 + i;
            let gg = (1920 + index) as usize;
            let dmg = radius as f32 * 2.0;
            helpers.state.damage[gg] += dmg;
        }
        let current = helpers
            .state
            .active_bullets
            .load(std::sync::atomic::Ordering::SeqCst);
        if current == 0 {
            writer.send(EndTurnEvent {});
            // idk, shit
            return;
        }
        helpers
            .state
            .active_bullets
            .store(current - 1, std::sync::atomic::Ordering::SeqCst);
        if current == 1 {
            writer.send(EndTurnEvent {});
        }
    };

pub const CLUSTER_HIT: CollisionFn =
    |helpers: &mut BulletHelpers, _: &mut EventWriter<EndTurnEvent>, info: &BulletInfo| {
        let current = helpers
            .state
            .active_bullets
            .load(std::sync::atomic::Ordering::Relaxed);
        helpers
            .state
            .active_bullets
            .store(current - 1, std::sync::atomic::Ordering::SeqCst);
        for i in 0..5 {
            let new_velocity = Vec2 {
                x: random::<f32>().clamp(-3.0, i as f32 * 2.0 - 3.0),
                y: 3.5,
            };
            let overriden_info = BulletInfo {
                velocity: &new_velocity,
                origin: info.origin,
                owner: info.owner,
            };
            (NORMAL_BULLET_FN)(helpers, &overriden_info)
        }
    };

pub const NUKE_HIT: CollisionFn =
    |helpers: &mut BulletHelpers, writer: &mut EventWriter<EndTurnEvent>, info: &BulletInfo| {
        // TODO make use of bullettype
        let radius = 200;
        for i in -(radius / 2)..(radius / 2) {
            // TODO use index
            let index = info.origin.x as i32 + i;
            let gg = (1920 + index) as usize;
            let dmg = radius as f32 * 2.0;
            helpers.state.damage[gg] += dmg;
        }
        //let texture_handle = helpers
        //    .assetserver
        //    .load("/assets/images/explosion_anim.png");
        //let texture_atlas =
        //    TextureAtlas::texture_rect(texture_handle, Vec2::new(32.0, 32.0), 3, 1, None, None);
        //let texture_atlas_handle = texture_atlases.add(texture_atlas);
        //commands.spawn(AudioBundle {
        //    source: helpers.assetserver.load("/assets/sounds/explosion.wav"),
        //    ..default()
        //});
        //// TODO aniimation
        //commands.spawn((
        //    SpriteSheetBundle {
        //        sprite: TextureAtlas {
        //            custom_size: Option::Some(Vec2 { x: 1.0, y: 1.0 }),
        //            ..default()
        //        },
        //        texture_atlas: texture_atlas_handle,
        //        transform: Transform {
        //            translation: bullet_transform.translation,
        //            scale: Vec3 {
        //                x: 150.0,
        //                y: 150.0,
        //                z: 1.0,
        //            },
        //            ..default()
        //        },
        //        ..default()
        //    },
        //    AnimationTimer {
        //        timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        //        counter: 2,
        //    },
        //));

        let current = helpers
            .state
            .active_bullets
            .load(std::sync::atomic::Ordering::SeqCst);
        if current == 0 {
            writer.send(EndTurnEvent {});
            // idk, shit
            return;
        }
        helpers
            .state
            .active_bullets
            .store(current - 1, std::sync::atomic::Ordering::SeqCst);
        if current == 1 {
            writer.send(EndTurnEvent {});
        }
    };

pub const NORMAL_BULLET: Bullet = Bullet {
    firefn: NORMAL_BULLET_FN,
    playerhitfn: REGULAR_HIT,
    groundhitfn: REGULAR_HIT,
    bullet_type: BulletType::RegularBullet,
};

pub const NORMAL_BULLET_FN: BulletFn = |helpers: &mut BulletHelpers, info: &BulletInfo| {
    let offset_origin = Vec3 {
        x: info.origin.x,
        y: info.origin.y + 20.0,
        z: 0.0,
    };
    helpers.commands.spawn((
        BulletMeshBundle {
            bullet: BulletEntity {
                velocity_shot: *info.velocity,
                velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
                // TODO implement
                damage: 10,
                radius: 10,
                owner: info.owner,
            },
            mesh: Mesh2d(helpers.meshes.add(Circle { radius: 1.0 })),
            material: MeshMaterial2d(helpers.materials.add(Color::BLACK)),
        },
        Transform {
            translation: offset_origin,
            scale: Vec3 {
                x: 10.0,
                y: 10.0,
                z: 1.0,
            },
            ..default()
        },
        BulletType::RegularBullet,
    ));
    let current = helpers
        .state
        .active_bullets
        .load(std::sync::atomic::Ordering::Relaxed);
    helpers
        .state
        .active_bullets
        .store(current + 1, std::sync::atomic::Ordering::SeqCst);
};

pub const FIRE_BULLET: Bullet = Bullet {
    firefn: FIRE_BULLET_FN,
    playerhitfn: REGULAR_HIT,
    groundhitfn: REGULAR_HIT,
    bullet_type: BulletType::FireBullet,
};

pub const FIRE_BULLET_FN: BulletFn = |helpers: &mut BulletHelpers,

                                      // TODO investigate adding an eventwriter for desapwn event ->  this could instead
                                      // be used to handle the state for triggering a new players turn
                                      // ->  each player only fires once, but this bullet can spawn
                                      // more bullets -> cluster.
                                      // This despawn method can handle these effects by allowing per
                                      // bullet overrides. The cluster can delegate the spawning of
                                      // the event for later.
                                      info: &BulletInfo| {
    let offset_origin = Vec3 {
        x: info.origin.x,
        y: info.origin.y + 20.0,
        z: 0.0,
    };
    helpers.commands.spawn((
        BulletMeshBundle {
            bullet: BulletEntity {
                velocity_shot: *info.velocity,
                velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
                // TODO implement
                damage: 30,
                radius: 20,
                owner: info.owner,
            },
            mesh: Mesh2d(helpers.meshes.add(Circle { radius: 2.0 })),
            material: MeshMaterial2d(helpers.materials.add(Color::srgb(1.0, 0.0, 0.0))),
        },
        Transform {
            translation: offset_origin,
            scale: Vec3 {
                x: 10.0,
                y: 10.0,
                z: 1.0,
            },
            ..default()
        },
        BulletType::FireBullet,
    ));
    let current = helpers
        .state
        .active_bullets
        .load(std::sync::atomic::Ordering::Relaxed);
    helpers
        .state
        .active_bullets
        .store(current + 1, std::sync::atomic::Ordering::SeqCst);
};

pub const CLUSTER_BULLET: Bullet = Bullet {
    firefn: CLUSTER_FN,
    playerhitfn: CLUSTER_HIT,
    groundhitfn: CLUSTER_HIT,
    bullet_type: BulletType::ClusterBullet,
};

pub const CLUSTER_FN: BulletFn = |helpers: &mut BulletHelpers, info: &BulletInfo| {
    let offset_origin = Vec3 {
        x: info.origin.x,
        y: info.origin.y + 20.0,
        z: 0.0,
    };
    helpers.commands.spawn((
        BulletMeshBundle {
            bullet: BulletEntity {
                velocity_shot: *info.velocity,
                velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
                // TODO implement
                damage: 10,
                radius: 10,
                owner: info.owner,
            },
            mesh: Mesh2d(helpers.meshes.add(Circle { radius: 2.0 })),
            material: MeshMaterial2d(helpers.materials.add(Color::srgb(0.0, 1.0, 0.0))),
        },
        Transform {
            translation: offset_origin,
            ..default()
        },
        BulletType::ClusterBullet,
    ));
    let current = helpers
        .state
        .active_bullets
        .load(std::sync::atomic::Ordering::Relaxed);
    helpers
        .state
        .active_bullets
        .store(current + 1, std::sync::atomic::Ordering::SeqCst);
};

pub const NUKE: Bullet = Bullet {
    firefn: NUKE_FN,
    playerhitfn: NUKE_HIT,
    groundhitfn: NUKE_HIT,
    bullet_type: BulletType::Nuke,
};

pub const NUKE_FN: BulletFn = |helpers: &mut BulletHelpers, info: &BulletInfo| {
    let offset_origin = Vec3 {
        x: info.origin.x,
        y: info.origin.y + 20.0,
        z: 0.0,
    };
    helpers.commands.spawn((
        //BulletSpriteBundle {
        //    bullet: BulletEntity {
        //        velocity_shot: *info.velocity,
        //        velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
        //        // TODO implement
        //        damage: 10,
        //        radius: 10,
        //        owner: info.owner,
        //    },
        //    sprite_bundle: Sprite {
        //        image: helpers.assetserver.load("../assets/nuke.gif"),
        //        // TODO transform?
        //        ..default()
        //    },
        //},
        BulletMeshBundle {
            bullet: BulletEntity {
                velocity_shot: *info.velocity,
                velocity_gravity: Vec2 { x: 0.0, y: 9.81 },
                // TODO implement
                damage: 200,
                radius: 200,
                owner: info.owner,
            },
            mesh: Mesh2d(helpers.meshes.add(Circle { radius: 3.0 })),
            material: MeshMaterial2d(helpers.materials.add(Color::srgb(0.0, 0.0, 1.0))),
        },
        Transform {
            translation: offset_origin,
            ..default()
        },
        BulletType::Nuke,
    ));
    let current = helpers
        .state
        .active_bullets
        .load(std::sync::atomic::Ordering::Relaxed);
    helpers
        .state
        .active_bullets
        .store(current + 1, std::sync::atomic::Ordering::SeqCst);
};
