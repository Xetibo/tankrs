use bevy::{
    asset::{AssetServer, Assets},
    prelude::{
        Commands, Component, Entity, Event, Mesh, Mut, Query, Res, ResMut, Resource, Transform,
    },
    sprite::{ColorMaterial, Sprite},
    utils::HashMap,
};

use crate::{
    bullets::{BulletCount, BulletInfo, BulletType},
    inputs::KeyMap,
    tank::Tank,
};

#[derive(Event)]
pub struct PlayerKillEvent {
    pub killer: u32,
    pub killed: u32,
}

#[derive(Event)]
pub struct FireEvent {}

#[derive(Event)]
pub struct EndTurnEvent {}

#[derive(Event)]
pub struct ResetEvent {}

#[derive(Component)]
pub struct Inventory {
    //
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Battle,
    Shop,
    StartMenu,
}

// TODO move out to models
#[derive(Resource)]
pub struct GameState {
    pub firing: bool,
    pub mode: GameMode,
    pub player_count: u32,
    pub player_count_input: String,
    pub player_count_parse_error: bool,
    pub wind: f32,
}

pub type BulletFn = fn(
    &mut Commands,
    &mut ResMut<Assets<Mesh>>,
    &mut ResMut<Assets<ColorMaterial>>,
    &Res<AssetServer>,
    &BulletInfo,
);
pub type BulletTypeAndFn = (BulletType, BulletFn);

pub type PlayerProps<'a> = Option<(
    Entity,
    Mut<'a, Player>,
    Mut<'a, Tank>,
    Mut<'a, Transform>,
    Mut<'a, Sprite>,
)>;

#[derive(Component, Clone)]
pub struct Player {
    pub player_number: u32,
    pub inventory: HashMap<BulletType, BulletCount>,
    pub selected_bullet: BulletTypeAndFn,
    pub health: i32,
    pub fuel: u32,
    pub money: u32,
    pub key_map: KeyMap,
    pub is_active: bool,
    pub fire_velocity: f32,
}

impl Player {
    pub fn selected_bullet(&self) -> BulletFn {
        self.selected_bullet.1
    }
}

pub fn polynomial(x: i32, rand: f32) -> f32 {
    let x = x as f32;
    //(f32::consts::E - x) * (x * f32::consts::E) *
    //(rand * power(x, 4)) + (rand * power(x, 3)) - (rand * power(x, 2)) - (rand * x)
    //power(x, 5) - power(x, 4) - (5.0 * power(x, 3)) + (3.0 * power(x, 2)) + (4.0 * x) - 3.0
    //((rand * x) * 0.00005).cos() * 1000.0 * rand
    (x * rand * 0.005).cos() * 100. * rand + 100.0
}

pub fn power(_num: f32, pow: i32) -> f32 {
    if pow > 0 {
        power(_num, pow - 1)
    } else {
        1.0
    }
}

pub fn get_current_player_props<'a>(
    query: &'a mut Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
) -> PlayerProps<'a> {
    let (mut entity_opt, mut player_opt, mut tank_opt, mut transform_opt, mut sprite_opt) =
        (None, None, None, None, None);
    for (entity, player, tank, transform, sprite) in query {
        if player.is_active {
            entity_opt = Some(entity);
            player_opt = Some(player);
            tank_opt = Some(tank);
            transform_opt = Some(transform);
            sprite_opt = Some(sprite);
        }
    }
    if let (Some(player), Some(entity), Some(tank), Some(transform), Some(sprite)) =
        (player_opt, entity_opt, tank_opt, transform_opt, sprite_opt)
    {
        Some((entity, player, tank, transform, sprite))
    } else {
        None
    }
}

pub fn random_wind() -> f32 {
    rand::random::<f32>().clamp(-0.3, 0.3)
}
