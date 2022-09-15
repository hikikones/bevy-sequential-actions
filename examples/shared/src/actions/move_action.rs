use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::{LookRotationExt, MoveTowardsTransformExt, RandomExt};

pub struct MoveActionPlugin;

impl Plugin for MoveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement).add_system(rotation);
    }
}

pub struct MoveAction(Vec3);

impl MoveAction {
    pub fn new(target: Vec3) -> Self {
        Self(target)
    }
}

impl Action for MoveAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world.entity_mut(entity).insert_bundle(MoveBundle {
            target: Target(self.0),
            speed: Speed(4.0),
        });
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, _reason: StopReason) {
        world.entity_mut(entity).remove_bundle::<MoveBundle>();
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
            Quat::look_rotation(dir, Vec3::Y),
            speed.0 * 2.0 * time.delta_seconds(),
        );
    }
}

pub struct MoveRandomAction {
    min: Vec3,
    max: Vec3,
    move_action: MoveAction,
}

impl MoveRandomAction {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min,
            max,
            move_action: MoveAction::new(Vec3::random(min, max)),
        }
    }
}

impl Action for MoveRandomAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        self.move_action.on_start(entity, world, _commands);
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
        self.move_action.on_stop(entity, world, reason);

        if let StopReason::Finished | StopReason::Canceled = reason {
            self.move_action.0 = Vec3::random(self.min, self.max);
        }
    }
}
