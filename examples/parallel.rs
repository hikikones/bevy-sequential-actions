use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    let actor = commands.spawn_actor(Vec3::ZERO, Quat::IDENTITY);

    commands
        .actions(actor)
        .add(WaitAction::new(0.5))
        .add_many(
            ExecutionMode::Parallel,
            [
                WaitAction::new(3.0).into_boxed(),
                MoveAction::new(Vec3::X).into_boxed(),
            ]
            .into_iter(),
        )
        .add(MoveAction::new(-Vec3::X));
}
