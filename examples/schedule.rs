use std::time::Duration;

use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 10.0,
        )))
        .add_schedule(EvenSchedule, Schedule::new())
        .add_schedule(OddSchedule, Schedule::new())
        .add_plugin(ScheduleRunnerPlugin)
        .add_startup_system(setup)
        .add_systems((
            run_custom_schedule,
            ActionHandler::check_actions::<With<EvenMarker>>().in_schedule(EvenSchedule),
            ActionHandler::check_actions::<With<OddMarker>>().in_schedule(OddSchedule),
        ))
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
        world.send_event(AppExit);
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
