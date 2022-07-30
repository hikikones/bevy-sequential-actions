use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::{LookRotationExt, MoveTowardsExt};

use super::{random_vec3, ACTIONS_STAGE};

pub struct MoveActionPlugin;

impl Plugin for MoveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            ACTIONS_STAGE,
            SystemSet::new()
                .with_system(move_system)
                .with_system(rotate_system),
        );
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

fn move_system(
    mut move_q: Query<(Entity, &mut Transform, &Target, &Speed)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, target, speed) in move_q.iter_mut() {
        if transform.move_towards(target.0, speed.0 * time.delta_seconds()) {
            commands.actions(entity).finish();
        }
    }
}

fn rotate_system(mut move_q: Query<(&mut Transform, &Target, &Speed)>, time: Res<Time>) {
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
    target: Option<Vec3>,
}

impl MoveRandomAction {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min,
            max,
            target: None,
        }
    }
}

impl Action for MoveRandomAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let target = if let Some(target) = self.target {
            target
        } else {
            let random = random_vec3(self.min, self.max);
            self.target = Some(random);
            random
        };

        world.entity_mut(entity).insert_bundle(MoveBundle {
            target: Target(target),
            speed: Speed(4.0),
        });
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
        world.entity_mut(entity).remove_bundle::<MoveBundle>();

        if let StopReason::Finished | StopReason::Canceled = reason {
            self.target = None;
        }
    }
}
