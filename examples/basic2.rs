use bevy::prelude::*;

use shared::playground::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PlaygroundPlugin)
        // .add_startup_system_to_stage(StartupStage::PreStartup, a)
        // .add_startup_system_set_to_stage(
        //     StartupStage::PreStartup,
        //     SystemSet::new().label("load_assets").with_system(a),
        // )
        // .add_startup_system_set_to_stage(
        //     StartupStage::PreStartup,
        //     SystemSet::new()
        //         .after("load_assets")
        //         .with_system(b)
        //         .with_system(c),
        // )
        .run();
}

fn setup(player_q: Query<Entity, With<Player>>, mut commands: Commands) {}

fn a() {
    println!("A");
}

fn b() {
    println!("B");
}

fn c() {
    println!("C");
}
