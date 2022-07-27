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
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            current: duration,
        }
    }
}

impl Action for WaitAction {
    fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world.entity_mut(entity).insert(Wait(self.duration));
    }

    fn finish(&mut self, entity: Entity, world: &mut World) {
        world.entity_mut(entity).remove::<Wait>();
    }

    fn cancel(&mut self, entity: Entity, world: &mut World) {
        self.finish(entity, world);
    }

    fn pause(&mut self, entity: Entity, world: &mut World) {
        self.current = world.entity_mut(entity).remove::<Wait>().unwrap().0;
        println!("PAUSE WAIT: {}", self.current);
    }

    fn resume(&mut self, entity: Entity, world: &mut World) {
        world.entity_mut(entity).insert(Wait(self.current));
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
