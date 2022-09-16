use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::{FromVec3Ext, RotateTowardsTransformExt};

use super::IntoValue;

pub struct RotateActionPlugin;

impl Plugin for RotateActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(rotation);
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

pub struct RotateConfig<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    pub target: V,
    pub speed: F,
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

impl<V, F> Action for RotateAction<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    fn on_start(&mut self, agent: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let rotate_bundle = self.bundle.take().unwrap_or(RotateBundle {
            target: Target(Quat::from_vec3(self.config.target.value())),
            speed: Speed(self.config.speed.value()),
        });

        world.entity_mut(agent).insert_bundle(rotate_bundle);
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        let bundle = world.entity_mut(agent).remove_bundle::<RotateBundle>();

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

fn rotation(
    mut rotate_q: Query<(&mut Transform, &Target, &Speed, &mut ActionFinished)>,
    time: Res<Time>,
) {
    for (mut transform, target, speed, mut finished) in rotate_q.iter_mut() {
        transform.rotate_towards(target.0, speed.0 * time.delta_seconds());

        if transform.rotation == target.0 {
            finished.confirm();
        }
    }
}
