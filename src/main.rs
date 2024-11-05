use std::time::Duration;

use bevy::{
    audio::AudioPlugin, core_pipeline::{bloom::Bloom, tonemapping::Tonemapping}, 
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, 
    pbr::NotShadowCaster, prelude::*
};
use camera::TopdownCamera;
use game_flow::{MapModifyEvent, SpawnFoodEvent};
use player::*;

mod camera;
mod player;
mod animation;
mod menu;
mod game_flow;
mod utils;

// Size
const MAP_SIZE: usize   = 25;
const CUBE_SPACE: f32   = 0.2;
const HEAD_SIZE: f32    = 0.6;
const BODY_SIZE: f32    = 0.4;
const FOOD_SIZE: f32    = 0.4;

// Colors
const SNAKE_HEAD_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const SNAKE_BODY_COLOR: Color = Color::srgb(0.0, 0.39, 1.0);
const FOOD_COLOR:       Color = Color::srgb(0.0, 0.39, 1.0);
const RED_COLOR:        Color = Color::srgb(1.0, 0.0, 0.0);
const GREEN_COLOR:      Color = Color::srgb(0.0, 1.0, 0.0);


fn main() {
    App::new()
        .add_plugins((
            // DefaultPlugins,
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

            // FrameTimeDiagnosticsPlugin,
            // LogDiagnosticsPlugin::default(),
        ))
        .init_state::<GameState>()
        .init_resource::<MapState>()
        .add_systems(OnEnter(GameState::Loading), load_assets)
        .add_systems(OnEnter(GameState::Menu), spawn_world)
        .add_systems(Update, (
            change_track,
            fade_in,
            fade_out,
        ))
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
    // @@ Audio @@
    // sfx
    pub pickup: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
    pub speed_boost: Handle<AudioSource>,
    pub cube_move: Handle<AudioSource>,
    pub button_click: Handle<AudioSource>,
    pub fail: Handle<AudioSource>,
    // soundtracks
    pub menu_track: Handle<AudioSource>,
    pub ingame_track: Handle<AudioSource>,
    

    // @@ Meshs and Materials @@
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
impl Default for MapState {
    fn default() -> Self {
        Self { 
            grid: Vec::new(), 
            score: 0, 
            time_elapsed: 0.0,
            map_change_timer: Timer::from_seconds(5.0, TimerMode::Repeating), 
        }
    }
}
impl MapState {
    fn set_grid(&mut self, grid: Vec<Entity>) {
        self.grid = grid;
    }

    fn update(
        mut map_state:  ResMut<MapState>, 
        time:           Res<Time>,
        mut ev_writer:  EventWriter<MapModifyEvent>,
    ) {
        map_state.time_elapsed += time.delta_secs();
        map_state.map_change_timer.tick(Duration::from_secs_f32(time.delta_secs()));
        if map_state.map_change_timer.just_finished() {
            ev_writer.send(MapModifyEvent{
                cube_count: 10 + ((map_state.time_elapsed / 20.) as usize).min(25)
            });
        }
    }
}

fn load_assets(
    mut commands:   Commands,
    asset_server:   Res<AssetServer>,
    mut meshes:     ResMut<Assets<Mesh>>,
    mut materials:  ResMut<Assets<StandardMaterial>>,
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
        game_over: asset_server.load("audio/game_over.ogg"),
        speed_boost: asset_server.load("audio/speed_boost.ogg"),
        cube_move: asset_server.load("audio/plop.ogg"),
        button_click: asset_server.load("audio/button_click.ogg"),
        fail: asset_server.load("audio/blip.ogg"),
        // soundtracks
        menu_track: asset_server.load("audio/Lake-Jupiter-John-Patitucci.ogg"),
        ingame_track: asset_server.load("audio/Sloppy-Clav-Godmode.ogg"),

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
    mut commands:   Commands,
    game_assets:    Res<GlobalAssets>,
    mut map_state:  ResMut<MapState>,
    mut spawn_food_event: EventWriter<SpawnFoodEvent>,
    cam_query:      Query<&TopdownCamera>,
) {
    match cam_query.get_single() {
        Ok(_) => {},
        Err(_) => {
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
        }
    }
    
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
    map_state.set_grid(grid);
    

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




// This component will be attached to an entity to fade the audio in
#[derive(Component)]
struct FadeIn;

// This component will be attached to an entity to fade the audio out
#[derive(Component)]
struct FadeOut;

// Every time the GameState resource changes, this system is run to trigger the song change.
fn change_track(
    mut commands: Commands,
    game_assets: Res<GlobalAssets>,
    soundtrack: Query<Entity, With<AudioSink>>,
    game_state: Res<State<GameState>>,
) {
    if !game_state.is_changed() {
        return; 
    }
    // Fade out all currently running tracks
    for track in soundtrack.iter() {
        commands.entity(track).insert(FadeOut);
    }
    match game_state.get() {
        GameState::Menu => {
            commands.spawn((
                AudioPlayer(game_assets.menu_track.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    volume: bevy::audio::Volume::ZERO,
                    ..default()
                },
                FadeIn,
            ));
        
        }
        GameState::InGame => {
            commands.spawn((
                AudioPlayer(game_assets.ingame_track.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    volume: bevy::audio::Volume::ZERO,
                    ..default()
                },
                FadeIn,
            ));
        }
        _ => {}
    }
    
    
    
 
}
const SOUND_TRACK_VOLUME: f32 = 0.4;
// Fade effect duration
const FADE_TIME: f32 = 2.0;

// Fades in the audio of entities that has the FadeIn component. Removes the FadeIn component once
// full volume is reached.
fn fade_in(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity), With<FadeIn>>,
    time: Res<Time>,
) {
    for (audio, entity) in audio_sink.iter_mut() {
        audio.set_volume(audio.volume() + time.delta_secs() / FADE_TIME);
        if audio.volume() >= SOUND_TRACK_VOLUME {
            audio.set_volume(SOUND_TRACK_VOLUME);
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}

// Fades out the audio of entities that has the FadeOut component. Despawns the entities once audio
// volume reaches zero.
fn fade_out(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity), With<FadeOut>>,
    time: Res<Time>,
) {
    for (audio, entity) in audio_sink.iter_mut() {
        audio.set_volume(audio.volume() - time.delta_secs() / FADE_TIME);
        if audio.volume() <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}