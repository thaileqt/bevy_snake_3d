use std::time::Duration;

use bevy::{
    audio::AudioPlugin, core_pipeline::{bloom::Bloom, tonemapping::Tonemapping}, pbr::NotShadowCaster, prelude::*
};
use camera::{CameraFollowTarget, TopdownCamera};
use game_flow::{Food, MapModifyEvent, SpawnFoodEvent};
use player::*;
use rand::{seq::SliceRandom, thread_rng};

mod camera;
mod player;
mod animation;
mod menu;
mod game_flow;
mod utils;

// Size
const MAP_SIZE: usize = 25;
const CUBE_SPACE: f32 = 0.2;
const HEAD_SIZE: f32 = 0.6;
const BODY_SIZE: f32 = 0.4;
const FOOD_SIZE: f32 = 0.4;

// Colors
const SNAKE_HEAD_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const SNAKE_BODY_COLOR: Color = Color::srgb(0.0, 0.39, 1.0);
const FOOD_COLOR:       Color = Color::srgb(0.0, 0.39, 1.0);

// Effects
const RED_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const GREEN_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(
                AudioPlugin {
                    global_volume: GlobalVolume::new(0.5),
                    ..default()
                }
            ),
            camera::CameraPlugin,
            player::PlayerPlugin,
            animation::AnimationPlugin,
            menu::MenuPlugin,
            game_flow::GameFlowPlugin,
        ))
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Loading), load_assets)
        .add_systems(OnExit(GameState::Loading), spawn_world)
        .run();
}
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Menu,
    InGame,
}



#[derive(Resource)]
pub struct GlobalAssets {
    // Audio
    pub pickup: Handle<AudioSource>,

    pub map_cube: Handle<Mesh>,
    pub map_cube_mat: Handle<StandardMaterial>,
    pub map_cube_mat_emission: Handle<StandardMaterial>,
    // Snake
    pub snake_head: Handle<Mesh>,
    pub snake_head_mat: Handle<StandardMaterial>,
    pub snake_body: Handle<Mesh>,
    pub snake_body_mat: Handle<StandardMaterial>,
    // Food
    pub food: Handle<Mesh>,
    pub food_mat: Handle<StandardMaterial>,

    // Effects
    pub red_mat: Handle<StandardMaterial>,
    pub green_mat: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct MapState {
    grid: Vec<Entity>,
    score: i32,
    time_elapsed: f32,
    map_change_timer: Timer,
}
type TilePos = (usize, usize);
#[derive(Clone, Component)]
pub struct CubeState {
    pub pos: TilePos,
    pub walkable: bool,
}

impl MapState {
    fn new(grid: Vec<Entity>) -> Self {
        Self { 
            grid, 
            score: 0, 
            time_elapsed: 0.0,
            map_change_timer: Timer::from_seconds(5.0, TimerMode::Repeating), 
        }
    }

    fn choose_random_tile_entities(&self, n: usize) -> Vec<Entity> {
        if n >= self.grid.len() {
            return self.grid.clone();
        }
        let mut rng = thread_rng();
        let mut chosen_tiles = self.grid.clone();
        chosen_tiles.as_mut_slice().choose_multiple(&mut rng, n).cloned().collect()
    }

    fn update(
        mut map_state: ResMut<MapState>, 
        time: Res<Time>,
        mut map_modify_ev_writer: EventWriter<MapModifyEvent>,
    ) {
        map_state.time_elapsed += time.delta_secs();
        map_state.map_change_timer.tick(Duration::from_secs_f32(time.delta_secs()));
        if map_state.map_change_timer.just_finished() {
            map_modify_ev_writer.send(MapModifyEvent{cube_count: 10 + ((map_state.time_elapsed / 20.) as usize).min(25)});
        }
    }
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Map
    let map_cube = meshes.add(Cuboid::new(1.0-CUBE_SPACE/2., 1.0-CUBE_SPACE/2., 1.0-CUBE_SPACE/2.));
    let map_cube_mat = materials.add(Color::srgb_u8(124, 144, 255));
    let map_cube_mat_emission = materials.add(StandardMaterial {
        emissive: SNAKE_BODY_COLOR.into(),
        ..default()
    });
    // Snake
    let snake_head = meshes.add(Cuboid::new(HEAD_SIZE, HEAD_SIZE, HEAD_SIZE));
    let snake_head_mat = materials.add(SNAKE_HEAD_COLOR);
    let snake_body = meshes.add(Cuboid::new(BODY_SIZE, BODY_SIZE, BODY_SIZE));
    let snake_body_mat = materials.add(StandardMaterial {
        emissive: SNAKE_BODY_COLOR.into(),
        ..default()
    });
    // Food
    let food = meshes.add(Cuboid::new(FOOD_SIZE, FOOD_SIZE, FOOD_SIZE));
    let food_mat = materials.add(StandardMaterial {
        emissive: FOOD_COLOR.into(),
        ..default()
    });

    commands.insert_resource(GlobalAssets {
        pickup: asset_server.load("audio/plop.ogg"),
        map_cube,
        map_cube_mat,
        map_cube_mat_emission,
        snake_head,
        snake_head_mat,
        snake_body,
        snake_body_mat,
        food,
        food_mat,

        red_mat: materials.add(StandardMaterial {
            base_color: RED_COLOR.into(),
            emissive: RED_COLOR.into(),
            ..default()
        }),
        green_mat: materials.add(StandardMaterial {
            base_color: GREEN_COLOR.into(),
            emissive: GREEN_COLOR.into(),
            ..default()
        }),
    });

    next_state.set(GameState::Menu);
}

fn spawn_world(
    mut commands: Commands,
    game_assets: Res<GlobalAssets>,
    mut spawn_food_event: EventWriter<SpawnFoodEvent>,
) {
    // Spawn camera follow player
    commands.spawn((
        Camera3d::default(),
        // Camera {
        //     hdr: true,
        //     ..default()
        // },
        // Tonemapping::TonyMcMapface,
        // Bloom::NATURAL,
        Transform::from_xyz(-4.5, 15.5, 19.0).looking_at(Vec3::ZERO, Vec3::Y),
        TopdownCamera::with_offset(Vec3::new(0.0, 15.0, 15.0)),
    ));

    let mut grid: Vec<Entity> = Vec::new();
    for i in 0..MAP_SIZE {
        for j in 0..MAP_SIZE {
            let cube_state = CubeState {
                pos: (i, j),
                walkable: true,
            };
            let cube = commands.spawn((
                cube_state,
                Mesh3d(game_assets.map_cube.clone()),
                MeshMaterial3d(game_assets.map_cube_mat.clone()),
                Transform::from_xyz(i as f32, -1.0, j as f32),
                NotShadowCaster,
            )).id();
            grid.push(cube);
        }
    }
    commands.insert_resource(MapState::new(grid));

    // Spawn player
    commands.spawn((
        Mesh3d(game_assets.snake_head.clone()),
        MeshMaterial3d(game_assets.snake_head_mat.clone()),
        Transform::from_xyz((MAP_SIZE as f32 / 2.0).floor(), 0.0, (MAP_SIZE as f32 / 2.0).floor()),
        Snake::default(),
        // CameraFollowTarget,
    )).with_children(|parent| {
        parent.spawn((
            SpotLight {
                range: 50.0,
                intensity: 5_000_000.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    });

    for _ in 0..4 {
        spawn_food_event.send(SpawnFoodEvent);
    }
    
}
