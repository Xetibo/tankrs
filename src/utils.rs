use bevy::{
    asset::{AssetServer, Assets},
    prelude::{
        Commands, Component, Entity, Event, Mesh, Mut, Query, Res, ResMut, Resource, Transform,
    },
    sprite::{ColorMaterial, Sprite},
    utils::HashMap,
};

use crate::{
    bullets::{BulletCount, BulletInfo, BulletType, NORMAL_BULLET},
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
    pub active_player: u32,
    pub player_count: u32,
    pub player_count_input: String,
    pub player_count_parse_error: bool,
    pub wind: f32,
}

impl GameState {
    pub fn increment_player(&mut self) {
        if self.active_player == self.player_count - 1 {
            self.active_player = 0;
        } else {
            self.active_player += 1;
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            firing: false,
            mode: GameMode::StartMenu,
            active_player: 0,
            player_count_input: "2".into(),
            player_count: 2,
            player_count_parse_error: false,
            wind: random_wind(),
        }
    }
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
    pub fire_velocity: f32,
}

impl Player {
    pub fn selected_bullet(&self) -> BulletFn {
        self.selected_bullet.1
    }

    /// Returns the x axis change according to fuel used
    pub fn drive(&mut self, fuel_change: u32) -> f32 {
        self.fuel = if self.fuel > fuel_change {
            self.fuel - fuel_change
        } else {
            0
        };
        self.fuel as f32
    }

    pub fn from_previous_or_initial(
        index: u32,
        props_opt: Option<&(u32, HashMap<BulletType, BulletCount>)>,
    ) -> Player {
        let (inventory, money) = if let Some(props) = props_opt {
            (props.1.clone(), props.0)
        } else {
            (BulletType::init_bullets(), 0)
        };
        Player {
            player_number: index,
            inventory,
            health: 100,
            fuel: 1000,
            money,
            key_map: KeyMap::default_keymap(),
            selected_bullet: (BulletType::RegularBullet, NORMAL_BULLET),
            fire_velocity: 1.0,
        }
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
    active_player_index: u32,
    query: &'a mut Query<(Entity, &mut Player, &mut Tank, &mut Transform, &mut Sprite)>,
) -> PlayerProps<'a> {
    let (mut entity_opt, mut player_opt, mut tank_opt, mut transform_opt, mut sprite_opt) =
        (None, None, None, None, None);
    for (entity, player, tank, transform, sprite) in query {
        if active_player_index == player.player_number {
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
