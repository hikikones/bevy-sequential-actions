use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::{FromEulerXYZExt, FromLookExt, RotateTowardsTransformExt};

use super::IntoValue;

pub struct RotateActionPlugin;

impl Plugin for RotateActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(rotation)
            .add_system(check_rotation.after(rotation));
    }
}

pub struct RotateAction<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    config: RotateConfig<V, F>,
    bundle: Option<RotateBundle>,
}

impl<V, F> RotateAction<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    pub fn new(config: RotateConfig<V, F>) -> Self {
        Self {
            config,
            bundle: None,
        }
    }
}

pub struct RotateConfig<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    pub target: RotateType<V>,
    pub speed: F,
}

pub enum RotateType<V>
where
    V: IntoValue<Vec3>,
{
    Look(V),
    Euler(V),
}

impl<V, F> Action for RotateAction<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    fn on_start(&mut self, id: ActionIds, world: &mut World, _commands: &mut ActionCommands) {
        world.entity_mut(id.executant()).insert(RotateMarker);

        let rotate_bundle = self.bundle.take().unwrap_or_else(|| {
            let target = match &self.config.target {
                RotateType::Look(dir) => Quat::from_look(dir.value(), Vec3::Y),
                RotateType::Euler(angles) => Quat::from_euler_xyz(angles.value()),
            };
            RotateBundle {
                target: Target(target),
                speed: Speed(self.config.speed.value()),
            }
        });

        world.entity_mut(id.agent()).insert_bundle(rotate_bundle);
    }

    fn on_stop(&mut self, id: ActionIds, world: &mut World, reason: StopReason) {
        let bundle = world.entity_mut(id.agent()).remove_bundle::<RotateBundle>();

        if let StopReason::Paused = reason {
            self.bundle = bundle;
        }
    }
}

#[derive(Bundle)]
struct RotateBundle {
    target: Target,
    speed: Speed,
}

#[derive(Component)]
struct Target(Quat);

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct RotateMarker;

fn rotation(mut rotate_q: Query<(&mut Transform, &Target, &Speed)>, time: Res<Time>) {
    for (mut transform, target, speed) in rotate_q.iter_mut() {
        transform.rotate_towards(target.0, speed.0 * time.delta_seconds());
    }
}

fn check_rotation(
    mut check_q: Query<(&ActionAgent, &mut ActionFinished), With<RotateMarker>>,
    transform_q: Query<(&Transform, &Target)>,
) {
    for (agent, mut finished) in check_q.iter_mut() {
        if let Ok((transform, target)) = transform_q.get(agent.id()) {
            if transform.rotation == target.0 {
                finished.set(true);
            }
        }
    }
}
