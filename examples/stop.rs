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
    let entity = commands.spawn_bundle(ActionsBundle::default()).id();

    // Add count and quit action with default config
    commands
        .action(entity)
        .push(CountAction::default())
        .push(QuitAction)
        .submit();
}

#[derive(Default)]
struct CountAction {
    current_count: Option<usize>,
}

impl Action for CountAction {
    fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let count = self.current_count.unwrap_or(0);
        world.entity_mut(entity).insert(Count(count));
    }

    fn remove(&mut self, entity: Entity, world: &mut World) {
        world.entity_mut(entity).remove::<Count>();
    }

    fn stop(&mut self, entity: Entity, world: &mut World) {
        // When stop is called, we need to store the current count progress.
        // This is so we can continue the count when start() is called again.
        let count = world.get::<Count>(entity).unwrap();
        self.current_count = Some(count.0);
        self.remove(entity, world);
    }
}

#[derive(Component)]
struct Count(usize);

fn count(mut count_q: Query<(Entity, &mut Count)>, mut commands: Commands) {
    for (entity, mut count) in count_q.iter_mut() {
        count.0 += 1;

        println!("Count: {}", count.0);

        if count.0 == 10 {
            // Stop current action and add InterruptAction to the front.
            commands
                .action(entity)
                .stop()
                .config(AddConfig {
                    order: AddOrder::Front,
                    start: true,
                    repeat: false,
                })
                .add(InterruptAction);
        } else if count.0 == 20 {
            // Count has finished. Issue next action.
            commands.action(entity).next();
        }
    }
}

struct QuitAction;

impl Action for QuitAction {
    fn start(&mut self, _entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let mut app_exit_ev = world.resource_mut::<Events<AppExit>>();
        app_exit_ev.send(AppExit);
    }

    fn remove(&mut self, _entity: Entity, _world: &mut World) {}
    fn stop(&mut self, _entity: Entity, _world: &mut World) {}
}

struct InterruptAction;

impl Action for InterruptAction {
    fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        println!("\n---------- Interrupt event! ----------");
        println!("Just wait a few seconds and actions will continue again.\n");

        world.entity_mut(entity).insert(InterruptMarker);
        let mut state = world.resource_mut::<State<InterruptState>>();
        state.set(InterruptState::Active).unwrap();
    }

    fn remove(&mut self, entity: Entity, world: &mut World) {
        world.entity_mut(entity).remove::<InterruptMarker>();
        let mut state = world.resource_mut::<State<InterruptState>>();
        state.set(InterruptState::None).unwrap();
    }

    fn stop(&mut self, entity: Entity, world: &mut World) {
        self.remove(entity, world);
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
    entity_q: Query<Entity, With<InterruptMarker>>,
    mut commands: Commands,
) {
    *timer += time.delta_seconds();

    if *timer >= 3.0 {
        commands.action(entity_q.single()).next();
    }
}
