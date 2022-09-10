use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::RandomExt;

pub struct WaitActionPlugin;

impl Plugin for WaitActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(wait_system);
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

fn wait_system(mut wait_q: Query<(&mut Wait, &mut FinishedCount)>, time: Res<Time>) {
    for (mut wait, mut finished) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();

        if wait.0 <= 0.0 {
            finished.increment();
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
