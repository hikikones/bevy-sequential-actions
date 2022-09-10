use std::marker::PhantomData;

use bevy::prelude::*;

use crate::*;

const UPDATE_STAGE: &str = "update";

struct Ecs {
    world: World,
    schedule: Schedule,
}

impl Ecs {
    fn new() -> Self {
        let world = World::new();
        let mut schedule = Schedule::default();

        schedule.add_stage(UPDATE_STAGE, SystemStage::parallel());
        schedule.add_stage_after(UPDATE_STAGE, CHECK_ACTIONS_STAGE, SystemStage::parallel());
        schedule.add_system_to_stage(CHECK_ACTIONS_STAGE, check_actions);

        let mut ecs = Self { world, schedule };
        ecs.add_system(countdown_system);
        ecs
    }

    fn add_system<Param, S: IntoSystem<(), (), Param>>(&mut self, system: S) {
        self.schedule.add_system_to_stage(UPDATE_STAGE, system);
    }

    fn run(&mut self) {
        self.schedule.run(&mut self.world);
    }

    fn spawn_action_entity(&mut self) -> Entity {
        self.world
            .spawn()
            .insert_bundle(ActionsBundle::default())
            .id()
    }

    fn actions(&mut self, entity: Entity) -> EntityWorldActions {
        self.world.actions(entity)
    }

    fn get_current_action(&self, entity: Entity) -> &CurrentAction {
        self.world.get::<CurrentAction>(entity).unwrap()
    }

    fn get_action_queue(&self, entity: Entity) -> &ActionQueue {
        self.world.get::<ActionQueue>(entity).unwrap()
    }
}

struct CountdownAction {
    count: u32,
    current: Option<u32>,
}

impl CountdownAction {
    fn new(count: u32) -> Self {
        Self {
            count,
            current: None,
        }
    }
}

#[derive(Component)]
struct Countdown(u32);

#[derive(Component)]
struct Finished;

#[derive(Component)]
struct Canceled;

#[derive(Component)]
struct Paused;

impl Action for CountdownAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world
            .entity_mut(entity)
            .insert(Countdown(self.current.unwrap_or(self.count)));
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
        let mut e = world.entity_mut(entity);
        let count = e.remove::<Countdown>();

        match reason {
            StopReason::Finished => {
                self.current = None;
                e.insert(Finished);
            }
            StopReason::Canceled => {
                self.current = None;
                e.insert(Canceled);
            }
            StopReason::Paused => {
                self.current = Some(count.unwrap().0);
                e.insert(Paused);
            }
        }
    }
}

fn countdown_system(mut countdown_q: Query<(&mut Countdown, &mut FinishedCount)>) {
    for (mut countdown, mut finished) in countdown_q.iter_mut() {
        countdown.0 = countdown.0.saturating_sub(1);

        if countdown.0 == 0 {
            finished.increment();
        }
    }
}

#[test]
fn add() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.get_action_queue(e).len() == 1);

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.get_action_queue(e).len() == 2);
}

#[test]
#[should_panic]
fn add_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.world.spawn().id();
    ecs.actions(e).add(CountdownAction::new(0));
}

#[test]
fn add_many_sequential() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add_many(
        ExecutionMode::Sequential,
        [
            CountdownAction::new(0).into_boxed(),
            CountdownAction::new(0).into_boxed(),
            CountdownAction::new(0).into_boxed(),
        ]
        .into_iter(),
    );

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 2);
}

#[test]
fn add_many_parallel() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add_many(
        ExecutionMode::Parallel,
        [
            CountdownAction::new(0).into_boxed(),
            CountdownAction::new(0).into_boxed(),
            CountdownAction::new(0).into_boxed(),
        ]
        .into_iter(),
    );

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);
}

#[test]
fn next() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Countdown>());
    assert!(!ecs.world.entity(e).contains::<Canceled>());

    ecs.actions(e).next();

    assert!(!ecs.world.entity(e).contains::<Countdown>());
    assert!(ecs.world.entity(e).contains::<Canceled>());
}

#[test]
#[should_panic]
fn next_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.world.spawn().id();
    ecs.actions(e).next();
}

#[test]
fn finish() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Countdown>());
    assert!(!ecs.world.entity(e).contains::<Finished>());

    ecs.run();

    assert!(!ecs.world.entity(e).contains::<Countdown>());
    assert!(ecs.world.entity(e).contains::<Finished>());
}

#[test]
fn pause() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Countdown>());
    assert!(!ecs.world.entity(e).contains::<Paused>());

    ecs.actions(e).pause();

    assert!(!ecs.world.entity(e).contains::<Countdown>());
    assert!(ecs.world.entity(e).contains::<Paused>());
}

#[test]
#[should_panic]
fn pause_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.world.spawn().id();
    ecs.actions(e).pause();
}

#[test]
fn skip() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .config(AddConfig {
            order: AddOrder::Back,
            start: false,
            repeat: Repeat::Amount(0),
        })
        .add(CountdownAction::new(0))
        .config(AddConfig {
            order: AddOrder::Back,
            start: false,
            repeat: Repeat::Amount(1),
        })
        .add(CountdownAction::new(0))
        .config(AddConfig {
            order: AddOrder::Back,
            start: false,
            repeat: Repeat::Forever,
        })
        .add(CountdownAction::new(0));

    assert!(ecs.get_action_queue(e).len() == 3);

    ecs.actions(e).skip();

    assert!(ecs.get_action_queue(e).len() == 2);

    ecs.actions(e).skip();

    assert!(ecs.get_action_queue(e).len() == 2);

    ecs.actions(e).skip();

    assert!(ecs.get_action_queue(e).len() == 2);

    ecs.actions(e).skip();

    assert!(ecs.get_action_queue(e).len() == 1);

    ecs.actions(e).skip();

    assert!(ecs.get_action_queue(e).len() == 1);

    ecs.actions(e).skip();

    assert!(ecs.get_action_queue(e).len() == 1);
}

#[test]
fn clear() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .add(CountdownAction::new(0))
        .add(CountdownAction::new(0))
        .add(CountdownAction::new(0))
        .clear();

    assert!(ecs.get_current_action(e).is_none());
    assert!(ecs.get_action_queue(e).len() == 0);
    assert!(ecs.world.entity(e).contains::<Canceled>());
}

#[test]
#[should_panic]
fn clear_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.world.spawn().id();
    ecs.actions(e).clear();
}

#[test]
fn repeat_amount() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .config(AddConfig {
            order: AddOrder::Back,
            start: true,
            repeat: Repeat::Amount(0),
        })
        .add(CountdownAction::new(0))
        .config(AddConfig {
            order: AddOrder::Back,
            start: true,
            repeat: Repeat::Amount(1),
        })
        .add(CountdownAction::new(0));

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 1);

    ecs.run();

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.run();

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.run();

    assert!(ecs.get_current_action(e).is_none());
}

#[test]
fn repeat_forever() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .config(AddConfig {
            order: AddOrder::Back,
            start: true,
            repeat: Repeat::Forever,
        })
        .add(CountdownAction::new(0));

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.run();

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.run();

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);
}

#[test]
fn despawn() {
    struct DespawnAction;
    impl Action for DespawnAction {
        fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
            world.despawn(entity);
        }
        fn on_stop(&mut self, _entity: Entity, _world: &mut World, _reason: StopReason) {}
    }

    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .add(CountdownAction::new(0))
        .add(DespawnAction)
        .add(CountdownAction::new(0));

    ecs.run();

    assert!(ecs.world.get_entity(e).is_none());
}

#[test]
fn order() {
    #[derive(Default)]
    struct Order<T: Default + Component>(PhantomData<T>);
    impl<T: Default + Component> Action for Order<T> {
        fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
            world.entity_mut(entity).insert(T::default());
        }
        fn on_stop(&mut self, entity: Entity, world: &mut World, _reason: StopReason) {
            world.entity_mut(entity).remove::<T>();
        }
    }

    #[derive(Default, Component)]
    struct A;
    #[derive(Default, Component)]
    struct B;
    #[derive(Default, Component)]
    struct C;

    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    // A, B, C
    ecs.actions(e).add_many(
        ExecutionMode::Sequential,
        [
            Order::<A>::default().into_boxed(),
            Order::<B>::default().into_boxed(),
            Order::<C>::default().into_boxed(),
        ]
        .into_iter(),
    );

    assert!(ecs.world.entity(e).contains::<A>());

    ecs.actions(e).next();

    assert!(ecs.world.entity(e).contains::<B>());

    ecs.actions(e).next();

    assert!(ecs.world.entity(e).contains::<C>());

    // A, B, C
    ecs.actions(e)
        .clear()
        .config(AddConfig {
            order: AddOrder::Front,
            start: false,
            repeat: Repeat::Amount(0),
        })
        .add_many(
            ExecutionMode::Sequential,
            [
                Order::<A>::default().into_boxed(),
                Order::<B>::default().into_boxed(),
                Order::<C>::default().into_boxed(),
            ]
            .into_iter(),
        )
        .next();

    assert!(ecs.world.entity(e).contains::<A>());

    ecs.actions(e).next();

    assert!(ecs.world.entity(e).contains::<B>());

    ecs.actions(e).next();

    assert!(ecs.world.entity(e).contains::<C>());

    // C, B, A
    ecs.actions(e)
        .clear()
        .config(AddConfig {
            order: AddOrder::Front,
            start: false,
            repeat: Repeat::Amount(0),
        })
        .add_many(
            ExecutionMode::Sequential,
            [
                Order::<C>::default().into_boxed(),
                Order::<B>::default().into_boxed(),
                Order::<A>::default().into_boxed(),
            ]
            .into_iter(),
        )
        .next();

    assert!(ecs.world.entity(e).contains::<C>());

    ecs.actions(e).next();

    assert!(ecs.world.entity(e).contains::<B>());

    ecs.actions(e).next();

    assert!(ecs.world.entity(e).contains::<A>());
}

#[test]
fn pause_resume() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    fn countdown_value(w: &mut World) -> u32 {
        w.query::<&Countdown>().single(w).0
    }

    ecs.actions(e).add(CountdownAction::new(100));

    ecs.run();

    assert!(countdown_value(&mut ecs.world) == 99);

    ecs.actions(e)
        .pause()
        .config(AddConfig {
            order: AddOrder::Front,
            start: true,
            repeat: Repeat::Amount(0),
        })
        .add(CountdownAction::new(2));

    ecs.run();
    ecs.run();
    ecs.run();

    assert!(countdown_value(&mut ecs.world) == 98);
}
