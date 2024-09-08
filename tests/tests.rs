use std::marker::PhantomData;

use bevy_app::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    prelude::*,
    query::{QueryData, QueryFilter},
};

use bevy_sequential_actions::*;

#[derive(Deref, DerefMut)]
struct TestApp(App);

impl TestApp {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(SequentialActionsPlugin)
            .add_systems(Update, countdown);

        Self(app)
    }

    fn spawn_agent(&mut self) -> Entity {
        self.world_mut().spawn(ActionsBundle::new()).id()
    }

    fn entity(&self, entity: Entity) -> EntityRef<'_> {
        self.world().entity(entity)
    }

    fn entity_mut(&mut self, entity: Entity) -> EntityWorldMut<'_> {
        self.world_mut().entity_mut(entity)
    }

    fn current_action(&self, agent: Entity) -> &CurrentAction {
        self.world().get::<CurrentAction>(agent).unwrap()
    }

    fn action_queue(&self, agent: Entity) -> &ActionQueue {
        self.world().get::<ActionQueue>(agent).unwrap()
    }

    fn query_filtered<D: QueryData, F: QueryFilter>(&mut self) -> QueryState<D, F> {
        self.world_mut().query_filtered()
    }

    fn despawn(&mut self, entity: Entity) -> bool {
        self.world_mut().despawn(entity)
    }

    fn get_entity(&self, entity: Entity) -> Option<EntityRef<'_>> {
        self.world().get_entity(entity)
    }
}

struct TestCountdownAction {
    count: i32,
    entity: Option<Entity>,
    current: Option<i32>,
}

impl TestCountdownAction {
    const fn new(count: i32) -> Self {
        Self {
            count,
            entity: None,
            current: None,
        }
    }
}

impl Action for TestCountdownAction {
    fn is_finished(&self, _agent: Entity, world: &World) -> bool {
        world.get::<Countdown>(self.entity.unwrap()).unwrap().0 <= 0
    }

    fn on_add(&mut self, _agent: Entity, world: &mut World) {
        self.entity = world.spawn((Added, CountdownMarker)).id().into();
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        let count = self.current.take().unwrap_or(self.count);
        world
            .entity_mut(self.entity.unwrap())
            .insert((Started, Countdown(count)));

        self.is_finished(agent, world)
    }

    fn on_stop(&mut self, _agent: Option<Entity>, world: &mut World, reason: StopReason) {
        let mut e = world.entity_mut(self.entity.unwrap());
        let countdown = e.insert(Stopped).take::<Countdown>();

        match reason {
            StopReason::Finished => {
                e.insert(Finished);
            }
            StopReason::Canceled => {
                e.insert(Canceled);
            }
            StopReason::Paused => {
                e.insert(Paused);
                self.current = countdown.unwrap().0.into();
            }
        };
    }

    fn on_remove(&mut self, _agent: Option<Entity>, world: &mut World) {
        world.entity_mut(self.entity.unwrap()).insert(Removed);
    }

    fn on_drop(self: Box<Self>, _agent: Option<Entity>, world: &mut World, reason: DropReason) {
        let mut e = world.entity_mut(self.entity.unwrap());
        e.insert(Dropped);

        match reason {
            DropReason::Done => e.insert(Done),
            DropReason::Skipped => e.insert(Skipped),
            DropReason::Cleared => e.insert(Cleared),
        };
    }
}

#[derive(Component)]
struct Countdown(i32);

fn countdown(mut countdown_q: Query<&mut Countdown>) {
    for mut countdown in &mut countdown_q {
        countdown.0 -= 1;
    }
}

#[derive(Component)]
struct CountdownMarker;

#[derive(Component)]
struct Added;

#[derive(Component)]
struct Started;

#[derive(Component)]
struct Stopped;

#[derive(Component)]
struct Finished;

#[derive(Component)]
struct Canceled;

#[derive(Component)]
struct Paused;

#[derive(Component)]
struct Removed;

#[derive(Component)]
struct Dropped;

#[derive(Component)]
struct Done;

#[derive(Component)]
struct Skipped;

#[derive(Component)]
struct Cleared;

#[test]
fn add() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a).add_action_with_config(
        AddConfig {
            start: false,
            order: AddOrder::Back,
        },
        TestCountdownAction::new(0),
    );

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).len() == 1);

    app.entity_mut(a).clear_actions().add_action_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Back,
        },
        TestCountdownAction::new(0),
    );

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).len() == 0);

    app.entity_mut(a).clear_actions().add_action_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Back,
        },
        TestCountdownAction::new(1),
    );

    assert!(app.current_action(a).is_some());
    assert!(app.action_queue(a).len() == 0);
}

#[test]
fn add_many() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a).add_actions_with_config(
        AddConfig {
            start: false,
            order: AddOrder::Back,
        },
        actions![TestCountdownAction::new(0), TestCountdownAction::new(0)],
    );

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).len() == 2);

    app.entity_mut(a).clear_actions().add_actions_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Back,
        },
        actions![TestCountdownAction::new(0), TestCountdownAction::new(0)],
    );

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).len() == 0);

    app.entity_mut(a).add_actions_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Back,
        },
        actions![TestCountdownAction::new(1), TestCountdownAction::new(1)],
    );

    assert!(app.current_action(a).is_some());
    assert!(app.action_queue(a).len() == 1);

    app.entity_mut(a).add_actions_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Back,
        },
        actions![],
    );

    assert!(app.current_action(a).is_some());
    assert!(app.action_queue(a).len() == 1);
}

#[test]
fn next() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a).add_action_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Back,
        },
        TestCountdownAction::new(1),
    );

    let e = app
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(app.world());

    assert_eq!(app.entity(e).contains::<Started>(), true);
    assert_eq!(app.entity(e).contains::<Canceled>(), false);

    app.entity_mut(a).next_action();

    assert_eq!(app.entity(e).contains::<Started>(), true);
    assert_eq!(app.entity(e).contains::<Canceled>(), true);
}

#[test]
fn finish() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a).add_action_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Back,
        },
        TestCountdownAction::new(1),
    );
    app.update();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());

    let e = app
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(app.world());

    assert_eq!(app.entity(e).contains::<Finished>(), true);
    assert_eq!(app.entity(e).contains::<Canceled>(), false);
    assert_eq!(app.entity(e).contains::<Paused>(), false);
    assert_eq!(app.entity(e).contains::<Dropped>(), true);
    assert_eq!(app.entity(e).contains::<Done>(), true);
    assert_eq!(app.entity(e).contains::<Skipped>(), false);
    assert_eq!(app.entity(e).contains::<Cleared>(), false);
}

#[test]
fn cancel() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a)
        .add_action_with_config(
            AddConfig {
                start: true,
                order: AddOrder::Back,
            },
            TestCountdownAction::new(1),
        )
        .cancel_action();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());

    let e = app
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(app.world());

    assert_eq!(app.entity(e).contains::<Finished>(), false);
    assert_eq!(app.entity(e).contains::<Canceled>(), true);
    assert_eq!(app.entity(e).contains::<Paused>(), false);
    assert_eq!(app.entity(e).contains::<Dropped>(), true);
    assert_eq!(app.entity(e).contains::<Done>(), true);
    assert_eq!(app.entity(e).contains::<Skipped>(), false);
    assert_eq!(app.entity(e).contains::<Cleared>(), false);
}

#[test]
fn pause() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a)
        .add_action_with_config(
            AddConfig {
                start: true,
                order: AddOrder::Back,
            },
            TestCountdownAction::new(1),
        )
        .pause_action();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).len() == 1);

    let e = app
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(app.world());

    assert_eq!(app.entity(e).contains::<Finished>(), false);
    assert_eq!(app.entity(e).contains::<Canceled>(), false);
    assert_eq!(app.entity(e).contains::<Paused>(), true);
    assert_eq!(app.entity(e).contains::<Dropped>(), false);
    assert_eq!(app.entity(e).contains::<Done>(), false);
    assert_eq!(app.entity(e).contains::<Skipped>(), false);
    assert_eq!(app.entity(e).contains::<Cleared>(), false);
}

#[test]
fn skip() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a)
        .add_action_with_config(
            AddConfig {
                start: false,
                order: AddOrder::Back,
            },
            TestCountdownAction::new(0),
        )
        .skip_next_action();

    assert!(app.action_queue(a).is_empty());

    let e = app
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(app.world());

    assert_eq!(app.entity(e).contains::<Added>(), true);
    assert_eq!(app.entity(e).contains::<Started>(), false);
    assert_eq!(app.entity(e).contains::<Stopped>(), false);
    assert_eq!(app.entity(e).contains::<Removed>(), true);
    assert_eq!(app.entity(e).contains::<Dropped>(), true);
    assert_eq!(app.entity(e).contains::<Done>(), false);
    assert_eq!(app.entity(e).contains::<Skipped>(), true);
    assert_eq!(app.entity(e).contains::<Cleared>(), false);
}

#[test]
fn clear() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a)
        .add_actions_with_config(
            AddConfig {
                start: true,
                order: AddOrder::Back,
            },
            actions![TestCountdownAction::new(1), TestCountdownAction::new(1)],
        )
        .clear_actions();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());
    assert_eq!(
        app.query_filtered::<Entity, With<Canceled>>()
            .iter(app.world())
            .len(),
        1
    );
    assert_eq!(
        app.query_filtered::<Entity, With<Removed>>()
            .iter(app.world())
            .len(),
        2
    );
    assert_eq!(
        app.query_filtered::<Entity, With<Dropped>>()
            .iter(app.world())
            .len(),
        2
    );
    assert_eq!(
        app.query_filtered::<Entity, With<Cleared>>()
            .iter(app.world())
            .len(),
        2
    );
}

#[test]
fn lifecycle() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a).add_action_with_config(
        AddConfig {
            start: false,
            order: AddOrder::Back,
        },
        TestCountdownAction::new(1),
    );

    let e = app
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(app.world());

    assert_eq!(app.entity(e).contains::<Added>(), true);
    assert_eq!(app.entity(e).contains::<Started>(), false);
    assert_eq!(app.entity(e).contains::<Stopped>(), false);
    assert_eq!(app.entity(e).contains::<Removed>(), false);
    assert_eq!(app.entity(e).contains::<Dropped>(), false);

    app.entity_mut(a).execute_actions();

    assert_eq!(app.entity(e).contains::<Added>(), true);
    assert_eq!(app.entity(e).contains::<Started>(), true);
    assert_eq!(app.entity(e).contains::<Stopped>(), false);
    assert_eq!(app.entity(e).contains::<Removed>(), false);
    assert_eq!(app.entity(e).contains::<Dropped>(), false);

    app.entity_mut(a).pause_action();

    assert_eq!(app.entity(e).contains::<Added>(), true);
    assert_eq!(app.entity(e).contains::<Started>(), true);
    assert_eq!(app.entity(e).contains::<Stopped>(), true);
    assert_eq!(app.entity(e).contains::<Removed>(), false);
    assert_eq!(app.entity(e).contains::<Dropped>(), false);

    app.entity_mut(a).clear_actions();

    assert_eq!(app.entity(e).contains::<Added>(), true);
    assert_eq!(app.entity(e).contains::<Started>(), true);
    assert_eq!(app.entity(e).contains::<Stopped>(), true);
    assert_eq!(app.entity(e).contains::<Removed>(), true);
    assert_eq!(app.entity(e).contains::<Dropped>(), true);
}

#[test]
fn order() {
    struct MarkerAction<M: Default + Component>(PhantomData<M>);
    impl<M: Default + Component> MarkerAction<M> {
        const fn new() -> Self {
            Self(PhantomData)
        }
    }
    impl<M: Default + Component> Action for MarkerAction<M> {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            true
        }
        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            world.entity_mut(agent).insert(M::default());
            false
        }
        fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, _reason: StopReason) {
            world.entity_mut(agent.unwrap()).remove::<M>();
        }
    }

    #[derive(Default, Component)]
    struct A;
    #[derive(Default, Component)]
    struct B;
    #[derive(Default, Component)]
    struct C;

    let mut app = TestApp::new();
    let a = app.spawn_agent();

    // Back
    app.entity_mut(a).add_actions_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Back,
        },
        actions![
            MarkerAction::<A>::new(),
            MarkerAction::<B>::new(),
            MarkerAction::<C>::new(),
        ],
    );

    assert_eq!(app.entity(a).contains::<A>(), true);
    assert_eq!(app.entity(a).contains::<B>(), false);
    assert_eq!(app.entity(a).contains::<C>(), false);

    app.update();

    assert_eq!(app.entity(a).contains::<A>(), false);
    assert_eq!(app.entity(a).contains::<B>(), true);
    assert_eq!(app.entity(a).contains::<C>(), false);

    app.update();

    assert_eq!(app.entity(a).contains::<A>(), false);
    assert_eq!(app.entity(a).contains::<B>(), false);
    assert_eq!(app.entity(a).contains::<C>(), true);

    // Front
    app.entity_mut(a)
        .clear_actions()
        .add_action_with_config(
            AddConfig {
                start: false,
                order: AddOrder::Back,
            },
            TestCountdownAction::new(0),
        )
        .add_actions_with_config(
            AddConfig {
                start: false,
                order: AddOrder::Front,
            },
            actions![
                MarkerAction::<A>::new(),
                MarkerAction::<B>::new(),
                MarkerAction::<C>::new(),
            ],
        )
        .execute_actions();

    assert_eq!(app.entity(a).contains::<A>(), true);
    assert_eq!(app.entity(a).contains::<B>(), false);
    assert_eq!(app.entity(a).contains::<C>(), false);

    app.update();

    assert_eq!(app.entity(a).contains::<A>(), false);
    assert_eq!(app.entity(a).contains::<B>(), true);
    assert_eq!(app.entity(a).contains::<C>(), false);

    app.update();

    assert_eq!(app.entity(a).contains::<A>(), false);
    assert_eq!(app.entity(a).contains::<B>(), false);
    assert_eq!(app.entity(a).contains::<C>(), true);
}

#[test]
fn pause_resume() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    fn countdown_value(app: &mut TestApp) -> i32 {
        app.world_mut().query::<&Countdown>().single(app.world()).0
    }

    app.entity_mut(a).add_action_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Back,
        },
        TestCountdownAction::new(10),
    );

    assert_eq!(countdown_value(&mut app), 10);

    app.update();

    assert_eq!(countdown_value(&mut app), 9);

    app.entity_mut(a).pause_action().add_action_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Front,
        },
        TestCountdownAction::new(1),
    );

    assert_eq!(countdown_value(&mut app), 1);

    app.update();

    assert_eq!(countdown_value(&mut app), 9);
}

#[test]
fn despawn_running() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a).add_actions_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Back,
        },
        actions![
            TestCountdownAction::new(10),
            TestCountdownAction::new(1),
            TestCountdownAction::new(1),
        ],
    );

    app.update();

    assert!(app.get_entity(a).is_some());
    assert_eq!(
        app.query_filtered::<Entity, With<Added>>()
            .iter(app.world())
            .len(),
        3
    );
    assert_eq!(
        app.query_filtered::<Entity, With<Started>>()
            .iter(app.world())
            .len(),
        1
    );
    assert_eq!(
        app.query_filtered::<Entity, With<Stopped>>()
            .iter(app.world())
            .len(),
        0
    );
    assert_eq!(
        app.query_filtered::<Entity, With<Removed>>()
            .iter(app.world())
            .len(),
        0
    );
    assert_eq!(
        app.query_filtered::<Entity, With<Dropped>>()
            .iter(app.world())
            .len(),
        0
    );

    app.despawn(a);

    assert!(app.get_entity(a).is_none());
    assert_eq!(
        app.query_filtered::<Entity, With<Added>>()
            .iter(app.world())
            .len(),
        3
    );
    assert_eq!(
        app.query_filtered::<Entity, With<Started>>()
            .iter(app.world())
            .len(),
        1
    );
    assert_eq!(
        app.query_filtered::<Entity, With<Stopped>>()
            .iter(app.world())
            .len(),
        1
    );
    assert_eq!(
        app.query_filtered::<Entity, With<Removed>>()
            .iter(app.world())
            .len(),
        3
    );
    assert_eq!(
        app.query_filtered::<Entity, With<Dropped>>()
            .iter(app.world())
            .len(),
        3
    );
}

// #[test]
// fn despawn_active_action() {
//     struct DespawnActiveAction;
//     impl Action for DespawnActiveAction {
//         fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
//             false
//         }

//         fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
//             assert!(world.get_entity(agent).is_some());
//             world.despawn(agent);
//             false
//         }

//         fn on_stop(&mut self, agent: Option<Entity>, _world: &mut World, reason: StopReason) {
//             assert!(agent.is_none());
//             assert_eq!(reason, StopReason::Finished);
//         }

//         fn on_drop(self: Box<Self>, agent: Option<Entity>, _world: &mut World, reason: DropReason) {
//             assert!(agent.is_none());
//             assert_eq!(reason, DropReason::Done);
//         }
//     }

//     let mut app = TestApp::new();
//     let a = app.spawn_agent();

//     app.entity_mut(a).add_actions_with_config(
//         AddConfig {
//             start: true,
//             order: AddOrder::Back,
//         },
//         actions![
//             DespawnActiveAction,
//             TestCountdownAction::new(1),
//             TestCountdownAction::new(1),
//         ],
//     );

//     app.update();

//     assert!(app.get_entity(a).is_none());
// }

#[test]
fn despawn_queue_action() {
    struct DespawnQueueAction;
    impl Action for DespawnQueueAction {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            true
        }

        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            assert!(world.get_entity(agent).is_some());
            world.despawn(agent);
            true
        }

        fn on_stop(&mut self, agent: Option<Entity>, _world: &mut World, reason: StopReason) {
            assert!(agent.is_none());
            assert_eq!(reason, StopReason::Finished);
        }

        fn on_drop(self: Box<Self>, agent: Option<Entity>, _world: &mut World, reason: DropReason) {
            assert!(agent.is_none());
            assert_eq!(reason, DropReason::Done);
        }
    }

    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a).add_actions_with_config(
        AddConfig {
            start: true,
            order: AddOrder::Back,
        },
        actions![
            TestCountdownAction::new(1),
            DespawnQueueAction,
            TestCountdownAction::new(0),
        ],
    );

    app.update();

    assert!(app.get_entity(a).is_none());
}

#[test]
fn reasons() {
    struct FinishedAction;
    impl Action for FinishedAction {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            true
        }

        fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
            true
        }

        fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, reason: StopReason) {
            assert_eq!(reason, StopReason::Finished);
        }

        fn on_drop(
            self: Box<Self>,
            _agent: Option<Entity>,
            _world: &mut World,
            reason: DropReason,
        ) {
            assert_eq!(reason, DropReason::Done);
        }
    }

    struct CanceledAction;
    impl Action for CanceledAction {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            false
        }

        fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
            false
        }

        fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, reason: StopReason) {
            assert_eq!(reason, StopReason::Canceled);
        }

        fn on_drop(
            self: Box<Self>,
            _agent: Option<Entity>,
            _world: &mut World,
            reason: DropReason,
        ) {
            assert_eq!(reason, DropReason::Done);
        }
    }

    struct PausedAction;
    impl Action for PausedAction {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            false
        }

        fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
            false
        }

        fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, reason: StopReason) {
            assert_eq!(reason, StopReason::Paused);
        }

        fn on_drop(
            self: Box<Self>,
            _agent: Option<Entity>,
            _world: &mut World,
            reason: DropReason,
        ) {
            assert_eq!(reason, DropReason::Skipped);
        }
    }

    struct ClearedAction;
    impl Action for ClearedAction {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            false
        }

        fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
            false
        }

        fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, reason: StopReason) {
            assert_eq!(reason, StopReason::Canceled);
        }

        fn on_drop(
            self: Box<Self>,
            _agent: Option<Entity>,
            _world: &mut World,
            reason: DropReason,
        ) {
            assert_eq!(reason, DropReason::Cleared);
        }
    }

    struct ActiveAction;
    impl Action for ActiveAction {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            false
        }

        fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
            false
        }

        fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, reason: StopReason) {
            assert_eq!(reason, StopReason::Canceled);
        }

        fn on_drop(
            self: Box<Self>,
            _agent: Option<Entity>,
            _world: &mut World,
            reason: DropReason,
        ) {
            assert_eq!(reason, DropReason::Done);
        }
    }

    struct InQueueAction;
    impl Action for InQueueAction {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            false
        }

        fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
            false
        }

        fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {
            panic!("this should not be called")
        }

        fn on_drop(
            self: Box<Self>,
            _agent: Option<Entity>,
            _world: &mut World,
            reason: DropReason,
        ) {
            assert_eq!(reason, DropReason::Cleared);
        }
    }

    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.entity_mut(a)
        .add_actions(actions![
            FinishedAction,
            CanceledAction,
            PausedAction,
            ClearedAction
        ])
        .cancel_action()
        .execute_actions()
        .pause_action()
        .skip_next_action()
        .clear_actions()
        .add_actions(actions![ActiveAction, InQueueAction, InQueueAction]);

    app.despawn(a);
}

#[test]
fn despawn_true_false() {
    struct DespawnAction<const B: bool>;

    impl<const B: bool> Action for DespawnAction<B> {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            true
        }

        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            assert!(world.get_entity(agent).is_some());
            world.despawn(agent);
            B
        }

        fn on_stop(&mut self, agent: Option<Entity>, _world: &mut World, reason: StopReason) {
            assert!(agent.is_none());
            match B {
                true => assert_eq!(reason, StopReason::Finished),
                false => assert_eq!(reason, StopReason::Canceled),
            }
        }

        fn on_drop(self: Box<Self>, agent: Option<Entity>, _world: &mut World, reason: DropReason) {
            assert!(agent.is_none());
            assert_eq!(reason, DropReason::Done);
        }
    }

    let mut app = TestApp::new();
    let a = app.spawn_agent();
    let b = app.spawn_agent();

    // app.entity_mut(a).add_action(DespawnAction::<true>);
    // app.entity_mut(b).add_action(DespawnAction::<false>);
    app.world_mut().actions(a).add(DespawnAction::<true>);
    app.world_mut().actions(b).add(DespawnAction::<false>);

    app.update();

    assert!(app.get_entity(a).is_none());
    assert!(app.get_entity(b).is_none());
}
