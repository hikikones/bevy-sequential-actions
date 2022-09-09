use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::RandomExt;

use super::CHECK_ACTIONS_STAGE;

pub struct WaitActionPlugin;

impl Plugin for WaitActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(wait_system);
        // app.add_system(wait_system).add_system(check_wait_status);
        // .add_system_to_stage(CHECK_ACTIONS_STAGE, check_wait_status);
    }
}

pub struct WaitAction {
    duration: f32,
    current: Option<f32>,
}

impl WaitAction {
    pub fn new(seconds: f32) -> Self {
        Self {
            duration: seconds,
            current: None,
        }
    }
}

impl Action for WaitAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let duration = self.current.unwrap_or(self.duration);
        world.entity_mut(entity).insert(Wait(duration));
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
        match reason {
            StopReason::Finished | StopReason::Canceled => {
                world.entity_mut(entity).remove::<Wait>();
                self.current = None;
            }
            StopReason::Paused => {
                let wait = world.entity_mut(entity).remove::<Wait>().unwrap();
                self.current = Some(wait.0);
            }
        }
    }
}

#[derive(Component)]
struct Wait(f32);

fn wait_system(mut wait_q: Query<(&mut Wait, &mut ActionFinished)>, time: Res<Time>) {
    for (mut wait, mut finished) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();

        if wait.0 <= 0.0 {
            finished.confirm();
        }
    }
}

fn check_wait_status(wait_q: Query<(Entity, &Wait)>, mut commands: Commands) {
    for (entity, wait) in wait_q.iter() {
        if wait.0 <= 0.0 {
            commands.actions(entity).finish();
        }
    }
}

pub struct WaitRandomAction {
    min: f32,
    max: f32,
    current: Option<f32>,
}

impl WaitRandomAction {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            current: None,
        }
    }
}

impl Action for WaitRandomAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let duration = self.current.unwrap_or(f32::random(self.min, self.max));
        world.entity_mut(entity).insert(Wait(duration));
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
        match reason {
            StopReason::Finished | StopReason::Canceled => {
                world.entity_mut(entity).remove::<Wait>();
                self.current = None;
            }
            StopReason::Paused => {
                let wait = world.entity_mut(entity).remove::<Wait>().unwrap();
                self.current = Some(wait.0);
            }
        }
    }
}
