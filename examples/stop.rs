use std::time::Duration;

use bevy::{
    app::{AppExit, ScheduleRunnerSettings},
    ecs::event::Events,
    prelude::*,
};

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 10.0,
        )))
        .add_plugins(MinimalPlugins)
        .add_state(InterruptState::None)
        .add_startup_system(setup)
        .add_system(count)
        .add_system_set(
            SystemSet::on_update(InterruptState::Active).with_system(on_interrupt_update),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Create entity with ActionsBundle
    let id = commands.spawn_bundle(ActionsBundle::default()).id();

    // Add count and quit action with default config
    commands
        .action_builder(id, AddConfig::default())
        .push(CountAction::default())
        .push(QuitAction)
        .submit();
}

#[derive(Default)]
struct CountAction {
    current_count: Option<usize>,
}

impl Action for CountAction {
    fn add(&mut self, actor: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let count = self.current_count.unwrap_or(0);
        world.entity_mut(actor).insert(Count(count));
    }

    fn remove(&mut self, actor: Entity, world: &mut World) {
        world.entity_mut(actor).remove::<Count>();
    }

    fn stop(&mut self, actor: Entity, world: &mut World) {
        // When stop is called, we need to store the current count progress.
        // This is so we can continue the count when add() is called again.
        let count = world.get::<Count>(actor).unwrap();
        self.current_count = Some(count.0);
        self.remove(actor, world);
    }
}

#[derive(Component)]
struct Count(usize);

fn count(mut count_q: Query<(Entity, &mut Count)>, mut commands: Commands) {
    for (actor, mut count) in count_q.iter_mut() {
        count.0 += 1;

        println!("Count: {}", count.0);

        if count.0 == 10 {
            // Stop current action and add InterruptAction to the front.
            commands.stop_action(actor);
            commands.add_action(
                actor,
                InterruptAction,
                AddConfig {
                    order: AddOrder::Front,
                    start: true,
                    repeat: false,
                },
            );
        } else if count.0 == 20 {
            // Count has finished. Issue next action.
            commands.next_action(actor);
        }
    }
}

struct QuitAction;

impl Action for QuitAction {
    fn add(&mut self, _actor: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let mut app_exit_ev = world.resource_mut::<Events<AppExit>>();
        app_exit_ev.send(AppExit);
    }

    fn remove(&mut self, _actor: Entity, _world: &mut World) {}

    fn stop(&mut self, _actor: Entity, _world: &mut World) {}
}

struct InterruptAction;

impl Action for InterruptAction {
    fn add(&mut self, actor: Entity, world: &mut World, _commands: &mut ActionCommands) {
        println!("\n---------- Interrupt event! ----------");
        println!("Just wait a few seconds and actions will continue again.\n");

        world.entity_mut(actor).insert(InterruptMarker);
        let mut state = world.resource_mut::<State<InterruptState>>();
        state.set(InterruptState::Active).unwrap();
    }

    fn remove(&mut self, actor: Entity, world: &mut World) {
        world.entity_mut(actor).remove::<InterruptMarker>();
        let mut state = world.resource_mut::<State<InterruptState>>();
        state.set(InterruptState::None).unwrap();
    }

    fn stop(&mut self, actor: Entity, world: &mut World) {
        self.remove(actor, world);
    }
}

#[derive(Component)]
struct InterruptMarker;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum InterruptState {
    None,
    Active,
}

fn on_interrupt_update(
    mut timer: Local<f32>,
    time: Res<Time>,
    actor_q: Query<Entity, With<InterruptMarker>>,
    mut commands: Commands,
) {
    *timer += time.delta_seconds();

    if *timer >= 3.0 {
        commands.next_action(actor_q.single());
    }
}
