use std::time::Duration;

use bevy::prelude::*;

use crate::{Food, SpawnFoodEvent, SpawnSnakeTail};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            move_snake,
            handle_direction_change,
        ));
    }
}


#[derive(Component, Clone)]
pub struct Snake {
    pub pos: Vec3,
    pub direction: Direction,
    pub target_position: Vec3,
    pub speed: f32,
    pub wait: Timer,
    pub bodies: Vec<Entity>,
}
#[derive(Component, Clone)]
pub struct SnakeBody {
    pub target_position: Vec3,
}

impl SnakeBody {
    pub fn new(at: Vec3) -> Self {
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
pub enum Direction { Up, Down, Left, Right }
impl Direction {
    pub fn norm(&self) -> Vec3 {
        match self {
            Direction::Up => Vec3::new(0.0 ,0.0, -1.0),
            Direction::Down => Vec3::new(0.0, 0.0 ,1.0),
            Direction::Left => Vec3::new(1.0, 0.0, 0.0),
            Direction::Right => Vec3::new(-1.0, 0.0, 0.0),
        }
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