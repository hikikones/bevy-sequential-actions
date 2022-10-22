use bevy::prelude::*;
use bevy_sequential_actions::*;

use super::IntoValue;

pub struct WaitActionPlugin;

impl Plugin for WaitActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(wait);
    }
}

pub struct WaitAction<F>
where
    F: IntoValue<f32>,
{
    duration: F,
    current: Option<f32>,
}

impl<F> WaitAction<F>
where
    F: IntoValue<f32>,
{
    pub fn new(seconds: F) -> Self {
        Self {
            duration: seconds,
            current: None,
        }
    }
}

impl<F> Action for WaitAction<F>
where
    F: IntoValue<f32>,
{
    fn on_start(&mut self, agent: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let duration = self.current.take().unwrap_or(self.duration.value());
        world.entity_mut(agent).insert(Wait(duration));
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        let wait = world.entity_mut(agent).remove::<Wait>();

        if let StopReason::Paused = reason {
            self.current = Some(wait.unwrap().0);
        }
    }
}

#[derive(Component)]
struct Wait(f32);

fn wait(mut wait_q: Query<(&mut Wait, &mut AgentState)>, time: Res<Time>) {
    for (mut wait, mut finished) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();

        if wait.0 <= 0.0 {
            finished.confirm_and_reset();
        }
    }
}
