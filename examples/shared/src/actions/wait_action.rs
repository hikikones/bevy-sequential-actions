use bevy::prelude::*;
use bevy_sequential_actions::*;

use super::IntoValue;

pub struct WaitActionPlugin;

impl Plugin for WaitActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(wait).add_system(check_wait.after(wait));
    }
}

pub struct WaitAction<T>
where
    T: IntoValue<f32>,
{
    duration: T,
    executor: Option<Entity>,
    current: Option<f32>,
}

impl<T> WaitAction<T>
where
    T: IntoValue<f32>,
{
    pub fn new(seconds: T) -> Self {
        Self {
            duration: seconds,
            executor: None,
            current: None,
        }
    }
}

impl<T> Action for WaitAction<T>
where
    T: IntoValue<f32>,
{
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let duration = self.current.take().unwrap_or(self.duration.value());
        let executor = world
            .spawn()
            .insert_bundle(WaitBundle {
                wait: Wait(duration),
                actor: ActionActor(entity),
            })
            .id();
        self.executor = Some(executor);
    }

    fn on_stop(&mut self, _entity: Entity, world: &mut World, reason: StopReason) {
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
    actor: ActionActor,
}

#[derive(Component)]
struct Wait(f32);

#[derive(Component)]
struct ActionActor(Entity);

fn wait(mut wait_q: Query<&mut Wait>, time: Res<Time>) {
    for mut wait in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();
    }
}

fn check_wait(wait_q: Query<(&Wait, &ActionActor)>, mut finished_q: Query<&mut ActionFinished>) {
    for (wait, actor) in wait_q.iter() {
        if wait.0 <= 0.0 {
            finished_q.get_mut(actor.0).unwrap().confirm();
        }
    }
}
