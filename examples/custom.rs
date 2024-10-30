use std::{marker::PhantomData, time::Duration};

use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use bevy_ecs::{prelude::*, query::QueryFilter, schedule::ScheduleLabel};

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .init_schedule(EvenSchedule)
        .init_schedule(OddSchedule)
        .add_plugins((
            ScheduleRunnerPlugin::run_loop(Duration::from_millis(100)),
            // Add custom plugin for the even schedule
            CustomSequentialActionsPlugin::new(EvenSchedule)
                .with_cleanup()
                .with_filter::<With<EvenMarker>>(),
            // Add custom plugin for the odd schedule
            CustomSequentialActionsPlugin::new(OddSchedule)
                // No cleanup for odd agents
                .with_filter::<With<OddMarker>>(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, run_custom_schedules)
        .run();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
struct EvenSchedule;

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
struct OddSchedule;

#[derive(Component)]
struct EvenMarker;

#[derive(Component)]
struct OddMarker;

fn setup(mut commands: Commands) {
    // Spawn agent with even marker for even schedule
    let agent_even = commands.spawn((SequentialActions, EvenMarker)).id();
    commands
        .actions(agent_even)
        .add(PrintForeverAction::new(format!(
            "Even: is_finished is called every even frame for agent {agent_even}."
        )));

    // Spawn agent with odd marker for odd schedule
    let agent_odd = commands.spawn((SequentialActions, OddMarker)).id();
    commands
        .actions(agent_odd)
        .add(PrintForeverAction::new(format!(
            "Odd:  is_finished is called every odd  frame for agent {agent_odd}."
        )));
}

fn run_custom_schedules(
    world: &mut World,
    mut frame_count: Local<u32>,
    mut agent_q: Local<QueryState<Entity, With<SequentialActions>>>,
) {
    if *frame_count % 2 == 0 {
        world.run_schedule(EvenSchedule);
    } else {
        world.run_schedule(OddSchedule);
    }

    if *frame_count == 10 {
        for agent in agent_q.iter(world).collect::<Vec<_>>() {
            world.despawn(agent);
        }
        world.send_event(AppExit::Success);
    }

    *frame_count += 1;
}

struct PrintForeverAction {
    message: String,
    agent: Entity,
}

impl PrintForeverAction {
    fn new(message: String) -> Self {
        Self {
            message,
            agent: Entity::PLACEHOLDER,
        }
    }
}

impl Action for PrintForeverAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        println!("{}", self.message);
        false
    }
    fn on_start(&mut self, agent: Entity, _world: &mut World) -> bool {
        self.agent = agent;
        false
    }
    fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {}
    fn on_drop(self: Box<Self>, _agent: Option<Entity>, _world: &mut World, _reason: DropReason) {
        // Notice that this is not called for odd agents when despawned...
        println!("Dropping action for agent {}...", self.agent);
    }
}

/// Custom plugin for sequential actions.
///
/// Action queue advancement will run in the specified schedule `S`,
/// and only for agents matching the specified query filter `F`.
/// With `cleanup` enabled, an observer will trigger for despawned agents
/// that ensures any remaining action is cleaned up.
struct CustomSequentialActionsPlugin<S: ScheduleLabel, F: QueryFilter> {
    schedule: S,
    cleanup: bool,
    filter: PhantomData<F>,
}

impl<S: ScheduleLabel> CustomSequentialActionsPlugin<S, ()> {
    const fn new(schedule: S) -> Self {
        Self {
            schedule,
            cleanup: false,
            filter: PhantomData,
        }
    }

    const fn with_cleanup(mut self) -> Self {
        self.cleanup = true;
        self
    }

    fn with_filter<F: QueryFilter>(self) -> CustomSequentialActionsPlugin<S, F> {
        CustomSequentialActionsPlugin {
            schedule: self.schedule,
            cleanup: self.cleanup,
            filter: PhantomData,
        }
    }
}

impl<S: ScheduleLabel, F: QueryFilter> CustomSequentialActionsPlugin<S, F> {
    fn check_actions_exclusive(
        world: &mut World,
        mut finished: Local<Vec<Entity>>,
        mut agent_q: Local<QueryState<(Entity, &CurrentAction), F>>,
    ) {
        // Collect all agents with finished action
        finished.extend(agent_q.iter(world).filter_map(|(agent, current_action)| {
            current_action
                .as_ref()
                .and_then(|action| action.is_finished(agent, world).then_some(agent))
        }));

        // Do something with the finished list if you want.
        // Perhaps sort by some identifier for deterministic behavior.

        // Advance the action queue
        for agent in finished.drain(..) {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Finished, world);
            SequentialActionsPlugin::start_next_action(agent, world);
        }
    }
}

impl Default for CustomSequentialActionsPlugin<Last, ()> {
    fn default() -> Self {
        Self::new(Last).with_cleanup()
    }
}

impl<S: ScheduleLabel + Clone, F: QueryFilter + Send + Sync + 'static> Plugin
    for CustomSequentialActionsPlugin<S, F>
{
    fn build(&self, app: &mut App) {
        // Add system for advancing action queue to specified schedule
        app.add_systems(self.schedule.clone(), Self::check_actions_exclusive);

        // Add observers for cleanup of actions when despawning agents
        if self.cleanup {
            app.add_observer(CurrentAction::on_remove_trigger::<F>)
                .add_observer(ActionQueue::on_remove_trigger::<F>);
        }
    }
}
