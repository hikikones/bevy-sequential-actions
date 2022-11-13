use std::marker::PhantomData;

use crate::*;

struct Ecs {
    world: World,
    update_schedule: Schedule,
    check_actions_schedule: Schedule,
}

impl Ecs {
    fn new() -> Self {
        Self {
            world: World::new(),
            update_schedule: Schedule::default()
                .with_stage("update", SystemStage::single(countdown)),
            check_actions_schedule: Schedule::default().with_stage(
                "check_actions",
                SystemStage::single_threaded()
                    .with_system_set(SequentialActionsPlugin::get_systems()),
            ),
        }
    }

    fn run(&mut self) {
        self.update_schedule.run_once(&mut self.world);
        self.check_actions_schedule.run_once(&mut self.world);
    }

    fn run_update_only(&mut self) {
        self.update_schedule.run_once(&mut self.world);
    }

    fn run_check_actions_only(&mut self) {
        self.check_actions_schedule.run_once(&mut self.world);
    }

    fn spawn_agent(&mut self) -> Entity {
        self.world.spawn(ActionsBundle::new()).id()
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

    fn action_finished(&self, agent: Entity) -> &ActionFinished {
        self.world.get::<ActionFinished>(agent).unwrap()
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
    fn on_start(&mut self, agent: Entity, world: &mut World, _commands: &mut ActionCommands) {
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
    assert!(ecs.world.query::<&Countdown>().iter(&mut ecs.world).count() == 3);
}

#[test]
fn add_many_parallel_empty() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .add_many(ExecutionMode::Parallel, [].into_iter());

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
    let e = ecs.world.spawn_empty().id();
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
    use bevy::prelude::DespawnRecursiveExt;

    let mut ecs = Ecs::new();
    let e = ecs.spawn_agent();

    ecs.actions(e)
        .add(CountdownAction::new(0))
        .add(|agent, _world: &mut World, commands: &mut ActionCommands| {
            commands.add(move |w: &mut World| {
                w.entity_mut(agent).despawn_recursive();
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
        .add(|agent, _world: &mut World, commands: &mut ActionCommands| {
            commands.add(move |w: &mut World| {
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
        fn on_start(&mut self, agent: Entity, world: &mut World, _commands: &mut ActionCommands) {
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
            repeat: Repeat::None,
        })
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

    ecs.actions(e).add_many(
        ExecutionMode::Parallel,
        actions![
            CountdownAction::new(0),
            CountdownAction::new(1),
            CountdownAction::new(2),
        ],
    );

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

// #[test]
// fn change_detection() {
//     let mut ecs = Ecs::new();
//     let e = ecs.spawn_agent();

//     fn changed_count(w: &mut World) -> usize {
//         w.query_filtered::<(), Changed<ActionFinished>>()
//             .iter(w)
//             .count()
//     }

//     assert!(changed_count(&mut ecs.world) == 1);

//     ecs.world.clear_trackers();

//     ecs.actions(e).add_many(
//         ExecutionMode::Parallel,
//         actions![
//             CountdownAction::new(0),
//             CountdownAction::new(1),
//             CountdownAction::new(2),
//         ],
//     );

//     assert!(changed_count(&mut ecs.world) == 0);

//     ecs.run();

//     assert!(changed_count(&mut ecs.world) == 1);

//     ecs.world.clear_trackers();
//     ecs.run();

//     assert!(changed_count(&mut ecs.world) == 2);
// }
