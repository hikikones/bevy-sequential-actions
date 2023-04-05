use std::time::Duration;

use bevy_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 10.0,
        )))
        .add_schedule(CustomSchedule, Schedule::new())
        .add_plugin(ScheduleRunnerPlugin)
        // Default plugin checks actions every frame
        .add_plugin(SequentialActionsPlugin::default())
        // Use a marker component for custom scheduling
        .add_plugin(SequentialActionsPlugin::<CustomMarker>::custom(
            |app: &mut App| {
                app.add_systems(
                    SequentialActionsPlugin::<CustomMarker>::get_systems()
                        .in_schedule(CustomSchedule),
                );
            },
        ))
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
}

fn setup(mut commands: Commands) {
    // Use default bundle for default schedule
    let agent_update = commands.spawn(ActionsBundle::default()).id();
    commands
        .actions(agent_update)
        .add(PrintForeverAction("Update"));

    // Use custom marker for custom schedule
    let agent_frame = commands.spawn(ActionsBundle::<CustomMarker>::new()).id();
    commands
        .actions(agent_frame)
        .add(PrintForeverAction("\nCustom\n"));
}

struct PrintForeverAction(&'static str);

impl Action for PrintForeverAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        println!("{}", self.0);
        false
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) {}
    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
