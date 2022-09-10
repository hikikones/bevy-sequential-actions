use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::{RandomExt, RotateTowardsTransformExt};

pub struct RotateActionPlugin;

impl Plugin for RotateActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(rotate_system);
    }
}

pub struct RotateAction(Quat);

impl RotateAction {
    pub fn new(target: Quat) -> Self {
        Self(target)
    }
}

impl Action for RotateAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world.entity_mut(entity).insert_bundle(RotateBundle {
            target: Target(self.0),
            speed: Speed(4.0),
        });
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, _reason: StopReason) {
        world.entity_mut(entity).remove_bundle::<RotateBundle>();
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

fn rotate_system(
    mut rotate_q: Query<(&mut Transform, &Target, &Speed, &mut FinishedCount)>,
    time: Res<Time>,
) {
    for (mut transform, target, speed, mut finished) in rotate_q.iter_mut() {
        transform.rotate_towards(target.0, speed.0 * time.delta_seconds());

        if transform.rotation == target.0 {
            finished.increment();
        }
    }
}

pub struct RotateRandomAction {
    euler_min: Vec3,
    euler_max: Vec3,
    target: Option<Quat>,
}

impl RotateRandomAction {
    pub fn new(euler_min: Vec3, euler_max: Vec3) -> Self {
        Self {
            euler_min,
            euler_max,
            target: None,
        }
    }
}

impl Action for RotateRandomAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let target = if let Some(target) = self.target {
            target
        } else {
            let random = Quat::random(self.euler_min, self.euler_max);
            self.target = Some(random);
            random
        };

        world.entity_mut(entity).insert_bundle(RotateBundle {
            target: Target(target),
            speed: Speed(4.0),
        });
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
        world.entity_mut(entity).remove_bundle::<RotateBundle>();

        if let StopReason::Finished | StopReason::Canceled = reason {
            self.target = None;
        }
    }
}
