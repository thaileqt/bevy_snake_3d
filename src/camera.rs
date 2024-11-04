use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, on_follow_target_added)
            .add_systems(Update, (
                smooth_follow, 
                update_camera,
            ).chain())
            ;
    }
}
#[derive(Component)]
pub struct CameraFollowTarget;

#[derive(Debug, Component)]
pub struct TopdownCamera {
    pub offset: Vec3,
    pub current_velocity: Vec3,
    pub smooth_time: f32,
    pub pos: Vec3,
    pub quat: Quat,
}

impl TopdownCamera {
    pub fn with_offset(offset: Vec3) -> Self {
        Self {
            offset,
            current_velocity: Vec3::ZERO,
            smooth_time: 0.3,
            pos: offset,
            quat: Quat::IDENTITY,
        }
    }
}

impl Default for TopdownCamera {
    fn default() -> Self {
        let offset = Vec3::new(0.0, 14.0, 24.0);
        Self {
            offset,
            current_velocity: Vec3::ZERO,
            smooth_time: 0.3,
            pos: offset,
            quat: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4),
        }
    }
}


fn on_follow_target_added(
    player_q: Query<&Transform, (Added<CameraFollowTarget>, Without<TopdownCamera>)>,
    mut camera_q: Query<(&mut Transform, &mut TopdownCamera), (With<TopdownCamera>, Without<CameraFollowTarget>)>,
) {
    if let (
        Ok((mut camera_transform, mut topdown_camera)), 
        Ok(player_transform)
    ) = (camera_q.get_single_mut(), player_q.get_single()) {
        camera_transform.translation = player_transform.translation + topdown_camera.offset;
        camera_transform.rotation = camera_transform.looking_at(player_transform.translation, Vec3::Y).rotation;

        topdown_camera.pos = camera_transform.translation;
        topdown_camera.quat = camera_transform.rotation;
    }
}

fn smooth_follow(
    time: Res<Time>,
    target_query: Query<&Transform, With<CameraFollowTarget>>,
    mut camera_query: Query<&mut TopdownCamera, (With<TopdownCamera>, Without<CameraFollowTarget>)>,
) {
    if let (
        Ok(target), 
        Ok(mut topdown_camera)
    ) = (target_query.get_single(), camera_query.get_single_mut()) {
        let target_position = target.translation + topdown_camera.offset;
        let (new_pos, new_vel) = smooth_damp(
            topdown_camera.pos,
            target_position,
            topdown_camera.current_velocity,
            topdown_camera.smooth_time,
            time.delta_secs(),
        );
        topdown_camera.pos = new_pos;
        topdown_camera.current_velocity = new_vel;

    }
}

fn update_camera(
    mut camera_query: Query<(&mut Transform, &TopdownCamera), With<TopdownCamera>>,
) {
    if let Ok((mut camera_transform, topdown_camera)) = camera_query.get_single_mut() {
        camera_transform.translation = topdown_camera.pos;
        camera_transform.rotation = topdown_camera.quat;
    }
}

/// Simulate SmoothDamp function from Unity
fn smooth_damp(
    current: Vec3,
    target: Vec3,
    current_velocity: Vec3,
    smooth_time: f32,
    delta_time: f32,
) -> (Vec3, Vec3) {
    let smooth_time = smooth_time.max(0.0001); // prevent division by zero
    let omega = 2.0 / smooth_time;
    let x = omega * delta_time;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let change = current - target;
    let temp = (current_velocity + omega * change) * delta_time;
    let new_velocity = (current_velocity - omega * temp) * exp;
    let new_position = target + (change + temp) * exp;
    (new_position, new_velocity)
}
