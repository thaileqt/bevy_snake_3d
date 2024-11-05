use bevy::{color::palettes::{css::{GREEN, WHITE}, tailwind::{GREEN_100, GREEN_950}}, prelude::*};
use rand::{thread_rng, Rng};
use crate::{camera::{CameraFollowTarget, TopdownCamera}, game_flow::Food, utils::format_time, GameState, GlobalAssets, MapState, MAP_SIZE};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(Update, (
                menu, 
                simulate_camera_movement,
            ).run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}
#[derive(Component)]
struct FakePlayer {
    pub target_position: Vec3,
    pub speed: f32,
}

#[derive(Resource)]
struct MenuData {
    button_entity: Entity,
}

const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.15, 0.4);



fn setup_menu(mut commands: Commands, map_state: Res<MapState>) {
    // setup camera movement
    let mut rng = thread_rng();
    let x_rand = rng.gen_range(0..MAP_SIZE);
    let z_rand = rng.gen_range(0..MAP_SIZE);
    commands.spawn((
        Transform::from_xyz(x_rand as f32, 0.0, z_rand as f32),
        GlobalTransform::default(),
        CameraFollowTarget,
        FakePlayer {
            target_position: Vec3::new(x_rand as f32, 0.0, z_rand as f32),
            speed: 1.0,
        },
    ));

    // setup ui
    let button_entity = commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                top: Val::Px(0.),
                left: Val::Px(0.),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7).into()),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Snake Game"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(25.),
                    ..default()
                }
            ));
            if map_state.score != 0 {
                parent.spawn((
                    Text::new(format!("[last game] score {} / time {}", map_state.score, format_time(map_state.time_elapsed) )),
                    TextFont {
                        font_size: 25.0,
                        ..default()
                    },
                ));
            }
            
            
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    BorderColor(Color::WHITE.with_alpha(0.)),
                ))
                .with_children(|parent| {
                    
                    parent.spawn((
                        Text::new("Play"),
                        TextFont {
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        })
        .id();
    commands.insert_resource(MenuData { button_entity });
}

fn menu(
    mut commands: Commands,
    game_assets: Res<GlobalAssets>,
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = WHITE.with_alpha(1.0).into();
                commands.spawn((
                    AudioPlayer::<AudioSource>(game_assets.button_click.clone()),
                    PlaybackSettings::DESPAWN,
                ));
                next_state.set(GameState::InGame);
            }
            Interaction::Hovered => {
                *color = WHITE.with_alpha(1.0).into();
            }
            Interaction::None => {
                *color = WHITE.with_alpha(0.0).into();
            }
        }
    }
}

fn cleanup_menu(
    mut commands: Commands, 
    menu_data: Res<MenuData>, 
    food_query: Query<Entity, With<Food>>,
    fake_player: Query<Entity, With<FakePlayer>>,
) {
    commands.entity(menu_data.button_entity).despawn_recursive();
    for entity in food_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.entity(fake_player.single()).despawn_recursive();
}


fn simulate_camera_movement(
    time: Res<Time>,
    mut fake_player: Query<(&mut Transform, &mut FakePlayer)>,
) {
    let (mut player_transform, mut player) = match fake_player.get_single_mut() {
        Ok(player) => player,
        Err(_) => return,
    };

    if (player_transform.translation - player.target_position).length() < 0.1 {
        let mut rng = thread_rng();
        let x_rand = rng.gen_range(0..MAP_SIZE);
        let z_rand = rng.gen_range(0..MAP_SIZE);
        player.target_position = Vec3::new(x_rand as f32, 0.0, z_rand as f32);
    } else {
        let player_position = player_transform.translation;
        player_transform.translation += (player.target_position - player_position).normalize() * player.speed * time.delta_secs();
    }
}

