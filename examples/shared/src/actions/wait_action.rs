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
    fn on_start(&mut self, id: ActionIds, world: &mut World, _commands: &mut ActionCommands) {
        let duration = self.current.take().unwrap_or(self.duration.value());
        world.entity_mut(id.executant()).insert(Wait(duration));
    }

    fn on_stop(&mut self, id: ActionIds, world: &mut World, reason: StopReason) {
        if let StopReason::Paused = reason {
            self.current = Some(world.get::<Wait>(id.executant()).unwrap().0);
        }
    }
}

#[derive(Component)]
struct Wait(f32);

fn wait(mut wait_q: Query<(&mut Wait, &mut ActionFinished)>, time: Res<Time>) {
    for (mut wait, mut finished) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();

        if wait.0 <= 0.0 {
            finished.set(true);
        }
    }
}
