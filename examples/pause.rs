use std::time::Duration;

use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 10.0,
        )))
        .add_plugin(ScheduleRunnerPlugin)
        .add_plugin(SequentialActionsPlugin::default())
        .add_startup_system(setup)
        .add_systems((count, frame_logic).chain())
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(ActionsBundle::new()).id();
    commands.actions(agent).add(CountForeverAction);
}

fn frame_logic(
    mut frame: Local<u32>,
    mut commands: Commands,
    agent_q: Query<Entity, With<ActionQueue>>,
) {
    const PAUSE_FRAME: u32 = 10;
    const RESUME_FRAME: u32 = PAUSE_FRAME * 2;
    const EXIT_FRAME: u32 = PAUSE_FRAME * 3;
    const MISSING_FRAMES: u32 = RESUME_FRAME - PAUSE_FRAME;

    println!("Frame: {}", *frame);

    if *frame == PAUSE_FRAME {
        println!("\nPAUSE\n");
        commands.actions(agent_q.single()).pause();
    }
    if *frame == RESUME_FRAME {
        println!("\nRESUME\n");
        commands.actions(agent_q.single()).execute();
    }
    if *frame == EXIT_FRAME {
        println!(
            "\nEXIT - Frame is now at {}, and Count should be {} (missing {} frames)",
            *frame,
            EXIT_FRAME - MISSING_FRAMES,
            MISSING_FRAMES,
        );
        commands.add(|world: &mut World| {
            world.send_event(AppExit);
        });
    }

    *frame += 1;
}

struct CountForeverAction;

impl Action for CountForeverAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> Finished {
        Finished(false)
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> Finished {
        let mut agent = world.entity_mut(agent);

        if agent.contains::<Paused>() {
            agent.remove::<Paused>();
        } else {
            agent.insert(Count::default());
        }

        Finished(false)
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        match reason {
            StopReason::Finished | StopReason::Canceled => {
                world.entity_mut(agent).remove::<Count>();
            }
            StopReason::Paused => {
                world.entity_mut(agent).insert(Paused);
            }
        }
    }
}

#[derive(Default, Component)]
struct Count(u32);

#[derive(Component)]
struct Paused;

fn count(mut count_q: Query<&mut Count, Without<Paused>>) {
    for mut count in &mut count_q {
        println!("Count: {}", count.0);
        count.0 += 1;
    }
}
