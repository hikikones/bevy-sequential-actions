use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::RotateTowardsTransformExt;

use super::IntoValue;

pub struct RotateActionPlugin;

impl Plugin for RotateActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(rotation);
    }
}

pub struct RotateAction<T>
where
    T: IntoValue<Vec3>,
{
    euler: T,
    current: Option<Quat>,
}

impl<T> RotateAction<T>
where
    T: IntoValue<Vec3>,
{
    pub fn new(target: T) -> Self {
        Self {
            euler: target,
            current: None,
        }
    }
}

impl<T> Action for RotateAction<T>
where
    T: IntoValue<Vec3>,
{
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let target = self.current.take().unwrap_or_else(|| {
            let euler = self.euler.value();
            Quat::from_euler(EulerRot::XYZ, euler.x, euler.y, euler.z)
        });
        world.entity_mut(entity).insert_bundle(RotateBundle {
            target: Target(target),
            speed: Speed(4.0),
        });
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
        let bundle = world.entity_mut(entity).remove_bundle::<RotateBundle>();
        if let StopReason::Paused = reason {
            self.current = Some(bundle.unwrap().target.0);
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
