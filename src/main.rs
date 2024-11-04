use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping}, 
    prelude::*,
};
use camera::{CameraFollowTarget, TopdownCamera};
use rand::{thread_rng, Rng};
use player::*;
use animation::*;

mod camera;
mod player;
mod animation;

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



fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            camera::CameraPlugin,
            player::PlayerPlugin,
            animation::AnimationPlugin,
        ))
        .init_state::<GameState>()
        .add_event::<SpawnFoodEvent>()
        .add_event::<SpawnSnakeTail>()
        .add_systems(OnEnter(GameState::Loading), load_assets)
        .add_systems(OnExit(GameState::Loading), spawn_world)
        .add_systems(Update, (
            spawn_food,
            spawn_snake_tail,
        ).run_if(in_state(GameState::InGame)))
        .run();
}
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Menu,
    InGame,
}

#[derive(Event)]
struct SpawnSnakeTail;
#[derive(Event)]
struct SpawnFoodEvent;
#[derive(Component)]
struct Food;

#[derive(Resource)]
pub struct GlobalAssets {
    pub map_cube: Handle<Mesh>,
    pub map_cube_mat: Handle<StandardMaterial>,
    // Snake
    pub snake_head: Handle<Mesh>,
    pub snake_head_mat: Handle<StandardMaterial>,
    pub snake_body: Handle<Mesh>,
    pub snake_body_mat: Handle<StandardMaterial>,
    // Food
    pub food: Handle<Mesh>,
    pub food_mat: Handle<StandardMaterial>,
}

fn load_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Map
    let map_cube = meshes.add(Cuboid::new(1.0-CUBE_SPACE/2., 1.0-CUBE_SPACE/2., 1.0-CUBE_SPACE/2.));
    let map_cube_mat = materials.add(Color::srgb_u8(124, 144, 255));
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
        map_cube,
        map_cube_mat,
        snake_head,
        snake_head_mat,
        snake_body,
        snake_body_mat,
        food,
        food_mat,
    });

    next_state.set(GameState::InGame);
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

    for i in 0..MAP_SIZE {
        for j in 0..MAP_SIZE {
            commands.spawn((
                Mesh3d(game_assets.map_cube.clone()),
                MeshMaterial3d(game_assets.map_cube_mat.clone()),
                Transform::from_xyz(i as f32, -1.0, j as f32),
            ));
        }
    }

    // Spawn player
    commands.spawn((
        Mesh3d(game_assets.snake_head.clone()),
        MeshMaterial3d(game_assets.snake_head_mat.clone()),
        Transform::from_xyz((MAP_SIZE as f32 / 2.0).floor(), 0.0, (MAP_SIZE as f32 / 2.0).floor()),
        Snake::default(),
        CameraFollowTarget,
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

    spawn_food_event.send(SpawnFoodEvent);
}

fn spawn_food(
    mut commands: Commands,
    game_assets: Res<GlobalAssets>,
    mut spawn_food_event: EventReader<SpawnFoodEvent>,
) {
    for _ in spawn_food_event.read() {
        let mut rng = thread_rng();
        let x_rand = rng.gen_range(0..MAP_SIZE);
        let z_rand = rng.gen_range(0..MAP_SIZE);
    
        commands.spawn((
            Food,
            FoodAnimation::default(),
            Mesh3d(game_assets.food.clone()),
            Transform::from_xyz(x_rand as f32, 0.0, z_rand as f32),
            MeshMaterial3d(game_assets.food_mat.clone()),
        )).with_children(|parent| {
            parent.spawn((
                SpotLight {
                    intensity: 5_000_000.0,
                    range: 10.0,
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_xyz(0.0, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ));
        });
    }
}


fn spawn_snake_tail(
    mut ev_reader: EventReader<SpawnSnakeTail>,
    mut commands: Commands,
    game_assets: Res<GlobalAssets>,
    mut snake_query: Query<(&Transform, &mut Snake), (With<Snake>, Without<SnakeBody>)>,
    snake_bodies_query: Query<(&Transform, &SnakeBody), (With<SnakeBody>, Without<Snake>)>,
) {
    for _ in ev_reader.read() {
        let (snake_transform, mut snake) =  snake_query.single_mut();
        // let mut tail: SnakeBody = SnakeBody::new(Vec3::ZERO);
        let mut tail_init_pos: Vec3 = Vec3::ZERO;
        let tail =  if snake.bodies.is_empty() {
            tail_init_pos = snake_transform.translation - snake.direction.norm();
            SnakeBody::new(snake.pos)
        } else {
            if let Ok((transform, data)) = snake_bodies_query.get(*snake.bodies.last().unwrap()) {
                
                let last_tail_dir = (data.target_position - transform.translation).normalize();
                tail_init_pos = transform.translation - last_tail_dir;
                SnakeBody::new(data.target_position-last_tail_dir)
            } else {
                SnakeBody::new(Vec3::ZERO)
            }
        };
        let entity = commands.spawn((
            tail,
            Transform::from_translation(tail_init_pos),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent.spawn((
                TailAppearAnimation::default(),
                Mesh3d(game_assets.snake_body.clone()),
                Transform::from_translation(Vec3::ZERO).with_scale(Vec3::ZERO),
                MeshMaterial3d(game_assets.snake_body_mat.clone()),
            ));
            parent.spawn((
                SpotLight {
                    range: 10.0,
                    intensity: 500_000.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_xyz(0.0, 3.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ));
        })
        .id();
        snake.bodies.push(entity);
    }
   
}
