use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::RotateTowardsExt;

use super::{random_quat, ACTIONS_STAGE};

pub struct RotateActionPlugin;

impl Plugin for RotateActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(ACTIONS_STAGE, SystemSet::new().with_system(rotate_system));
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
    mut move_q: Query<(Entity, &mut Transform, &Target, &Speed)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, target, speed) in move_q.iter_mut() {
        if transform.rotate_towards(target.0, speed.0 * time.delta_seconds()) {
            commands.actions(entity).finish();
        }
    }
}

pub struct RotateRandomAction {
    min: Vec3,
    max: Vec3,
    target: Option<Quat>,
}

impl RotateRandomAction {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min,
            max,
            target: None,
        }
    }
}

impl Action for RotateRandomAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let target = if let Some(target) = self.target {
            target
        } else {
            let random = random_quat(self.min, self.max);
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
