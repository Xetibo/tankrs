use std::{fmt::Display, hash::Hash};

use bevy::{
    color::Color,
    math::{UVec2, Vec2, Vec3},
    prelude::{default, Bundle, Circle, Component, EventWriter, Mesh2d, Transform},
    sprite::{ColorMaterial, MeshMaterial2d, Sprite, TextureAtlas, TextureAtlasLayout},
    time::{Timer, TimerMode},
    utils::HashMap,
};
use enum_iterator::Sequence;
use rand::random;

use crate::{
    utils::{BulletFn, BulletHelpers, CollisionFn, EndTurnEvent},
    AnimationIndices, AnimationTimer,
};

pub fn set_dmg(helpers: &mut BulletHelpers, info: &BulletInfo) {
    let radius = info.radius;
    let lower = -((radius / 2) as i32);
    let upper = (radius / 2) as i32;
    for i in lower..upper {
        let index = info.origin.x as i32 + i;
        let offset_index = (960 + index) as usize;
        let dmg = radius as f32 * 2.0;
        if offset_index > 0 && offset_index < 1921 {
            helpers.state.damage[offset_index] += dmg;
        }
    }
}

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
            BulletType::FireBullet => 100,
            BulletType::ClusterBullet => 500,
            BulletType::Nuke => 2000,
        }
    }

    pub fn get_max_count(&self) -> u32 {
        match self {
            BulletType::RegularBullet => u32::MAX,
            BulletType::FireBullet => 20,
            BulletType::ClusterBullet => 5,
            BulletType::Nuke => 3,
        }
    }

    pub fn get_radius(&self) -> u32 {
        match self {
            BulletType::RegularBullet => 10,
            BulletType::FireBullet => 30,
            BulletType::ClusterBullet => 5,
            BulletType::Nuke => 150,
        }
    }

    pub fn get_dmg(&self) -> u32 {
        match self {
            BulletType::RegularBullet => 20,
            BulletType::FireBullet => 60,
            BulletType::ClusterBullet => 5,
            BulletType::Nuke => 200,
        }
    }

    pub fn init_bullets() -> HashMap<BulletType, BulletCount> {
        let mut map = HashMap::new();
        map.insert(BulletType::RegularBullet, BulletCount::Unlimited);
        // TODO LAST remove
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
    pub radius: u32,
    pub dmg: u32,
}

impl<'a> BulletInfo<'a> {
    pub fn new(velocity: &'a Vec2, origin: &'a Vec3, owner: u32, radius: u32, dmg: u32) -> Self {
        Self {
            velocity,
            origin,
            owner,
            radius,
            dmg,
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
        set_dmg(helpers, info);
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
                radius: info.radius,
                dmg: info.dmg,
            };
            (NORMAL_BULLET_FN)(helpers, &overriden_info)
        }
    };

pub const NUKE_HIT: CollisionFn =
    |helpers: &mut BulletHelpers, writer: &mut EventWriter<EndTurnEvent>, info: &BulletInfo| {
        // TODO make use of bullettype
        set_dmg(helpers, info);

        let texture = helpers.assetserver.load("nuke.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(200), 3, 4, None, None);
        let texture_atlas_layout = helpers.atlas.add(layout);
        // Use only the subset of sprites in the sheet that make up the run animation
        let animation_indices = AnimationIndices { first: 0, last: 9 };
        helpers.commands.spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: animation_indices.first,
                },
            ),
            Transform {
                translation: *info.origin,
                ..Default::default()
            },
            animation_indices,
            BulletType::Nuke,
            AnimationTimer(Timer::from_seconds(0.01, TimerMode::Repeating)),
        ));

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
                damage: info.dmg,
                radius: info.radius,
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
                damage: info.dmg,
                radius: info.radius,
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
                damage: info.dmg,
                radius: info.radius,
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

pub const NUKE_EXPLOSION: Bullet = Bullet {
    // TODO empty fire func
    firefn: NUKE_FN,
    // TODO Add empty hit
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
                damage: info.dmg,
                radius: info.radius,
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
