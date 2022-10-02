use std::marker::PhantomData;

use crate::*;

const UPDATE_STAGE: &str = "update_test";

struct Ecs {
    world: World,
    schedule: Schedule,
}

impl Ecs {
    fn new() -> Self {
        let world = World::new();
        let mut schedule = Schedule::default();

        schedule.add_stage(UPDATE_STAGE, SystemStage::single_threaded());
        schedule.add_system_set_to_stage(
            UPDATE_STAGE,
            SystemSet::new()
                .with_system(countdown)
                .with_system(check_countdown.after(countdown)),
        );

        schedule.add_stage_after(
            UPDATE_STAGE,
            CHECK_ACTIONS_STAGE,
            SystemStage::single_threaded(),
        );
        schedule.add_system_set_to_stage(
            CHECK_ACTIONS_STAGE,
            SystemSet::new()
                .with_system(count_finished_actions)
                .with_system(check_finished_actions.after(count_finished_actions)),
        );

        Self { world, schedule }
    }

    fn run(&mut self) {
        self.schedule.run(&mut self.world);
    }

    fn spawn_agent(&mut self) -> Entity {
        self.world
            .spawn()
            .insert_bundle(ActionsBundle::default())
            .id()
    }

    fn actions(&mut self, agent: Entity) -> AgentWorldActions {
        self.world.actions(agent)
    }

    fn current_action(&self, agent: Entity) -> &CurrentAction {
        self.world.get::<CurrentAction>(agent).unwrap()
    }

    fn action_queue(&self, agent: Entity) -> &ActionQueue {
        self.world.get::<ActionQueue>(agent).unwrap()
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
struct CountdownMarker;

#[derive(Component)]
struct Finished;

#[derive(Component)]
struct Canceled;

#[derive(Component)]
struct Paused;

impl Action for CountdownAction {
    fn on_start(&mut self, state: &mut WorldState, _commands: &mut ActionCommands) {
        state.world.entity_mut(state.agent).insert(CountdownMarker);
        state
            .world
            .entity_mut(state.executant)
            .insert(Countdown(self.current.take().unwrap_or(self.count)));
    }

    fn on_stop(&mut self, state: &mut WorldState, reason: StopReason) {
        let mut agent = state.world.entity_mut(state.agent);
        agent.remove::<CountdownMarker>();

        match reason {
            StopReason::Finished => {
                agent.insert(Finished);
            }
            StopReason::Canceled => {
                agent.insert(Canceled);
            }
            StopReason::Paused => {
                agent.insert(Paused);
                self.current = Some(state.world.get::<Countdown>(state.executant).unwrap().0);
            }
        }
    }
}

fn countdown(mut countdown_q: Query<&mut Countdown>) {
    for mut countdown in countdown_q.iter_mut() {
        countdown.0 = countdown.0.saturating_sub(1);
    }
}

fn check_countdown(mut countdown_q: Query<(&Countdown, &mut ActionFinished)>) {
    for (countdown, mut finished) in countdown_q.iter_mut() {
        if countdown.0 == 0 {
            finished.0 = true;
        }
    }
}

#[test]
fn add() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.action_queue(e).len() == 1);

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.action_queue(e).len() == 2);
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
    let e = ecs.spawn_agent();

    ecs.actions(e).add_many(
        ExecutionMode::Sequential,
        [
            CountdownAction::new(0).into_boxed(),
            CountdownAction::new(0).into_boxed(),
            CountdownAction::new(0).into_boxed(),
        ]
        .into_iter(),
    );

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 2);
}

#[test]
fn add_many_parallel() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_many(
        ExecutionMode::Parallel,
        [
            CountdownAction::new(0).into_boxed(),
            CountdownAction::new(0).into_boxed(),
            CountdownAction::new(0).into_boxed(),
        ]
        .into_iter(),
    );

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
}

#[test]
fn next() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<CountdownMarker>());
    assert!(!ecs.world.entity(e).contains::<Canceled>());

    ecs.actions(e).next();

    assert!(!ecs.world.entity(e).contains::<CountdownMarker>());
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
    let e = ecs.spawn_agent();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<CountdownMarker>());
    assert!(!ecs.world.entity(e).contains::<Finished>());

    ecs.run();

    assert!(!ecs.world.entity(e).contains::<CountdownMarker>());
    assert!(ecs.world.entity(e).contains::<Finished>());
}

#[test]
fn cancel() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<CountdownMarker>());
    assert!(!ecs.world.entity(e).contains::<Canceled>());

    ecs.actions(e).cancel();

    assert!(!ecs.world.entity(e).contains::<CountdownMarker>());
    assert!(ecs.world.entity(e).contains::<Canceled>());
}

#[test]
fn pause() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<CountdownMarker>());
    assert!(!ecs.world.entity(e).contains::<Paused>());

    ecs.actions(e).pause();

    assert!(!ecs.world.entity(e).contains::<CountdownMarker>());
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
    let e = ecs.spawn_agent();

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

    assert!(ecs.action_queue(e).len() == 3);

    ecs.actions(e).skip();

    assert!(ecs.action_queue(e).len() == 2);

    ecs.actions(e).skip();

    assert!(ecs.action_queue(e).len() == 2);

    ecs.actions(e).skip();

    assert!(ecs.action_queue(e).len() == 2);

    ecs.actions(e).skip();

    assert!(ecs.action_queue(e).len() == 1);

    ecs.actions(e).skip();

    assert!(ecs.action_queue(e).len() == 1);

    ecs.actions(e).skip();

    assert!(ecs.action_queue(e).len() == 1);
}

#[test]
fn clear() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .add(CountdownAction::new(0))
        .add(CountdownAction::new(0))
        .add(CountdownAction::new(0))
        .clear();

    assert!(ecs.current_action(e).is_none());
    assert!(ecs.action_queue(e).len() == 0);
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
    let e = ecs.spawn_agent();

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

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 1);

    ecs.run();

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);

    ecs.run();

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);

    ecs.run();

    assert!(ecs.current_action(e).is_none());
}

#[test]
fn repeat_forever() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .config(AddConfig {
            order: AddOrder::Back,
            start: true,
            repeat: Repeat::Forever,
        })
        .add(CountdownAction::new(0));

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);

    ecs.run();

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);

    ecs.run();

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
}

#[test]
fn despawn() {
    struct DespawnAction;
    impl Action for DespawnAction {
        fn on_start(&mut self, state: &mut WorldState, _commands: &mut ActionCommands) {
            state.world.despawn(state.agent);
        }
        fn on_stop(&mut self, _state: &mut WorldState, _reason: StopReason) {}
    }

    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

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
        fn on_start(&mut self, state: &mut WorldState, _commands: &mut ActionCommands) {
            state.world.entity_mut(state.agent).insert(T::default());
        }
        fn on_stop(&mut self, state: &mut WorldState, _reason: StopReason) {
            state.world.entity_mut(state.agent).remove::<T>();
        }
    }

    #[derive(Default, Component)]
    struct A;
    #[derive(Default, Component)]
    struct B;
    #[derive(Default, Component)]
    struct C;

    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

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
    let e = ecs.spawn_agent();

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
