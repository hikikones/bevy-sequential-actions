use std::time::Duration;

use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .init_schedule(EvenSchedule)
        .init_schedule(OddSchedule)
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 10.0,
        )))
        .add_systems(Startup, setup)
        .add_systems(Update, run_custom_schedule)
        .add_systems(
            EvenSchedule,
            SequentialActionsPlugin::check_actions::<With<EvenMarker>>,
        )
        .add_systems(
            OddSchedule,
            SequentialActionsPlugin::check_actions::<With<OddMarker>>,
        )
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

fn run_custom_schedule(world: &mut World, mut frame_count: Local<u32>) {
    if *frame_count % 2 == 0 {
        world.run_schedule(EvenSchedule);
    } else {
        world.run_schedule(OddSchedule);
    }

    if *frame_count == 10 {
        world.send_event(AppExit::Success);
    }

    *frame_count += 1;
}

fn setup(mut commands: Commands) {
    // Spawn agent with even marker for even schedule
    let agent_even = commands.spawn((ActionsBundle::new(), EvenMarker)).id();
    commands.actions(agent_even).add(PrintForeverAction(format!(
        "Even: is_finished is called every even frame for agent {agent_even:?}."
    )));

    // Spawn agent with odd marker for odd schedule
    let agent_odd = commands.spawn((ActionsBundle::new(), OddMarker)).id();
    commands.actions(agent_odd).add(PrintForeverAction(format!(
        "Odd:  is_finished is called every odd  frame for agent {agent_odd:?}."
    )));
}

struct PrintForeverAction(String);

impl Action for PrintForeverAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        println!("{}", self.0);
        false
    }
    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
        false
    }
    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
