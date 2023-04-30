use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugin(ScheduleRunnerPlugin)
        .add_plugin(SequentialActionsPlugin::default())
        .add_startup_system(setup)
        .add_system(countdown)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(ActionsBundle::default()).id();
    commands.actions(agent).add_many(actions![
        ParallelActions {
            actions: actions![
                PrintAction("hello"),
                CountdownAction::new(5),
                PrintAction("world"),
                CountdownAction::new(10),
            ]
        },
        |_, world: &mut World| -> bool {
            world.send_event(AppExit);
            false
        }
    ]);
}

struct ParallelActions<const N: usize> {
    actions: [BoxedAction; N],
}

impl<const N: usize> Action for ParallelActions<N> {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.actions
            .iter()
            .all(|action| action.is_finished(agent, world))
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        for action in self.actions.iter_mut() {
            action.on_add(agent, world);
        }
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        let mut finished = [false; N];
        for (i, action) in self.actions.iter_mut().enumerate() {
            finished[i] = action.on_start(agent, world);
        }
        finished.into_iter().all(|b| b)
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        for action in self.actions.iter_mut() {
            action.on_stop(agent, world, reason);
        }
    }

    fn on_remove(&mut self, agent: Entity, world: &mut World) {
        for action in self.actions.iter_mut() {
            action.on_remove(agent, world);
        }
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

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

struct CountdownAction {
    count: i32,
    entity: Entity,
}

impl CountdownAction {
    const fn new(count: i32) -> Self {
        Self {
            count,
            entity: Entity::PLACEHOLDER,
        }
    }
}

impl Action for CountdownAction {
    fn is_finished(&self, _agent: Entity, world: &World) -> bool {
        world.get::<Countdown>(self.entity).unwrap().0 <= 0
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
            println!("Countdown({:?}): {}", self.entity, self.count);
        }

        self.is_finished(agent, world)
    }

    fn on_stop(&mut self, _agent: Entity, world: &mut World, reason: StopReason) {
        if let StopReason::Paused = reason {
            world.entity_mut(self.entity).insert(Paused);
        }
    }

    fn on_remove(&mut self, _agent: Entity, world: &mut World) {
        world.despawn(self.entity);
    }
}

#[derive(Component)]
struct Countdown(i32);

#[derive(Component)]
struct Paused;

fn countdown(mut countdown_q: Query<(Entity, &mut Countdown), Without<Paused>>) {
    for (entity, mut countdown) in &mut countdown_q {
        countdown.0 -= 1;
        println!("Countdown({:?}): {}", entity, countdown.0);
    }
}
