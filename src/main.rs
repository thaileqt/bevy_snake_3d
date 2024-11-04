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
        .add_event::<SpawnFoodEvent>()
        .add_event::<SpawnSnakeTail>()
        .add_systems(Startup, (
            spawn_map,
        ).chain())
        .add_systems(Update, (
            spawn_food,
            spawn_snake_tail,
        ))
        .run();
}

#[derive(Event)]
struct SpawnSnakeTail;
#[derive(Event)]
struct SpawnFoodEvent;
#[derive(Component)]
struct Food;

fn spawn_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    let map_cube = meshes.add(Cuboid::new(1.0-CUBE_SPACE/2., 1.0-CUBE_SPACE/2., 1.0-CUBE_SPACE/2.));
    let map_cube_mat = materials.add(Color::srgb_u8(124, 144, 255));
    for i in 0..MAP_SIZE {
        for j in 0..MAP_SIZE {
            commands.spawn((
                Mesh3d(map_cube.clone()),
                MeshMaterial3d(map_cube_mat.clone()),
                Transform::from_xyz(i as f32, -1.0, j as f32),
            ));
        }
    }

    // Spawn player
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(HEAD_SIZE, HEAD_SIZE, HEAD_SIZE))),
        MeshMaterial3d(materials.add(SNAKE_HEAD_COLOR)),
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_food_event: EventReader<SpawnFoodEvent>,
) {
    for _ in spawn_food_event.read() {
        let mut rng = thread_rng();
        let x_rand = rng.gen_range(0..MAP_SIZE);
        let z_rand = rng.gen_range(0..MAP_SIZE);
    
        commands.spawn((
            Food,
            FoodAnimation::default(),
            Mesh3d(meshes.add(Cuboid::new(FOOD_SIZE, FOOD_SIZE, FOOD_SIZE))),
            Transform::from_xyz(x_rand as f32, 0.0, z_rand as f32),
            MeshMaterial3d(materials.add(StandardMaterial {
                emissive: FOOD_COLOR.into(),
                ..default()
            })),
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
            // TailAppearAnimation::default(),
            
            // MeshMaterial3d(materials.add(Color::srgb_u8(0, 100, 255))),
        ))
        .with_children(|parent| {
            parent.spawn((
                TailAppearAnimation::default(),
                Mesh3d(meshes.add(Cuboid::new(BODY_SIZE, BODY_SIZE, BODY_SIZE))),
                Transform::from_translation(Vec3::ZERO).with_scale(Vec3::ZERO),
                MeshMaterial3d(materials.add(StandardMaterial {
                    emissive: SNAKE_BODY_COLOR.into(),
                    ..default()
                })),
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
