use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::{FromLookExt, MoveTowardsTransformExt};

use super::IntoValue;

pub struct MoveActionPlugin;

impl Plugin for MoveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement)
            .add_system(check_movement.after(movement))
            .add_system(rotation);
    }
}

pub struct MoveAction<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    config: MoveConfig<V, F>,
    bundle: Option<MoveBundle>,
}

pub struct MoveConfig<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    pub target: V,
    pub speed: F,
    pub rotate: bool,
}

impl<V, F> MoveAction<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    pub fn new(config: MoveConfig<V, F>) -> Self {
        Self {
            config,
            bundle: None,
        }
    }
}

impl<V, F> Action for MoveAction<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    fn on_start(&mut self, id: ActionIds, world: &mut World, _commands: &mut ActionCommands) {
        world.entity_mut(id.executant()).insert(MoveMarker);

        let move_bundle = self.bundle.take().unwrap_or(MoveBundle {
            target: Target(self.config.target.value()),
            speed: Speed(self.config.speed.value()),
        });

        world.entity_mut(id.agent()).insert_bundle(move_bundle);

        if self.config.rotate {
            world.entity_mut(id.agent()).insert(RotateMarker);
        }
    }

    fn on_stop(&mut self, id: ActionIds, world: &mut World, reason: StopReason) {
        let bundle = world.entity_mut(id.agent()).remove_bundle::<MoveBundle>();

        if let StopReason::Paused = reason {
            self.bundle = bundle;
        }

        if self.config.rotate {
            world.entity_mut(id.agent()).remove::<RotateMarker>();
        }
    }
}

#[derive(Bundle)]
struct MoveBundle {
    target: Target,
    speed: Speed,
}

#[derive(Component)]
struct Target(Vec3);

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct MoveMarker;

#[derive(Component)]
struct RotateMarker;

fn movement(mut move_q: Query<(&mut Transform, &Target, &Speed)>, time: Res<Time>) {
    for (mut transform, target, speed) in move_q.iter_mut() {
        transform.move_towards(target.0, speed.0 * time.delta_seconds());
    }
}

fn check_movement(
    mut check_q: Query<(&ActionAgent, &mut ActionFinished), With<MoveMarker>>,
    transform_q: Query<(&Transform, &Target)>,
) {
    for (agent, mut finished) in check_q.iter_mut() {
        if let Ok((transform, target)) = transform_q.get(agent.entity()) {
            if transform.translation == target.0 {
                finished.set(true);
            }
        }
    }
}

fn rotation(
    mut rotate_q: Query<(&mut Transform, &Target, &Speed), With<RotateMarker>>,
    time: Res<Time>,
) {
    for (mut transform, target, speed) in rotate_q.iter_mut() {
        let dir = target.0 - transform.translation;

        if dir == Vec3::ZERO {
            continue;
        }

        transform.rotation = Quat::slerp(
            transform.rotation,
            Quat::from_look(dir, Vec3::Y),
            speed.0 * 2.0 * time.delta_seconds(),
        );
    }
}
