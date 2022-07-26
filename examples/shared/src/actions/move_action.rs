use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::{LookRotationExt, MoveTowardsExt};

pub struct MoveActionPlugin;

impl Plugin for MoveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_action).add_system(rotate);
    }
}

pub struct MoveAction(Vec3);

impl MoveAction {
    pub fn new(target: Vec3) -> Self {
        Self(target)
    }
}

impl Action for MoveAction {
    fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world.entity_mut(entity).insert_bundle(MoveBundle {
            target: Target(self.0),
            speed: Speed(4.0),
        });
    }

    fn finish(&mut self, entity: Entity, world: &mut World) {
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

fn move_action(
    mut q: Query<(Entity, &mut Transform, &Target, &Speed)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, target, speed) in q.iter_mut() {
        if transform.move_towards(target.0, speed.0 * time.delta_seconds()) {
            commands.actions(entity).next();
        }
    }
}

fn rotate(mut q: Query<(&mut Transform, &Target, &Speed)>, time: Res<Time>) {
    for (mut transform, target, speed) in q.iter_mut() {
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
