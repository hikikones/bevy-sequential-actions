use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::{FromLookExt, MoveTowardsTransformExt};

use super::IntoValue;

pub struct MoveActionPlugin;

impl Plugin for MoveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement).add_system(rotation);
    }
}

pub struct MoveAction<T>
where
    T: IntoValue<Vec3>,
{
    target: T,
    current: Option<Vec3>,
}

impl<T> MoveAction<T>
where
    T: IntoValue<Vec3>,
{
    pub fn new(target: T) -> Self {
        Self {
            target,
            current: None,
        }
    }
}

impl<T> Action for MoveAction<T>
where
    T: IntoValue<Vec3>,
{
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let target = self.current.take().unwrap_or(self.target.value());

        world.entity_mut(entity).insert_bundle(MoveBundle {
            target: Target(target),
            speed: Speed(4.0),
        });
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
        let bundle = world.entity_mut(entity).remove_bundle::<MoveBundle>();

        if let StopReason::Paused = reason {
            self.current = Some(bundle.unwrap().target.0);
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

fn movement(
    mut move_q: Query<(&mut Transform, &Target, &Speed, &mut ActionFinished)>,
    time: Res<Time>,
) {
    for (mut transform, target, speed, mut finished) in move_q.iter_mut() {
        transform.move_towards(target.0, speed.0 * time.delta_seconds());

        if transform.translation == target.0 {
            finished.confirm();
        }
    }
}

fn rotation(mut move_q: Query<(&mut Transform, &Target, &Speed)>, time: Res<Time>) {
    for (mut transform, target, speed) in move_q.iter_mut() {
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
