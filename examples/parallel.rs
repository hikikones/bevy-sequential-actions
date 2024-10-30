use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins((ScheduleRunnerPlugin::default(), SequentialActionsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, countdown)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(SequentialActions).id();
    commands.actions(agent).add((
        ParallelActions::new(actions![
            PrintAction("hello"),
            CountdownAction::new(2),
            PrintAction("world"),
            CountdownAction::new(4),
        ]),
        |_agent, world: &mut World| {
            world.send_event(AppExit::Success);
            false
        },
    ));
}

struct ParallelActions<const N: usize> {
    actions: [BoxedAction; N],
}

impl<const N: usize> ParallelActions<N> {
    const fn new(actions: [BoxedAction; N]) -> Self {
        Self { actions }
    }
}

impl<const N: usize> Action for ParallelActions<N> {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.actions
            .iter()
            .all(|action| action.is_finished(agent, world))
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        self.actions
            .iter_mut()
            .for_each(|action| action.on_add(agent, world));
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        std::array::from_fn::<bool, N, _>(|i| self.actions[i].on_start(agent, world))
            .into_iter()
            .all(|b| b)
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        self.actions
            .iter_mut()
            .for_each(|action| action.on_stop(agent, world, reason));
    }

    fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
        self.actions
            .iter_mut()
            .for_each(|action| action.on_remove(agent, world));
    }
}

struct PrintAction(&'static str);

impl Action for PrintAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
        println!("{}", self.0);
        true
    }

    fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {}
}

struct CountdownAction {
    count: u32,
    entity: Entity,
}

impl CountdownAction {
    const fn new(count: u32) -> Self {
        Self {
            count,
            entity: Entity::PLACEHOLDER,
        }
    }
}

impl Action for CountdownAction {
    fn is_finished(&self, _agent: Entity, world: &World) -> bool {
        world.get::<Countdown>(self.entity).unwrap().0 == 0
    }

    fn on_add(&mut self, _agent: Entity, world: &mut World) {
        self.entity = world.spawn_empty().id();
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        let mut entity = world.entity_mut(self.entity);

        if entity.contains::<Paused>() {
            entity.remove::<Paused>();
        } else {
            entity.insert(Countdown(self.count));
            println!("Countdown({}): {}", self.entity, self.count);
        }

        self.is_finished(agent, world)
    }

    fn on_stop(&mut self, _agent: Option<Entity>, world: &mut World, reason: StopReason) {
        if reason == StopReason::Paused {
            world.entity_mut(self.entity).insert(Paused);
        }
    }

    fn on_remove(&mut self, _agent: Option<Entity>, world: &mut World) {
        world.despawn(self.entity);
    }
}

#[derive(Component)]
struct Countdown(u32);

#[derive(Component)]
struct Paused;

fn countdown(mut countdown_q: Query<(Entity, &mut Countdown), Without<Paused>>) {
    for (entity, mut countdown) in &mut countdown_q {
        countdown.0 = countdown.0.saturating_sub(1);
        println!("Countdown({}): {}", entity, countdown.0);
    }
}
