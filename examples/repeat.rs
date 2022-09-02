use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*, extensions::LookRotationExt};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    for i in 0..4 {
        let start = Vec3::new(-3.0, 0.0, -2.0) + Vec3::X * i as f32 * 2.0;
        let end = start + Vec3::Z * 4.0;

        let actor = commands.spawn_actor(start, Quat::look_rotation(Vec3::Z, Vec3::Y));

        let repeat = match i {
            3 => AddRepeat::Infinite,
            _ => AddRepeat::Finite(i),
        };

        commands
            .actions(actor)
            .config(AddConfig {
                order: AddOrder::Back,
                start: true,
                repeat,
            })
            .add(WaitAction::new(0.5))
            .add(MoveAction::new(end))
            .add(WaitAction::new(0.5))
            .add(MoveAction::new(start));
    }
}
