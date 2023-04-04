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
        check_actions_schedule.add_systems(SequentialActionsPlugin::get_systems());

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

    fn run_update_only(&mut self) {
        self.update_schedule.run(&mut self.world);
    }

    fn run_check_actions_only(&mut self) {
        self.check_actions_schedule.run(&mut self.world);
    }

    fn spawn_agent(&mut self) -> Entity {
        self.world.spawn(ActionsBundle::new()).id()
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

    fn action_finished(&self, agent: Entity) -> &ActionFinished {
        self.world.get::<ActionFinished>(agent).unwrap()
    }

    fn active_count(&self, agent: Entity) -> u32 {
        self.current_action(agent).as_ref().unwrap().0.len()
    }

    fn finished_count(&self, agent: Entity) -> u32 {
        self.action_finished(agent).total()
    }
}

struct CountdownAction {
    count: u32,
    entity: Option<Entity>,
    current: Option<u32>,
}

impl CountdownAction {
    fn new(count: u32) -> Self {
        Self {
            count,
            entity: None,
            current: None,
        }
    }
}

#[derive(Component)]
struct Countdown(u32);

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

impl Action for CountdownAction {
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

fn countdown(
    mut countdown_q: Query<(&mut Countdown, &Agent)>,
    mut finished_q: Query<&mut ActionFinished>,
) {
    for (mut countdown, agent) in countdown_q.iter_mut() {
        if countdown.0 == 0 {
            finished_q.get_mut(agent.0).unwrap().confirm_and_reset();
            continue;
        }

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

    ecs.actions(e).add_sequence(actions![
        CountdownAction::new(0),
        CountdownAction::new(0),
        CountdownAction::new(0),
    ]);

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 2);
}

#[test]
fn add_many_parallel() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_parallel(actions![
        CountdownAction::new(0),
        CountdownAction::new(0),
        CountdownAction::new(0),
    ]);

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 3);
}

#[test]
fn add_many_linked() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_linked(|builder| {
        builder.add_sequence(actions![
            CountdownAction::new(0),
            CountdownAction::new(0),
            CountdownAction::new(0)
        ]);
    });

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 1);
}

#[test]
fn add_many_empty() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .add_sequence(actions![])
        .add_parallel(actions![])
        .add_linked(|_| {});

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
fn next_parallel() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_parallel(actions![
        CountdownAction::new(0),
        CountdownAction::new(0),
        CountdownAction::new(0),
    ]);

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 3);

    ecs.actions(e).next();

    assert!(ecs.current_action(e).is_none());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 0);
}

#[test]
fn next_linked() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_linked(|builder| {
        builder.add_sequence(actions![
            CountdownAction::new(0),
            CountdownAction::new(0),
            CountdownAction::new(0)
        ]);
    });

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 1);

    ecs.actions(e).next();

    assert!(ecs.current_action(e).is_none());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 0);
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
fn finish_parallel() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_parallel(actions![
        CountdownAction::new(0),
        CountdownAction::new(0),
        CountdownAction::new(0),
    ]);

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 3);

    ecs.run();

    assert!(ecs.current_action(e).is_none());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 0);
}

#[test]
fn finish_linked() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_linked(|builder| {
        builder.add_sequence(actions![
            CountdownAction::new(0),
            CountdownAction::new(0),
            CountdownAction::new(0)
        ]);
    });

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 1);

    ecs.run();

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 1);

    ecs.run();

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 1);

    ecs.run();

    assert!(ecs.current_action(e).is_none());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 0);
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
fn cancel_parallel() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_parallel(actions![
        CountdownAction::new(0),
        CountdownAction::new(0),
        CountdownAction::new(0),
    ]);

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 3);

    ecs.actions(e).cancel();

    assert!(ecs.current_action(e).is_none());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 0);
}

#[test]
fn cancel_linked() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_linked(|builder| {
        builder.add_sequence(actions![
            CountdownAction::new(0),
            CountdownAction::new(0),
            CountdownAction::new(0)
        ]);
    });

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 1);

    ecs.actions(e).cancel();

    assert!(ecs.current_action(e).is_none());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 0);
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
fn pause_parallel() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_parallel(actions![
        CountdownAction::new(0),
        CountdownAction::new(0),
        CountdownAction::new(0),
    ]);

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 3);

    ecs.actions(e).pause();

    assert!(ecs.current_action(e).is_none());
    assert!(ecs.action_queue(e).len() == 1);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 0);
}

#[test]
fn pause_linked() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_linked(|builder| {
        builder.add_sequence(actions![
            CountdownAction::new(0),
            CountdownAction::new(0),
            CountdownAction::new(0)
        ]);
    });

    assert!(ecs.current_action(e).is_some());
    assert!(ecs.action_queue(e).len() == 0);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 1);

    ecs.actions(e).pause();

    assert!(ecs.current_action(e).is_none());
    assert!(ecs.action_queue(e).len() == 1);
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 0);
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

    ecs.actions(e)
        .start(false)
        .repeat(Repeat::Amount(0))
        .add(CountdownAction::new(0))
        .repeat(Repeat::Amount(1))
        .add(CountdownAction::new(0))
        .repeat(Repeat::Forever)
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
fn skip_parallel() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .start(false)
        .add_parallel(actions![CountdownAction::new(0), CountdownAction::new(0)])
        .repeat(Repeat::Amount(1))
        .add_parallel(actions![CountdownAction::new(0), CountdownAction::new(0)])
        .repeat(Repeat::Forever)
        .add_parallel(actions![CountdownAction::new(0), CountdownAction::new(0)]);

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
fn skip_linked() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .start(false)
        .add_linked(|builder| {
            builder.add_sequence(actions![CountdownAction::new(0), CountdownAction::new(0)]);
        })
        .repeat(Repeat::Amount(1))
        .add_linked(|builder| {
            builder.add_sequence(actions![CountdownAction::new(0), CountdownAction::new(0)]);
        })
        .repeat(Repeat::Forever)
        .add_linked(|builder| {
            builder.add_sequence(actions![CountdownAction::new(0), CountdownAction::new(0)]);
        });

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
    let e = ecs.world.spawn_empty().id();
    ecs.actions(e).clear();
}

#[test]
fn repeat_amount() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .repeat(Repeat::Amount(0))
        .add(CountdownAction::new(0))
        .repeat(Repeat::Amount(1))
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
        .repeat(Repeat::Forever)
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

    assert!(!ecs.world.entity(e).contains::<ActionFinished>());
    assert!(!ecs.world.entity(e).contains::<ActionQueue>());
    assert!(!ecs.world.entity(e).contains::<CurrentAction>());
}

#[test]
fn order() {
    #[derive(Default)]
    struct Order<T: Default + Component>(PhantomData<T>);
    impl<T: Default + Component> Action for Order<T> {
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
    ecs.actions(e).add_sequence(actions![
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
        .repeat(Repeat::Amount(0))
        .add_sequence(actions![
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
        .repeat(Repeat::Amount(0))
        .add_sequence(actions![
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

    fn countdown_value(w: &mut World) -> u32 {
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

    assert!(countdown_value(&mut ecs.world) == 0);

    ecs.run();

    assert!(countdown_value(&mut ecs.world) == 99);
}

#[test]
fn reset_count() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add_parallel(actions![
        CountdownAction::new(0),
        CountdownAction::new(1),
        CountdownAction::new(2),
    ]);

    assert!(ecs.action_finished(e).reset_count == 0);

    ecs.run_update_only();

    assert!(ecs.action_finished(e).reset_count == 1);

    ecs.run_check_actions_only();

    assert!(ecs.action_finished(e).reset_count == 0);

    ecs.run_update_only();

    assert!(ecs.action_finished(e).reset_count == 2);

    ecs.run_check_actions_only();

    assert!(ecs.action_finished(e).reset_count == 0);

    ecs.run_update_only();

    assert!(ecs.action_finished(e).reset_count == 3);

    ecs.run_check_actions_only();

    assert!(ecs.action_finished(e).reset_count == 0);
    assert!(ecs.current_action(e).is_none());
}

#[test]
fn finished_count() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add(CountdownAction::new(0));

    ecs.run_update_only();

    assert!(ecs.active_count(e) == 1);
    assert!(ecs.finished_count(e) == 1);

    ecs.run_check_actions_only();

    assert!(ecs.finished_count(e) == 0);
    assert!(ecs.current_action(e).is_none());

    ecs.actions(e).add_parallel(actions![
        CountdownAction::new(0),
        CountdownAction::new(0),
        CountdownAction::new(0),
    ]);

    ecs.run_update_only();

    assert!(ecs.active_count(e) == 3);
    assert!(ecs.finished_count(e) == 3);

    ecs.run_check_actions_only();

    assert!(ecs.finished_count(e) == 0);
    assert!(ecs.current_action(e).is_none());

    ecs.actions(e).add_linked(|builder| {
        builder
            .add(CountdownAction::new(0))
            .add_parallel(actions![CountdownAction::new(0), CountdownAction::new(0)]);
    });

    ecs.run_update_only();

    assert!(ecs.active_count(e) == 1);
    assert!(ecs.finished_count(e) == 1);

    ecs.run_check_actions_only();
    ecs.run_update_only();

    assert!(ecs.active_count(e) == 2);
    assert!(ecs.finished_count(e) == 2);

    ecs.run_check_actions_only();

    assert!(ecs.finished_count(e) == 0);
    assert!(ecs.current_action(e).is_none());
}

#[test]
fn change_detection() {
    let mut ecs = Ecs::new();
    let e1 = ecs.spawn_agent();
    let e2 = ecs.spawn_agent();
    let e3 = ecs.spawn_agent();
    let e4 = ecs.spawn_agent();

    fn changed_count(w: &mut World) -> usize {
        w.query_filtered::<(), Changed<ActionFinished>>()
            .iter(w)
            .count()
    }

    assert!(changed_count(&mut ecs.world) == 4);

    ecs.world.clear_trackers();

    ecs.actions(e1).add(CountdownAction::new(0));
    ecs.actions(e2).add(CountdownAction::new(1));
    ecs.actions(e3).add(CountdownAction::new(2));
    ecs.actions(e4).add(CountdownAction::new(3));

    assert!(changed_count(&mut ecs.world) == 0);

    ecs.run();

    assert!(changed_count(&mut ecs.world) == 4);

    ecs.world.clear_trackers();

    ecs.actions(e3).cancel();
    ecs.actions(e4).pause();

    ecs.run();

    assert!(changed_count(&mut ecs.world) == 1);
}

#[test]
#[should_panic]
fn finished_exceeds_active_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e).add(|agent, world: &mut World| {
        let mut finished = world.get_mut::<ActionFinished>(agent).unwrap();
        finished.confirm_and_persist();
        finished.confirm_and_reset();
    });

    ecs.run();
}
