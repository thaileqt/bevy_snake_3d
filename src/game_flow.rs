use bevy::prelude::*;
use rand::thread_rng;
use rand::Rng;
use crate::player::*;
use crate::animation::*;
use crate::{GameState, GlobalAssets, MAP_SIZE};

pub struct GameFlowPlugin;
impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnFoodEvent>()
            .add_event::<SpawnSnakeTail>()
            .add_systems(OnEnter(GameState::InGame), |mut spawn_food_event: EventWriter<SpawnFoodEvent>| {
                spawn_food_event.send(SpawnFoodEvent);
            })
            .add_systems(Update, (
                spawn_food,
                spawn_snake_tail,
            ));
    }
}

#[derive(Event)]
pub struct SpawnSnakeTail;
#[derive(Event)]
pub struct SpawnFoodEvent;
#[derive(Component)]
pub struct Food;

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
