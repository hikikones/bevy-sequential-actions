use bevy::prelude::*;

use bevy_sequential_actions::*;

pub(super) struct TileTrapActionPlugin;

impl Plugin for TileTrapActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(StopState::None)
            .add_system_set(SystemSet::on_update(StopState::Active).with_system(on_stop_update));
    }
}

pub struct TileTrapAction;

impl Action for TileTrapAction {
    fn start(&mut self, actor: Entity, world: &mut World, _commands: &mut ActionCommands) {
        println!("\n---------- Tile Trap Event! ----------");
        println!("Press 'Enter' to continue.\n");

        world.entity_mut(actor).insert(TrapMarker);
        let mut state = world.get_resource_mut::<State<StopState>>().unwrap();
        state.set(StopState::Active).unwrap();
    }

    fn stop(&mut self, actor: Entity, world: &mut World) {
        world.entity_mut(actor).remove::<TrapMarker>();
        let mut state = world.get_resource_mut::<State<StopState>>().unwrap();
        state.set(StopState::None).unwrap();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StopState {
    None,
    Active,
}

#[derive(Component)]
struct TrapMarker;

fn on_stop_update(
    keyboard: Res<Input<KeyCode>>,
    actor_q: Query<Entity, With<TrapMarker>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Return) {
        commands.action(actor_q.single()).next();
    }
}
