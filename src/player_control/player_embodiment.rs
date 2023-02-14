use crate::movement::general_movement::{Jump, Velocity, Walker};
use crate::player_control::actions::Actions;
use crate::player_control::camera::{IngameCamera, IngameCameraKind};
use crate::util::trait_extension::{F32Ext, Vec2Ext, Vec3Ext};
use crate::GameState;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

pub struct PlayerEmbodimentPlugin;

/// This plugin handles player related stuff like general_movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerEmbodimentPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Timer>()
            .register_type::<Player>()
            .register_type::<PlayerSensor>()
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(handle_jump.after("set_actions").before("apply_jumping"))
                    .with_system(
                        handle_horizontal_movement
                            .after("set_actions")
                            .after("update_camera_transform")
                            .before("apply_walking"),
                    )
                    .with_system(
                        set_camera_actions
                            .label("set_camera_actions")
                            .after("set_actions")
                            .before("update_camera_transform")
                            .before("apply_walking"),
                    )
                    .with_system(
                        handle_camera_kind
                            .label("handle_camera_kind")
                            .after("switch_camera_kind")
                            .before("apply_walking"),
                    )
                    .with_system(handle_speed_effects.label("handle_speed_effects")),
            );
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Player;

#[derive(Debug, Clone, Eq, PartialEq, Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub struct PlayerSensor;

fn handle_jump(actions: Res<Actions>, mut player_query: Query<&mut Jump, With<Player>>) {
    for mut jump in &mut player_query {
        if actions.player.jump {
            jump.requested = true;
        }
    }
}

fn handle_horizontal_movement(
    actions: Res<Actions>,
    mut player_query: Query<&mut Walker, With<Player>>,
    camera_query: Query<&IngameCamera>,
) {
    let camera = match camera_query.iter().next() {
        Some(camera) => camera,
        None => return,
    };
    let movement = match actions.player.movement {
        Some(movement) => movement,
        None => return,
    };

    let forward = camera.forward().xz().normalize();
    let sideward = forward.perp();
    let forward_action = forward * movement.y;
    let sideward_action = sideward * movement.x;
    let direction = (forward_action + sideward_action).x0y().normalize();

    for mut walker in &mut player_query {
        walker.direction = Some(direction);
        walker.sprinting = actions.player.sprint;
    }
}

fn set_camera_actions(actions: Res<Actions>, mut camera_query: Query<&mut IngameCamera>) {
    let mut camera = match camera_query.iter_mut().next() {
        Some(camera) => camera,
        None => return,
    };

    camera.actions = actions.camera.clone();
}

fn handle_camera_kind(
    mut with_player: Query<(&mut Transform, &mut Visibility), With<Player>>,
    camera_query: Query<(&Transform, &IngameCamera), Without<Player>>,
) {
    for (camera_transform, camera) in camera_query.iter() {
        for (mut player_transform, mut visibility) in with_player.iter_mut() {
            match camera.kind {
                IngameCameraKind::FirstPerson(_) => {
                    let up = camera.up();
                    let horizontal_direction = camera_transform.forward().split(up).horizontal;
                    let looking_target = player_transform.translation + horizontal_direction;
                    player_transform.look_at(looking_target, up);
                    visibility.is_visible = false;
                }
                IngameCameraKind::ThirdPerson(_) | IngameCameraKind::FixedAngle(_) => {
                    visibility.is_visible = true
                }
            }
        }
    }
}

fn handle_speed_effects(
    velocities: Query<&Velocity, With<Player>>,
    mut projections: Query<&mut Projection, With<IngameCamera>>,
) {
    for velocity in velocities.iter() {
        let speed_squared = velocity.0.length_squared();
        for mut projection in projections.iter_mut() {
            if let Projection::Perspective(ref mut perspective) = projection.deref_mut() {
                const MAX_SPEED_FOR_FOV: f32 = 10.;
                const MIN_FOV: f32 = 0.75;
                const MAX_FOV: f32 = 1.7;
                let scale = (speed_squared / MAX_SPEED_FOR_FOV.squared())
                    .min(1.0)
                    .squared();
                perspective.fov = MIN_FOV + (MAX_FOV - MIN_FOV) * scale;
            }
        }
    }
}
