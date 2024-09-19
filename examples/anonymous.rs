use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins((ScheduleRunnerPlugin::default(), SequentialActionsPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(ActionsBundle::new()).id();
}

struct AnonymousAction<IsFinished, OnAdd, OnStart, OnStop, OnRemove, OnDrop>
where
    IsFinished: Fn(Entity, &World) -> bool,
    OnAdd: FnMut(Entity, &mut World),
    OnStart: FnMut(Entity, &mut World) -> bool,
    OnStop: FnMut(Entity, &mut World, StopReason),
    OnRemove: FnMut(Entity, &mut World),
    OnDrop: FnOnce(Entity, &mut World, DropReason),
{
    is_finished: IsFinished,
    on_add: OnAdd,
    on_start: OnStart,
    on_stop: OnStop,
    on_remove: OnRemove,
    on_drop: OnDrop,
}

impl<IsFinished, OnAdd, OnStart, OnStop, OnRemove, OnDrop> Action
    for AnonymousAction<IsFinished, OnAdd, OnStart, OnStop, OnRemove, OnDrop>
where
    IsFinished: Fn(Entity, &World) -> bool + Send + Sync + 'static,
    OnAdd: FnMut(Entity, &mut World) + Send + Sync + 'static,
    OnStart: FnMut(Entity, &mut World) -> bool + Send + Sync + 'static,
    OnStop: FnMut(Entity, &mut World, StopReason) + Send + Sync + 'static,
    OnRemove: FnMut(Entity, &mut World) + Send + Sync + 'static,
    OnDrop: FnOnce(Entity, &mut World, DropReason) + Send + Sync + 'static,
{
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        (self.is_finished)(agent, world)
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        (self.on_add)(agent, world)
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        (self.on_start)(agent, world)
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        (self.on_stop)(agent, world, reason)
    }

    fn on_remove(&mut self, agent: Entity, world: &mut World) {
        (self.on_remove)(agent, world)
    }

    fn on_drop(self: Box<Self>, agent: Entity, world: &mut World, reason: DropReason) {
        (self.on_drop)(agent, world, reason);
    }
}
