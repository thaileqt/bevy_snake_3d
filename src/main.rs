//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::{f32::consts::PI, time::Duration};

use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping}, 
    prelude::*,
};
use camera::{CameraFollowTarget, TopdownCamera};
use rand::{thread_rng, Rng};

mod camera;


const MAP_SIZE: usize = 25;
const CUBE_SPACE: f32 = 0.2;
// Snake data
const HEAD_SIZE: f32 = 0.6;
const BODY_SIZE: f32 = 0.4;


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            camera::CameraPlugin,
        ))
        .add_event::<SpawnFoodEvent>()
        .add_event::<SpawnSnakeTail>()
        .add_systems(Startup, (
            spawn_map,
        ).chain())
        .add_systems(Update, (
            handle_direction_change, 
            move_snake, 
            spawn_food,
            spawn_snake_tail,
            update_food_animation,
            update_tail_appear_animation,
        ))
        .run();
}

#[derive(Component, Clone)]
struct Snake {
    pos: Vec3,
    direction: Direction,
    target_position: Vec3,
    speed: f32,
    wait: Timer,
    bodies: Vec<Entity>,
}
#[derive(Component, Clone)]
struct SnakeBody {
    target_position: Vec3,
}

impl SnakeBody {
    fn new(at: Vec3) -> Self {
        Self { target_position: at }
    }

}

impl Snake {
    fn get_next_target(&self) -> Vec3 {
        self.pos + self.direction.norm()
    }
}

impl Default for Snake {
    fn default() -> Self {
        let start_pos = Vec3::new(12.0, 0.0, 12.0);
        let start_dir = Direction::Up;
        let start_target_pos = start_pos + start_dir.norm();
        Self {
            direction: start_dir,
            pos: start_pos,
            speed: 5.0,
            target_position: start_target_pos,
            wait: Timer::from_seconds(1.0, TimerMode::Repeating),
            bodies: Vec::new(),
        }
    }
}
#[derive(Clone, Copy)]
enum Direction { Up, Down, Left, Right }
impl Direction {
    fn norm(&self) -> Vec3 {
        match self {
            Direction::Up => Vec3::new(0.0 ,0.0, -1.0),
            Direction::Down => Vec3::new(0.0, 0.0 ,1.0),
            Direction::Left => Vec3::new(1.0, 0.0, 0.0),
            Direction::Right => Vec3::new(-1.0, 0.0, 0.0),
        }
    }

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
        Camera {
            hdr: true,
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
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
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 0, 0))),
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
            Mesh3d(meshes.add(Cuboid::new(0.35, 0.35, 0.35))),
            Transform::from_xyz(x_rand as f32, 0.0, z_rand as f32),
            MeshMaterial3d(materials.add(StandardMaterial {
                emissive: Color::srgb_u8(0, 100, 255).into(),
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
                Mesh3d(meshes.add(Cuboid::new(0.35, 0.35, 0.35))),
                Transform::from_translation(Vec3::ZERO).with_scale(Vec3::ZERO),
                MeshMaterial3d(materials.add(StandardMaterial {
                    emissive: Color::srgb_u8(0, 100, 255).into(),
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


fn move_snake(
    mut commands: Commands,
    time: Res<Time>,
    mut snake_query: Query<(&mut Transform, &mut Snake), (With<Snake>, Without<Food>, Without<SnakeBody>)>,
    mut snake_bodies_query: Query<(&mut Transform, &mut SnakeBody),  (With<SnakeBody>, Without<Snake>, Without<Food>)>,
    food_query: Query<(Entity, &mut Transform), (With<Food>, Without<Snake>, Without<SnakeBody>)>,
    mut spawn_food_event_writer: EventWriter<SpawnFoodEvent>,
    mut spawn_snake_tail_event_writer: EventWriter<SpawnSnakeTail>,
) {

    let (mut transform, mut snake) = match snake_query.get_single_mut() {
        Ok(transform) => transform,
        Err(_) => return,
    };
    let snake_speed = snake.speed;
    snake.wait.tick(Duration::from_secs_f32(time.delta_secs() * snake_speed));


    if snake.wait.just_finished() {
        snake.pos = snake.target_position;
        transform.translation = snake.target_position;
        snake.target_position = snake.get_next_target();

        // update snake bodies
        let snake_pos = snake.pos;
        let mut prev_body_pos: Vec3 = Vec3::ZERO;
        for (body_index, entity) in snake.bodies.iter_mut().enumerate() {
            if let Ok((mut body_transform, mut body_data)) = snake_bodies_query.get_mut(*entity) {
                body_transform.translation = body_data.target_position;
                body_data.target_position = if body_index == 0 {
                    snake_pos
                } else {
                    prev_body_pos
                };
                prev_body_pos = body_transform.translation;
            }
        }

        // check for food collision
        if let Ok((entity, food_transform)) = food_query.get_single() {
            if (snake.pos.xz() - food_transform.translation.xz()).length() < 0.1 {
                // despawn food
                commands.entity(entity).despawn_recursive();
                // spawn new food
                spawn_food_event_writer.send(SpawnFoodEvent);
                spawn_snake_tail_event_writer.send(SpawnSnakeTail);
            }
        }
        
    } else {
        // update snake head pos
        transform.translation += (snake.target_position - snake.pos).normalize() * time.delta_secs() * snake.speed;
        // update snake bodies pos
        let snake_speed = snake.speed;
        for entity in snake.bodies.iter_mut() {
            if let Ok((mut body_transform, body_data)) = snake_bodies_query.get_mut(*entity) {
                let curr_pos = body_transform.translation;
                body_transform.translation += (body_data.target_position - curr_pos).normalize_or_zero() * time.delta_secs() * snake_speed;
            }
        }
    }
}

fn handle_direction_change(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Snake>,
) {
    for mut snake in query.iter_mut() {
        if keyboard.just_pressed(KeyCode::KeyA) {
            snake.direction = Direction::Right;
        } else if keyboard.just_pressed(KeyCode::KeyD) {
            snake.direction = Direction::Left;
        } else if keyboard.just_pressed(KeyCode::KeyW) {
            snake.direction = Direction::Up;
        } else if keyboard.just_pressed(KeyCode::KeyS) {
            snake.direction = Direction::Down;
        } 
    }
}

#[derive(Component)]
pub struct TailAppearAnimation {
    pub duration: f32,
    pub elapsed: f32,
}
impl Default for TailAppearAnimation {
    fn default() -> Self {
        Self {
            duration: 0.25,
            elapsed: 0.0,
        }
    }
}
#[derive(Component)]
pub struct FoodAnimation {
    pub duration: f32,
    pub elapsed: f32,
    pub amplitude: f32,
}
impl Default for FoodAnimation {
    fn default() -> Self {
        Self {
            duration: 2.0,
            elapsed: 0.0,
            amplitude: 0.5, // control how much the item moves up and down
        }
    }
}

fn update_food_animation(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut FoodAnimation)>,
) {
    for (mut transform, mut effect) in query.iter_mut() {
        // rotate around y axis
        effect.elapsed += time.delta_secs();
        let cycle_pos = effect.elapsed / effect.duration;
        let ease_val = ease_in_out_sine(cycle_pos);
        transform.rotate(Quat::from_rotation_y(time.delta_secs()));
        // move up down a little bit
        transform.translation.y = ease_val * effect.amplitude;
    }
}

fn update_tail_appear_animation (
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut TailAppearAnimation)>,
) {
    for (entity, mut transform, mut anim) in query.iter_mut() {
        anim.elapsed += time.delta_secs();
        let progress = (anim.elapsed / anim.duration).min(1.0);
        transform.scale = Vec3::splat(progress);
        if progress >= 1.0 {
            commands.entity(entity).remove::<TailAppearAnimation>();
        }
    }
}

pub fn ease_in_out_sine(t: f32) -> f32 {
    0.5 * (1.0 - (std::f32::consts::PI * t).cos())
}