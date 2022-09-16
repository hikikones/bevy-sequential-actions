use bevy::prelude::*;
use bevy_sequential_actions::*;

use super::IntoValue;

pub struct WaitActionPlugin;

impl Plugin for WaitActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(wait).add_system(check_wait.after(wait));
    }
}

pub struct WaitAction<F>
where
    F: IntoValue<f32>,
{
    duration: F,
    executor: Option<Entity>,
    current: Option<f32>,
}

impl<F> WaitAction<F>
where
    F: IntoValue<f32>,
{
    pub fn new(seconds: F) -> Self {
        Self {
            duration: seconds,
            executor: None,
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

        self.executor = Some(
            world
                .spawn()
                .insert_bundle(WaitBundle {
                    wait: Wait(duration),
                    agent: Agent(agent),
                })
                .id(),
        );
    }

    fn on_stop(&mut self, _agent: Entity, world: &mut World, reason: StopReason) {
        let executor = self.executor.unwrap();

        if let StopReason::Paused = reason {
            self.current = Some(world.get::<Wait>(executor).unwrap().0);
        }

        world.despawn(executor);
    }
}

#[derive(Bundle)]
struct WaitBundle {
    wait: Wait,
    agent: Agent,
}

#[derive(Component)]
struct Wait(f32);

#[derive(Component)]
struct Agent(Entity);

fn wait(mut wait_q: Query<&mut Wait>, time: Res<Time>) {
    for mut wait in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();
    }
}

fn check_wait(wait_q: Query<(&Wait, &Agent)>, mut finished_q: Query<&mut ActionFinished>) {
    for (wait, agent) in wait_q.iter() {
        if wait.0 <= 0.0 {
            finished_q.get_mut(agent.0).unwrap().confirm();
        }
    }
}
