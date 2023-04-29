use std::time::Duration;

use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 10.0,
        )))
        .add_schedule(CustomSchedule, Schedule::new())
        .add_plugin(ScheduleRunnerPlugin)
        // Add default plugin for default schedule
        .add_plugin(SequentialActionsPlugin::default())
        // Schedule manually with marker component for custom schedule
        .add_systems(
            SequentialActionsPlugin::<CustomMarker>::get_systems().in_schedule(CustomSchedule),
        )
        .add_startup_system(setup)
        .add_system(run_custom_schedule)
        .run();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
struct CustomSchedule;

#[derive(Default, Component)]
struct CustomMarker;

fn run_custom_schedule(world: &mut World, mut frame_count: Local<u32>) {
    *frame_count += 1;

    if *frame_count % 10 == 0 {
        world.run_schedule(CustomSchedule);
    }

    if *frame_count == 30 {
        world.send_event(AppExit);
    }
}

fn setup(mut commands: Commands) {
    // Use default bundle for default schedule
    let agent_default = commands.spawn(ActionsBundle::default()).id();
    commands.actions(agent_default).add(PrintForeverAction(
        "Default: is_finished is called every frame in CoreSet::Last",
    ));

    // Use custom marker for custom schedule
    let agent_custom = commands.spawn(ActionsBundle::<CustomMarker>::new()).id();
    commands.actions(agent_custom).add(PrintForeverAction(
        "\nCustom: is_finished is called every 10th frame in CoreSet::Update\n",
    ));
}

struct PrintForeverAction(&'static str);

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
