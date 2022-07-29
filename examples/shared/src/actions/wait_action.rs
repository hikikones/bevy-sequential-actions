use bevy::prelude::*;
use bevy_sequential_actions::*;

use super::ACTIONS_STAGE;

pub struct WaitActionPlugin;

impl Plugin for WaitActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(ACTIONS_STAGE, SystemSet::new().with_system(wait));
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
        world
            .entity_mut(entity)
            .insert(Wait(self.current.unwrap_or(self.duration)));
    }

    fn on_finish(&mut self, entity: Entity, world: &mut World) {
        world.entity_mut(entity).remove::<Wait>();
        self.current = None;
    }

    fn on_cancel(&mut self, entity: Entity, world: &mut World) {
        self.on_finish(entity, world);
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World) {
        let wait = world.entity_mut(entity).remove::<Wait>().unwrap();
        self.current = Some(wait.0);
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
