use std::time::Duration;

use bevy_app::{AppExit, ScheduleRunnerPlugin, prelude::*};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins((
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 10.0)),
            SequentialActionsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (count, frame_logic).chain())
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(SequentialActions).id();
    commands.actions(agent).add(CountForeverAction);
}

fn frame_logic(
    mut frame: Local<u32>,
    mut commands: Commands,
    agent_q: Single<Entity, With<SequentialActions>>,
) {
    const PAUSE_FRAME: u32 = 10;
    const RESUME_FRAME: u32 = PAUSE_FRAME * 2;
    const EXIT_FRAME: u32 = PAUSE_FRAME * 3;
    const MISSING_FRAMES: u32 = RESUME_FRAME - PAUSE_FRAME;

    println!("Frame: {}", *frame);

    let agent = agent_q.entity();

    if *frame == PAUSE_FRAME {
        println!("\nPAUSE\n");
        commands.actions(agent).pause();
    }
    if *frame == RESUME_FRAME {
        println!("\nRESUME\n");
        commands.actions(agent).execute();
    }
    if *frame == EXIT_FRAME {
        println!(
            "\nEXIT - Frame is now at {}, and Count should be {} (missing {} frames)",
            *frame,
            EXIT_FRAME - MISSING_FRAMES,
            MISSING_FRAMES,
        );
        commands.queue(|world: &mut World| {
            world.write_message(AppExit::Success);
        });
    }

    *frame += 1;
}

struct CountForeverAction;

impl Action for CountForeverAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        false
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        let mut agent = world.entity_mut(agent);

        if agent.contains::<Paused>() {
            agent.remove::<Paused>();
        } else {
            agent.insert(Count::default());
        }

        false
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        match reason {
            StopReason::Finished | StopReason::Canceled => {
                world.entity_mut(agent.unwrap()).remove::<Count>();
            }
            StopReason::Paused => {
                world.entity_mut(agent.unwrap()).insert(Paused);
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
