use bevy::prelude::*;
use rand::{thread_rng, Rng};
use crate::{camera::{CameraFollowTarget, TopdownCamera}, game_flow::Food, GameState, MAP_SIZE};

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
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);


fn setup_menu(mut commands: Commands) {
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
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
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
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(GameState::InGame);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
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

