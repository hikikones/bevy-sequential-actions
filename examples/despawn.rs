use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugin(ScheduleRunnerPlugin)
        .add_plugin(SequentialActionsPlugin::default())
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(ActionsBundle::default()).id();
    commands.actions(agent).add_many(actions![
        PrintAction("First action"),
        DespawnAction,
        PrintAction("This action should not run"),
    ]);
}

struct DespawnAction;

impl Action for DespawnAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        // Don't advance the action queue by always returning false.
        false
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) {
        println!("Despawn!");
        world.despawn(agent);
        world.send_event(AppExit);
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

struct PrintAction(&'static str);

impl Action for PrintAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) {
        println!("{}", self.0);
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
