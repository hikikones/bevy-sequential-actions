use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins((ScheduleRunnerPlugin::default(), SequentialActionsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, countdown)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(ActionsBundle::new()).id();
    commands
        .actions(agent)
        // Add a single action
        .add(DemoAction)
        // Add multiple actions
        .add_many(actions![
            PrintAction("hello"),
            PrintAction("there"),
            CountdownAction::new(10)
        ])
        // Add an anonymous action with a closure
        .add(|_agent, world: &mut World| -> bool {
            // on_start
            world.send_event(AppExit::Success);
            false
        });
}

struct DemoAction;

impl Action for DemoAction {
    // Required method
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        println!("is_finished: called every frame in the Last schedule");
        true
    }

    // Required method
    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
        println!("on_start: called when an action is started");

        // Returning true here marks the action as already finished,
        // and will immediately advance the action queue.
        false
    }

    // Required method
    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {
        println!("on_stop: called when an action is stopped");
    }

    // Optional method (empty by default)
    fn on_add(&mut self, _agent: Entity, _world: &mut World) {
        println!("on_add: called when an action is added to the queue");
    }

    // Optional method (empty by default)
    fn on_remove(&mut self, _agent: Entity, _world: &mut World) {
        println!("on_remove: called when an action is removed from the queue");
    }

    // Optional method (empty by default)
    fn on_drop(self: Box<Self>, _agent: Entity, _world: &mut World, _reason: DropReason) {
        println!("on_drop: the last method to be called with full ownership");
    }
}

struct PrintAction(&'static str);

impl Action for PrintAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
        println!("{}", self.0);
        true
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

struct CountdownAction {
    count: u32,
    current: Option<u32>,
}

impl CountdownAction {
    const fn new(count: u32) -> Self {
        Self {
            count,
            current: None,
        }
    }
}

impl Action for CountdownAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        let current_count = world.get::<Countdown>(agent).unwrap().0;
        println!("Countdown: {current_count}");

        // Determine if countdown has reached zero
        current_count == 0
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        // Take current count (if paused), or use full count
        let count = self.current.take().unwrap_or(self.count);

        // Run the countdown system on the agent
        world.entity_mut(agent).insert(Countdown(count));

        // Is action already finished?
        self.is_finished(agent, world)
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        // Take the countdown component from the agent
        let countdown = world.entity_mut(agent).take::<Countdown>();

        // Store current count when paused
        if reason == StopReason::Paused {
            self.current = countdown.unwrap().0.into();
        }
    }
}

#[derive(Component)]
struct Countdown(u32);

fn countdown(mut countdown_q: Query<&mut Countdown>) {
    for mut countdown in &mut countdown_q {
        countdown.0 -= 1;
    }
}
