use crate::level_instantiation::spawning::{
    GameObject, PrimedGameObjectSpawner, PrimedGameObjectSpawnerImplementor,
};
use crate::movement::general_movement::{CharacterAnimations, KinematicCharacterBundle, Model};
use crate::movement::navigation::Follower;
use crate::movement::physics::CustomCollider;
use crate::world_interaction::dialog::{DialogId, DialogTarget};
use anyhow::{Context, Result};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::TAU;

pub const HEIGHT: f32 = 1.;
pub const RADIUS: f32 = 0.4;

pub struct NpcSpawner;

impl PrimedGameObjectSpawnerImplementor for NpcSpawner {
    fn spawn<'a, 'b: 'a>(
        &self,
        spawner: &'b mut PrimedGameObjectSpawner<'_, '_, 'a>,
        _object: GameObject,
        transform: Transform,
    ) -> Result<Entity> {
        let gltf = spawner
            .gltf
            .get(&spawner.scenes.character)
            .context("Failed to load scene for NPC")?;

        Ok(spawner
            .commands
            .spawn((
                PbrBundle {
                    transform,
                    ..default()
                },
                Name::new("NPC"),
                KinematicCharacterBundle::capsule(HEIGHT, RADIUS),
                Follower,
                CharacterAnimations {
                    idle: spawner.animations.character_idle.clone(),
                    walk: spawner.animations.character_walking.clone(),
                    aerial: spawner.animations.character_running.clone(),
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    DialogTarget {
                        dialog_id: DialogId::new("follower"),
                    },
                    Name::new("NPC Dialog Collider"),
                    Collider::cylinder(HEIGHT / 2., RADIUS * 5.),
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::KINEMATIC_STATIC,
                    CustomCollider,
                ));
                parent.spawn((
                    SceneBundle {
                        scene: gltf.scenes[0].clone(),
                        transform: Transform {
                            translation: Vec3::new(0., -HEIGHT, 0.),
                            scale: Vec3::splat(0.012),
                            rotation: Quat::from_rotation_y(TAU / 2.),
                        },
                        ..default()
                    },
                    Model,
                    Name::new("NPC Model"),
                ));
            })
            .id())
    }
}