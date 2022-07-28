use bevy::prelude::*;
use bevy_sequential_actions::*;

pub struct WaitActionPlugin;

impl Plugin for WaitActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(wait);
    }
}

pub struct WaitAction {
    duration: f32,
    current: f32,
}

impl WaitAction {
    pub fn new(seconds: f32) -> Self {
        Self {
            duration: seconds,
            current: 0.0,
        }
    }
}

impl Action for WaitAction {
    fn start(
        &mut self,
        state: StartState,
        entity: Entity,
        world: &mut World,
        _commands: &mut ActionCommands,
    ) {
        match state {
            StartState::Init => {
                world.entity_mut(entity).insert(Wait(self.duration));
            }
            StartState::Resume => {
                world.entity_mut(entity).insert(Wait(self.current));
            }
        }
    }

    fn stop(&mut self, reason: StopReason, entity: Entity, world: &mut World) {
        match reason {
            StopReason::Completed | StopReason::Canceled => {
                world.entity_mut(entity).remove::<Wait>();
            }
            StopReason::Paused => {
                self.current = world.entity_mut(entity).remove::<Wait>().unwrap().0;
            }
        }
    }
}

#[derive(Component)]
struct Wait(f32);

fn wait(mut wait_q: Query<(Entity, &mut Wait)>, time: Res<Time>, mut commands: Commands) {
    for (actor, mut wait) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();
        if wait.0 <= 0.0 {
            commands.actions(actor).finish();
        }
    }
}
