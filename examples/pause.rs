use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_sequential_actions::*;
use shared::{Countdown, CountdownAction, SharedActionsPlugin};

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(100))),
            SequentialActionsPlugin,
            SharedActionsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, frame_logic)
        .run();
}

const PAUSE_FRAME: u32 = 10;
const RESUME_FRAME: u32 = PAUSE_FRAME * 2;
const EXIT_FRAME: u32 = PAUSE_FRAME * 3;
const MISSING_FRAMES: u32 = RESUME_FRAME - PAUSE_FRAME;

fn setup(mut commands: Commands) {
    let agent = commands.spawn(SequentialActions).id();
    commands
        .actions(agent)
        .add(CountdownAction::new(EXIT_FRAME + 1));
}

fn frame_logic(
    mut frame: Local<u32>,
    mut commands: Commands,
    agent_q: Single<Entity, With<SequentialActions>>,
    countdown_q: Query<&Countdown>,
) {
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
        let countdown = countdown_q.get(agent).unwrap().current_count();
        println!(
            "\nEXIT - Frame is now at {} and Countdown is {} (missing {} frames)",
            *frame, countdown, MISSING_FRAMES,
        );
        commands.queue(|world: &mut World| {
            world.write_message(AppExit::Success);
        });
    }

    *frame += 1;
}
