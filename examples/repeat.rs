use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*, extensions::FromLookExt};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    let range = 0..4;
    let last = (range.len() - 1) as u32;
    let x = last as f32;

    for i in range {
        let start = Vec3::new(-x, 0.0, -2.0) + Vec3::X * i as f32 * 2.0;
        let end = start + Vec3::Z * 4.0;

        let agent = commands.spawn_agent(start, Quat::from_look(Vec3::Z, Vec3::Y));

        let repeat = match i {
            i if i < last => Repeat::Amount(i),
            _ => Repeat::Forever,
        };

        commands
            .actions(agent)
            .config(AddConfig {
                order: AddOrder::Back,
                start: true,
                repeat,
            })
            .add(WaitAction::new(0.5))
            .add(MoveAction::new(MoveConfig {
                target: end,
                speed: 4.0,
                rotate: true,
            }))
            .add(WaitAction::new(0.5))
            .add(MoveAction::new(MoveConfig {
                target: start,
                speed: 4.0,
                rotate: true,
            }));
    }
}
