use std::marker::PhantomData;

use crate::*;

struct Ecs {
    world: World,
    update_schedule: Schedule,
    check_actions_schedule: Schedule,
}

impl Ecs {
    fn new() -> Self {
        let mut world = World::new();
        world.init_resource::<DeferredActions>();

        let mut update_schedule = Schedule::default();
        update_schedule.add_system(countdown);

        let mut check_actions_schedule = Schedule::default();
        check_actions_schedule
            .add_systems(SequentialActionsPlugin::<DefaultAgentMarker>::get_systems());

        Self {
            world,
            update_schedule,
            check_actions_schedule,
        }
    }

    fn run(&mut self) {
        self.update_schedule.run(&mut self.world);
        self.check_actions_schedule.run(&mut self.world);
    }

    fn _run_update_only(&mut self) {
        self.update_schedule.run(&mut self.world);
    }

    fn _run_check_actions_only(&mut self) {
        self.check_actions_schedule.run(&mut self.world);
    }

    fn spawn_agent(&mut self) -> Entity {
        self.world.spawn(ActionsBundle::default()).id()
    }

    fn actions(&mut self, agent: Entity) -> AgentActions {
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
    count: i32,
    entity: Option<Entity>,
    current: Option<i32>,
}

impl CountdownAction {
    fn new(count: i32) -> Self {
        Self {
            count,
            entity: None,
            current: None,
        }
    }
}

impl Action for CountdownAction {
    fn is_finished(&self, _agent: Entity, world: &World) -> bool {
        world.get::<Countdown>(self.entity.unwrap()).unwrap().0 <= 0
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) {
        self.entity = Some(
            world
                .spawn((
                    Countdown(self.current.take().unwrap_or(self.count)),
                    Agent(agent),
                ))
                .id(),
        );
        world.entity_mut(agent).insert(Active);
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        world.entity_mut(agent).remove::<Active>();

        let entity = self.entity.take().unwrap();

        match reason {
            StopReason::Finished => {
                world.entity_mut(agent).insert(Finished);
            }
            StopReason::Canceled => {
                world.entity_mut(agent).insert(Canceled);
            }
            StopReason::Paused => {
                world.entity_mut(agent).insert(Paused);
                self.current = Some(world.get::<Countdown>(entity).unwrap().0);
            }
        }

        world.despawn(entity);
    }
}

#[derive(Component)]
struct Countdown(i32);

#[derive(Component)]
struct Agent(Entity);

#[derive(Component)]
struct Active;

#[derive(Component)]
struct Finished;

#[derive(Component)]
struct Canceled;

#[derive(Component)]
struct Paused;

fn countdown(mut countdown_q: Query<&mut Countdown>) {
    for mut countdown in countdown_q.iter_mut() {
        countdown.0 -= 1;
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
    let e = ecs.world.spawn_empty().id();
    ecs.actions(e).add(CountdownAction::new(0));
}

#[test]
fn add_many_sequential() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_many(actions![
        CountdownAction::new(0),
        CountdownAction::new(0),
        CountdownAction::new(0),
    ]);

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 2);
}

#[test]
fn add_many_empty() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_many(actions![]);

    assert!(ecs.current_action(e).is_none());
}

#[test]
fn next() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Active>());
    assert!(!ecs.world.entity(e).contains::<Canceled>());

    ecs.actions(e).next();

    assert!(!ecs.world.entity(e).contains::<Active>());
    assert!(ecs.world.entity(e).contains::<Canceled>());
}

#[test]
#[should_panic]
fn next_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.world.spawn_empty().id();
    ecs.actions(e).next();
}

#[test]
fn finish() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Active>());
    assert!(!ecs.world.entity(e).contains::<Finished>());

    ecs.run();

    assert!(!ecs.world.entity(e).contains::<Active>());
    assert!(ecs.world.entity(e).contains::<Finished>());
}

#[test]
fn cancel() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Active>());
    assert!(!ecs.world.entity(e).contains::<Canceled>());

    ecs.actions(e).cancel();

    assert!(!ecs.world.entity(e).contains::<Active>());
    assert!(ecs.world.entity(e).contains::<Canceled>());
}

#[test]
fn cancel_none() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .add(CountdownAction::new(0))
        .add(CountdownAction::new(0));

    assert!(ecs.current_action(e).is_some());

    ecs.actions(e).cancel();

    assert!(ecs.current_action(e).is_none());
}

#[test]
fn pause() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Active>());
    assert!(!ecs.world.entity(e).contains::<Paused>());

    ecs.actions(e).pause();

    assert!(!ecs.world.entity(e).contains::<Active>());
    assert!(ecs.world.entity(e).contains::<Paused>());
}

#[test]
fn pause_none() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .add(CountdownAction::new(0))
        .add(CountdownAction::new(0));

    assert!(ecs.current_action(e).is_some());

    ecs.actions(e).pause();

    assert!(ecs.current_action(e).is_none());
}

#[test]
#[should_panic]
fn pause_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.world.spawn_empty().id();
    ecs.actions(e).pause();
}

#[test]
fn skip() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).start(false).add_many(actions![
        CountdownAction::new(0),
        CountdownAction::new(0),
        CountdownAction::new(0),
    ]);

    assert!(ecs.action_queue(e).len() == 3);

    ecs.actions(e).skip();

    assert!(ecs.action_queue(e).len() == 2);

    ecs.actions(e).skip();

    assert!(ecs.action_queue(e).len() == 1);

    ecs.actions(e).skip();

    assert!(ecs.action_queue(e).len() == 0);
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
    let e = ecs.world.spawn_empty().id();
    ecs.actions(e).clear();
}

#[test]
fn despawn() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .add(CountdownAction::new(0))
        .add(|agent, world: &mut World| {
            world.deferred_actions(agent).custom(move |w: &mut World| {
                w.despawn(agent);
            });
        })
        .add(CountdownAction::new(0));

    ecs.run();
    ecs.run();

    assert!(ecs.world.get_entity(e).is_none());
}

#[test]
fn remove_bundle() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .add(CountdownAction::new(0))
        .add(|agent, world: &mut World| {
            world.deferred_actions(agent).custom(move |w: &mut World| {
                w.entity_mut(agent).remove::<ActionsBundle>();
            });
        })
        .add(CountdownAction::new(0));

    ecs.run();
    ecs.run();

    assert!(!ecs.world.entity(e).contains::<ActionQueue>());
    assert!(!ecs.world.entity(e).contains::<CurrentAction>());
}

#[test]
fn order() {
    #[derive(Default)]
    struct Order<T: Default + Component>(PhantomData<T>);
    impl<T: Default + Component> Action for Order<T> {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            true
        }
        fn on_start(&mut self, agent: Entity, world: &mut World) {
            world.entity_mut(agent).insert(T::default());
        }
        fn on_stop(&mut self, agent: Entity, world: &mut World, _reason: StopReason) {
            world.entity_mut(agent).remove::<T>();
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
    ecs.actions(e).add_many(actions![
        Order::<A>::default(),
        Order::<B>::default(),
        Order::<C>::default(),
    ]);

    assert!(ecs.world.entity(e).contains::<A>());

    ecs.actions(e).next();

    assert!(ecs.world.entity(e).contains::<B>());

    ecs.actions(e).next();

    assert!(ecs.world.entity(e).contains::<C>());

    // A, B, C
    ecs.actions(e)
        .clear()
        .start(false)
        .order(AddOrder::Front)
        .add_many(actions![
            Order::<A>::default(),
            Order::<B>::default(),
            Order::<C>::default(),
        ])
        .next();

    assert!(ecs.world.entity(e).contains::<A>());

    ecs.actions(e).next();

    assert!(ecs.world.entity(e).contains::<B>());

    ecs.actions(e).next();

    assert!(ecs.world.entity(e).contains::<C>());

    // C, B, A
    ecs.actions(e)
        .clear()
        .start(false)
        .order(AddOrder::Front)
        .add_many(actions![
            Order::<C>::default(),
            Order::<B>::default(),
            Order::<A>::default(),
        ])
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

    fn countdown_value(w: &mut World) -> i32 {
        w.query::<&Countdown>().single(w).0
    }

    ecs.actions(e).add(CountdownAction::new(100));

    ecs.run();

    assert!(countdown_value(&mut ecs.world) == 99);

    ecs.actions(e)
        .pause()
        .order(AddOrder::Front)
        .add(CountdownAction::new(2));

    assert!(countdown_value(&mut ecs.world) == 2);

    ecs.run();

    assert!(countdown_value(&mut ecs.world) == 1);

    ecs.run();

    assert!(countdown_value(&mut ecs.world) == 99);
}
