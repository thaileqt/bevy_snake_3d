use std::{ops::{Add, Mul, Sub}, time::Duration};
use crate::{game_flow::BodyIndex, utils::*, CubeState, Snake, FOOD_COLOR};
use bevy::prelude::*;

use crate::{GameState, GlobalAssets};

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_food_animation,
            update_tail_appear_animation,
            update_deactive_cube_animation,
            update_active_cube_animation,
            update_dead_effect,
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

#[derive(Component)]
pub struct DeactiveCubeAnimation {
    pub warn_duration: f32,
    pub warn_elapsed: f32,

    pub duration: f32,
    pub elapsed: f32,
    pub from: Vec3,
    pub to: Vec3,
}
impl DeactiveCubeAnimation {
    pub fn new(from: Vec3, to: Vec3) -> Self {
        Self {
            warn_duration: 1.5,
            warn_elapsed: 0.0,
            duration: 0.5,
            elapsed: 0.0,
            from,
            to,
        }
    }
}

fn update_deactive_cube_animation (
    time:           Res<Time>,
    game_assets:    Res<GlobalAssets>,
    mut query:      Query<(&mut Transform, &mut CubeState, &mut MeshMaterial3d<StandardMaterial> , &mut DeactiveCubeAnimation)>,
) {
    for (mut transform, mut cube, mut mat, mut anim) in query.iter_mut() {
        // Phase 1
        if anim.warn_elapsed < anim.warn_duration {
            anim.warn_elapsed += time.delta_secs();
            // let progress = (anim.warn_elapsed / anim.warn_duration).min(1.0);
            if anim.warn_elapsed < 1.0 {
                *mat = MeshMaterial3d(game_assets.red_mat.clone());
            }
            else {
                // *mat = MeshMaterial3d(anim.origin_mat.clone());
                *mat = MeshMaterial3d(game_assets.map_cube_mat_emission.clone());
            }
        } else {
            // Phase 2
            // cube.walkable = false;
            anim.elapsed += time.delta_secs();
            let progress = (anim.elapsed / anim.duration).min(1.0);
            if progress < 1.0 {
                transform.translation = lerp(anim.from, anim.to, progress);
            }
            if progress >= 1.0 {
                transform.translation = anim.to;
                // commands.entity(entity).remove::<DeactiveCubeAnimation>();
                // mat = 
                // if let Some(material) = materials.get_mut(mat) {
                //     *material = game_assets.map_cube_mat_emission.clone();
                // }
                // *mat = MeshMaterial3d(game_assets.map_cube_mat_emission.clone());
                cube.walkable = false;
                // commands.entity(entity).insert(MeshMaterial3d(game_assets.map_cube_mat_emission.clone()));
            }
        }
    }
}



#[derive(Component)]
pub struct ActiveCubeAnimation {
    pub warn_duration: f32,
    pub warn_elapsed: f32,

    pub duration: f32,
    pub elapsed: f32,
    pub from: Vec3,
    pub to: Vec3,
}
impl ActiveCubeAnimation {
    pub fn new(from: Vec3, to: Vec3) -> Self {
        Self {
            warn_duration: 1.5,
            warn_elapsed: 0.0,
            duration: 0.5,
            elapsed: 0.0,
            from,
            to,
        }
    }
}

fn update_active_cube_animation (
    mut commands:   Commands,
    time:           Res<Time>,
    game_assets:    Res<GlobalAssets>,
    mut query:      Query<
        (Entity, &mut Transform, &mut CubeState, &mut MeshMaterial3d<StandardMaterial>, &mut ActiveCubeAnimation)
    >,
) {
    for (entity, mut transform, mut cube, mut mat, mut anim) in query.iter_mut() {
        // Phase 1
        if anim.warn_elapsed < anim.warn_duration {
            anim.warn_elapsed += time.delta_secs();
    
        } else {
            // Phase 2
            anim.elapsed += time.delta_secs();
            let progress = (anim.elapsed / anim.duration).min(1.0);
            transform.translation = lerp(anim.from, anim.to, progress);
            if progress >= 1.0 {
                transform.translation = anim.to;
                commands.entity(entity).remove::<ActiveCubeAnimation>();
                *mat = MeshMaterial3d(game_assets.map_cube_mat.clone());
                cube.walkable = true;
            }
        }
    }
}

#[derive(Component)]
pub struct DeadEffect {
    timer: Timer,
}

impl DeadEffect {
    pub fn new(timer: Timer) -> Self {
        Self { timer }
    }
}


fn update_dead_effect (
    mut commands:   Commands,
    time:           Res<Time>,
    game_assets:    Res<GlobalAssets>,
    mut query:      Query<
        (Entity, &mut Transform, &mut MeshMaterial3d<StandardMaterial>, &BodyIndex, &mut DeadEffect)
    >,
    player_query:   Query<&Snake>,
) {
    for (entity, mut transform, mut mat, body_index, mut anim) in query.iter_mut() {
        anim.timer.tick(Duration::from_secs_f32(time.delta_secs()));
        if anim.timer.just_finished() {
            *mat = MeshMaterial3d(game_assets.red_mat.clone());
            commands.entity(entity).remove::<DeadEffect>();

            if player_query.single().bodies.len() == body_index.0 + 1 {
                commands.spawn((
                    AudioPlayer::<AudioSource>(game_assets.game_over.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }
    }
}