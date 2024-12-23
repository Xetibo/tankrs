use std::sync::atomic::AtomicU32;

use crate::{
    bullets::{Bullet, BulletCount, BulletInfo, BulletType, NORMAL_BULLET},
    inputs::KeyMap,
    tank::Tank,
};
use bevy::{
    asset::{AssetServer, Assets},
    prelude::{
        Commands, Component, Entity, Event, EventWriter, Mesh, Mut, Query, Res, ResMut, Resource,
        Transform,
    },
    sprite::{ColorMaterial, Sprite},
    utils::HashMap,
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

#[derive(Event)]
pub struct RedrawTerrainEvent {}

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
    pub active_bullets: AtomicU32,
    pub mode: GameMode,
    pub active_player: u32,
    pub player_count: u32,
    pub player_count_input: String,
    pub player_count_parse_error: bool,
    pub wind: f32,
    pub rand: f32,
    pub damage: [f32; (1920 * 2) + 1],
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
        let (wind, rand) = next_random();
        GameState {
            active_bullets: AtomicU32::new(0),
            mode: GameMode::StartMenu,
            active_player: 0,
            player_count_input: "2".into(),
            player_count: 2,
            player_count_parse_error: false,
            wind,
            rand,
            damage: [0.0; (1920 * 2) + 1],
        }
    }
}

pub struct BulletHelpers<'w, 's, 'a>
where
    'w: 'a,
    's: 'a,
{
    pub commands: &'a mut Commands<'w, 's>,
    pub state: &'a mut GameState,
    pub meshes: &'a mut ResMut<'w, Assets<Mesh>>,
    pub materials: &'a mut ResMut<'w, Assets<ColorMaterial>>,
    pub assetserver: &'a Res<'w, AssetServer>,
}

pub type BulletFn = fn(&mut BulletHelpers, &BulletInfo);

pub type CollisionFn = fn(&mut BulletHelpers, &mut EventWriter<EndTurnEvent>, &BulletInfo);

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
    pub selected_bullet: Bullet,
    pub health: i32,
    pub fuel: u32,
    pub money: u32,
    pub key_map: KeyMap,
    pub fire_velocity: f32,
}

impl Player {
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
            selected_bullet: NORMAL_BULLET,
            fire_velocity: 1.0,
        }
    }
}

pub fn polynomial(x: i32, state: &GameState) -> f32 {
    let damage_index = (x + 1920) as usize;
    let damage = state.damage[damage_index];

    let x = x as f32;
    let rand = state.rand;
    let f3 = (x * rand * 0.005).cos();
    ((rand * 800. * f3) - damage).max(10.0)
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

pub fn next_random() -> (f32, f32) {
    (
        rand::random::<f32>().clamp(-0.3, 0.3),
        rand::random::<f32>().clamp(0.5, 1.5),
    )
}
