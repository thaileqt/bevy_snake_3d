use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rand::thread_rng;
use rand::Rng;
use crate::player;
use crate::player::*;
use crate::animation::*;
use crate::utils::choose_random;
use crate::utils::choose_random_n;
use crate::utils::format_time;
use crate::utils::RandomChooser;
use crate::CubeState;
use crate::MapState;
use crate::{GameState, GlobalAssets, MAP_SIZE};


const BOOST_SPEED_AT: [usize; 5] = [
    5, 10, 20, 30, 40 // 
];

pub struct GameFlowPlugin;
impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnFoodEvent>()
            .add_event::<SpawnSnakeTail>()
            .add_event::<MapModifyEvent>()
            .add_systems(OnEnter(GameState::InGame), |mut spawn_food_event: EventWriter<SpawnFoodEvent>| {
                spawn_food_event.send(SpawnFoodEvent);
            })
            .add_systems(OnEnter(GameState::InGame), (spawn_hud))
            .add_systems(Update, (
                spawn_food,
            ))
            .add_systems(Update, (
                spawn_snake_tail,
                update_score,
                check_for_game_end,
                update_play_time,
                MapState::update,
                map_modify_event_listener,
            ).run_if(in_state(GameState::InGame)));
    }
}
#[derive(Event)]
pub struct MapModifyEvent {
    pub cube_count: usize,
}
#[derive(Event)]
pub struct SpawnSnakeTail;
#[derive(Event)]
pub struct SpawnFoodEvent;
#[derive(Component)]
pub struct Food;
#[derive(Component)]
pub struct ScoreText;
#[derive(Component)]
pub struct PlayTimeText;


#[derive(SystemParam)]
struct PositionQueryParam<'w, 's> {
    cube_query: Query<'w, 's, (Entity, &'static CubeState)>,
    player_query: Query<'w, 's, &'static Snake>,
    player_body_query: Query<'w, 's, &'static SnakeBody>,
    food_query: Query<'w, 's, &'static Transform, With<Food>>,
}

impl<'w, 's> PositionQueryParam<'w, 's> {
    fn get_empty_cubes(&self) -> Vec<(Entity, (usize, usize))> {
        let mut walkable_poses = self.cube_query.iter()
            .filter(|(_, c)| c.walkable)
            .map(|(entity, c)| (entity, c.pos))
            .collect::<Vec<_>>();
        // println!("{:?}", walkable_poses);
        let mut player_poses = Vec::new();

        if let Ok(player) = self.player_query.get_single() {
            player_poses.push((player.target_position.x as usize, player.target_position.z as usize));
            for body in self.player_body_query.iter() {
                player_poses.push((body.target_position.x as usize, body.target_position.z as usize));
            }
            // add some position around snake head
            let offset: usize = (player.speed.min(6.0) - 1.0) as usize;
            for i in 0..offset {
                for j in 0..offset {
                    let pos = (i, j);
                    if !player_poses.contains(&pos) {
                        player_poses.push(pos);
                    }
                }
            }
        }
        
        
        if let Ok(food_transform) = self.food_query.get_single() {
            let mut food_poses = Vec::new();
            food_poses.push((food_transform.translation.x as usize, food_transform.translation.z as usize)) ;
            let food_offset: usize = 3;
            for i in 0..food_offset {
                for j in 0..food_offset {
                    let pos = (i, j);
                    if !food_poses.contains(&pos) {
                        food_poses.push(pos);
                    }
                }
            }
            walkable_poses.retain(|&(_, pos)| !food_poses.contains(&pos));
        }

        walkable_poses
        .into_iter()
        .filter(|(_, x)| !player_poses.contains(x))
        // .map(|(entity, _)| entity)
        .collect::<Vec<_>>()
    }
}

fn map_modify_event_listener(
    mut ev_reader: EventReader<MapModifyEvent>,
    mut commands: Commands,
    map_state: Res<MapState>,
    game_assets: Res<GlobalAssets>,
    mut cubes_query: Query<&mut Transform, (Without<DeactiveCubeAnimation>, Without<Food>)>,
    inactive_cubes: Query<(Entity, &Transform),(With<DeactiveCubeAnimation>, Without<ActiveCubeAnimation>, Without<Food>)>,
    pos_param: PositionQueryParam,

) {
    for ev in ev_reader.read() {
        // let cubes = get_entities_to_deactive(&pos_param);
        let cubes = pos_param.get_empty_cubes().choose_random_n(ev.cube_count);
        let cubes = cubes.iter().map(|c| c.0).collect::<Vec<_>>();
        for e in cubes.iter() {
            if let Ok(transform) = cubes_query.get_mut(*e) {
                commands.entity(*e).insert(DeactiveCubeAnimation::new(
                    game_assets.map_cube_mat.clone(), 
                    transform.translation, 
                    transform.translation.with_y(transform.translation.y + 1.0)
                ));
                // cube_state.walkable = false;
            }
           
        }

        inactive_cubes.iter().for_each(|(entity, transform)| {
            commands.entity(entity).remove::<DeactiveCubeAnimation>();
            commands.entity(entity).insert(ActiveCubeAnimation::new(
                game_assets.map_cube_mat_emission.clone(),
                transform.translation,
                transform.translation.with_y(transform.translation.y - 1.0)
            ));
        });
    }
}

fn spawn_hud(
    mut commands: Commands,
) {
    commands.spawn((
       
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Column,
            top: Val::Px(0.),
            left: Val::Px(0.),
            margin: UiRect::all(Val::Px(15.0)),
            ..default()
        },
        
    )).with_children(|parent| {
        parent.spawn((
            PlayTimeText,
            Text::new("time: 0"),
        ));
        parent.spawn((
            ScoreText,
            Text::new("score: 0"),
        ));
       
        
    });
}
fn update_score(
    mut map_state: ResMut<MapState>,
    mut query: Query<&mut Text, With<ScoreText>>,
    mut evs: EventReader<SpawnSnakeTail>,
) {
    for _ in evs.read() {
        let mut score_text = match query.get_single_mut() {
            Ok(z) => z,
            Err(_) => return,
        };
        map_state.score += 1;
        score_text.0 = format!("score: {}", map_state.score);
    }
    
}

fn update_play_time(
    map_state: Res<MapState>,
    mut query: Query<&mut Text, With<PlayTimeText>>,
) {
    let mut score_text = match query.get_single_mut() {
        Ok(z) => z,
        Err(_) => return,
    };
    score_text.0 = format!("time: {}", format_time(map_state.time_elapsed));
}

fn spawn_food(
    mut commands: Commands,
    game_assets: Res<GlobalAssets>,
    map_state: Res<MapState>,
    // cube_query: Query<&CubeState>,
    // player_query: Query<&Snake>,
    // player_body_query: Query<&SnakeBody>,
    mut spawn_food_event: EventReader<SpawnFoodEvent>,
    // food_query: Query<&Transform, With<Food>>,
    pos_param: PositionQueryParam,
) {
    for _ in spawn_food_event.read() {

        // let empty_poses = get_empty_positions(&cube_query, &player_query, &player_body_query, &food_query);
        let empty_poses = pos_param.get_empty_cubes()
            .iter()
            .map(|x|x.1)
            .collect::<Vec<_>>();
        if let Some(spawn_pos) = empty_poses.choose_random() {
            commands.spawn((
                Food,
                FoodAnimation::default(),
                Mesh3d(game_assets.food.clone()),
                Transform::from_xyz(spawn_pos.0 as f32, 0.0, spawn_pos.1 as f32),
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
        } else {
            warn!("No available position found for spawning FOOD!")
        }
    
       
    }
}

// fn get_empty_positions(
//     cube_query: &Query<&CubeState>,
//     player_query: &Query<&Snake>,
//     player_body_query: &Query<&SnakeBody>,
//     food_query: &Query<&Transform, With<Food>>,
// ) -> Vec<(usize, usize)> {
//     let mut walkable_poses = cube_query.iter()
//             .filter(|c| c.walkable)
//             .map(|c| c.pos)
//             .collect::<Vec<_>>();
//     // println!("{:?}", walkable_poses);
//     let mut player_poses = Vec::new();

//     if let Ok(player) = player_query.get_single() {
//         player_poses.push((player.target_position.x as usize, player.target_position.z as usize));
//         for body in player_body_query.iter() {
//             player_poses.push((body.target_position.x as usize, body.target_position.z as usize));
//         }
//     }
//     if let Ok(food_transform) = food_query.get_single() {
//         let food_pos = (food_transform.translation.x as usize, food_transform.translation.z as usize) ;
//         walkable_poses.retain(|&pos| pos != food_pos);
//     }

//     walkable_poses
//     .into_iter()
//     .filter(|x| !player_poses.contains(x))
//     .collect::<Vec<_>>()

// }


fn get_entities_to_deactive(
    pos_params: &PositionQueryParam,
) -> Vec<(Entity, (usize, usize))> {
    let mut walkable_poses = pos_params.cube_query.iter()
            .filter(|(_, c)| c.walkable)
            .map(|(entity, c)| (entity, c.pos))
            .collect::<Vec<_>>();
    // println!("{:?}", walkable_poses);
    let mut player_poses = Vec::new();

    if let Ok(player) = pos_params.player_query.get_single() {
        player_poses.push((player.target_position.x as usize, player.target_position.z as usize));
        for body in pos_params.player_body_query.iter() {
            player_poses.push((body.target_position.x as usize, body.target_position.z as usize));
        }
        // add some position around snake head
        let offset: usize = (player.speed.min(6.0) - 1.0) as usize;
        for i in 0..offset {
            for j in 0..offset {
                let pos = (i, j);
                if !player_poses.contains(&pos) {
                    player_poses.push(pos);
                }
            }
        }
    }
    
    
    if let Ok(food_transform) = pos_params.food_query.get_single() {
        let mut food_poses = Vec::new();
        food_poses.push((food_transform.translation.x as usize, food_transform.translation.z as usize)) ;
        let food_offset: usize = 3;
        for i in 0..food_offset {
            for j in 0..food_offset {
                let pos = (i, j);
                if !food_poses.contains(&pos) {
                    food_poses.push(pos);
                }
            }
        }
        walkable_poses.retain(|&(_, pos)| !food_poses.contains(&pos));
    }

    walkable_poses
    .into_iter()
    .filter(|(_, x)| !player_poses.contains(x))
    // .map(|(entity, _)| entity)
    .collect::<Vec<_>>()

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
            // parent.spawn((
            //     SpotLight {
            //         range: 10.0,
            //         intensity: 500_000.0,
            //         shadows_enabled: false,
            //         ..default()
            //     },
            //     Transform::from_xyz(0.0, 3.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            // ));
        })
        .id();
        snake.bodies.push(entity);

        // Check for speed boost
        BOOST_SPEED_AT
            .iter()
            .any(|&num_body| num_body == snake.bodies.len())
            .then(||snake.speed += 1.0);
    }
   
}

fn check_for_game_end(
    map_state: Res<MapState>,
    player: Query<&Snake>,
    cube_query: Query<&CubeState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let player = match player.get_single() {
        Ok(player) => player,
        Err(_) => return,
    };
    // Check for player walking outside map
    if player.target_position.x < 0.0 || player.target_position.z < 0.0 || player.target_position.x >= MAP_SIZE as f32 || player.target_position.z >= MAP_SIZE as f32 {
        info!("Player outside");
        return;
    }

    // Check collision between player and non-walkable cube
    let should_end: bool = cube_query.iter()
        .filter(|cube| !cube.walkable)
        .any(|cube| cube.pos == (player.target_position.x as usize, player.target_position.z as usize));
    if should_end {
        info!("Shoud end");
        return;
    }

    
    
}