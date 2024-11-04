use std::time::Duration;

use bevy::prelude::*;

use crate::GameState;

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_food_animation,
            update_tail_appear_animation,
        ));
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