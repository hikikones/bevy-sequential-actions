use std::marker::PhantomData;

use bevy_app::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

#[derive(Deref, DerefMut)]
struct TestApp(App);

impl TestApp {
    fn default() -> Self {
        let mut app = App::new();
        app.add_plugins(SequentialActionsPlugin)
            .add_systems(Update, countdown);

        Self(app)
    }

    fn spawn_agent(&mut self) -> Entity {
        self.world.spawn(ActionsBundle::new()).id()
    }

    // fn actions(&mut self, agent: Entity) -> AgentActions {
    //     self.world.actions(agent)
    // }

    fn actions(&mut self, agent: Entity, f: impl FnOnce(SequentialActionsBuilder)) {
        let mut actions = self.world.resource_mut::<SequentialActions>();
        let builder = actions.entity(agent);
        f(builder);
    }

    fn current_action(&self, agent: Entity) -> &CurrentAction {
        self.world.get::<CurrentAction>(agent).unwrap()
    }

    fn action_queue(&self, agent: Entity) -> &ActionQueue {
        self.world.get::<ActionQueue>(agent).unwrap()
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

    fn on_stop(&mut self, _agent: Entity, world: &mut World, reason: StopReason) {
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

    fn on_remove(&mut self, _agent: Entity, world: &mut World) {
        world.entity_mut(self.entity.unwrap()).insert(Removed);
    }

    fn on_drop(self: Box<Self>, _agent: Entity, world: &mut World, reason: DropReason) {
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
    let mut app = TestApp::default();
    let agent = app.spawn_agent();

    app.actions(agent, |mut builder| {
        builder.start(false).add(TestCountdownAction::new(0));
    });
    // .start(false)
    // .add(TestCountdownAction::new(0));

    app.update();

    assert!(app.current_action(agent).is_none());
    assert!(app.action_queue(agent).len() == 1);

    // app.actions(agent)
    //     .clear()
    //     .start(true)
    //     .add(TestCountdownAction::new(0));

    app.actions(agent, |mut builder| {
        builder
            .clear()
            // .start(true)
            .add(TestCountdownAction::new(10));
    });

    app.update();

    assert!(app.current_action(agent).is_some());
    assert!(app.action_queue(agent).len() == 0);

    // app.actions(agent)
    //     .start(true)
    //     .add(TestCountdownAction::new(1));

    app.actions(agent, |mut builder| {
        builder.add(TestCountdownAction::new(10));
    });

    app.update();

    assert!(app.current_action(agent).is_some());
    assert!(app.action_queue(agent).len() == 1);
}

#[test]
fn add_many() {
    let mut app = TestApp::default();
    let agent = app.spawn_agent();

    // app.actions(agent).start(false).add_many(actions![
    //     TestCountdownAction::new(0),
    //     TestCountdownAction::new(0)
    // ]);

    app.actions(agent, |mut builder| {
        builder.start(false).add_many(actions![
            TestCountdownAction::new(0),
            TestCountdownAction::new(0),
        ]);
    });

    app.update();

    assert!(app.current_action(agent).is_none());
    assert!(app.action_queue(agent).len() == 2);

    // app.actions(agent).clear().start(true).add_many(actions![
    //     TestCountdownAction::new(0),
    //     TestCountdownAction::new(0)
    // ]);

    app.actions(agent, |mut builder| {
        builder.clear().start(true).add_many(actions![
            TestCountdownAction::new(10),
            TestCountdownAction::new(10),
        ]);
    });

    app.update();

    assert!(app.current_action(agent).is_some());
    assert!(app.action_queue(agent).len() == 1);

    // app.actions(agent).start(true).add_many(actions![
    //     TestCountdownAction::new(1),
    //     TestCountdownAction::new(1)
    // ]);

    app.world
        .resource_mut::<SequentialActions>()
        .entity(agent)
        // .start(true)
        .add_many(actions![
            TestCountdownAction::new(10),
            TestCountdownAction::new(10),
        ]);

    // app.actions(agent, |mut builder| {
    //     builder.clear().start(true).add_many(actions![
    //         TestCountdownAction::new(10),
    //         TestCountdownAction::new(10),
    //     ]);
    // });

    app.update();

    assert!(app.current_action(agent).is_some());
    assert!(app.action_queue(agent).len() == 3);

    // app.actions(agent).add_many(actions![]);

    app.world
        .resource_mut::<SequentialActions>()
        .entity(agent)
        .add_many(actions![]);

    assert!(app.current_action(agent).is_some());
    assert!(app.action_queue(agent).len() == 3);
}

#[test]
fn next() {
    let mut app = TestApp::default();
    let agent = app.spawn_agent();

    // app.actions(a).add(TestCountdownAction::new(1));
    app.world
        .resource_mut::<SequentialActions>()
        .entity(agent)
        .add(TestCountdownAction::new(10));

    app.update();

    let e = app
        .world
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Started>(), true);
    assert_eq!(app.world.entity(e).contains::<Canceled>(), false);

    // app.actions(agent).next();
    app.world
        .resource_mut::<SequentialActions>()
        .entity(agent)
        .next();

    app.update();

    assert_eq!(app.world.entity(e).contains::<Started>(), true);
    assert_eq!(app.world.entity(e).contains::<Canceled>(), true);
}

#[test]
fn finish() {
    let mut app = TestApp::default();
    let a = app.spawn_agent();

    // app.actions(a).add(TestCountdownAction::new(1));
    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .add(TestCountdownAction::new(0));

    app.update();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());

    let e = app
        .world
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Finished>(), true);
    assert_eq!(app.world.entity(e).contains::<Canceled>(), false);
    assert_eq!(app.world.entity(e).contains::<Paused>(), false);
    assert_eq!(app.world.entity(e).contains::<Dropped>(), true);
    assert_eq!(app.world.entity(e).contains::<Done>(), true);
    assert_eq!(app.world.entity(e).contains::<Skipped>(), false);
    assert_eq!(app.world.entity(e).contains::<Cleared>(), false);
}

#[test]
fn cancel() {
    let mut app = TestApp::default();
    let a = app.spawn_agent();

    // app.actions(a).add(TestCountdownAction::new(1)).cancel();
    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .add(TestCountdownAction::new(1))
        .cancel();

    app.update();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());

    let e = app
        .world
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Finished>(), false);
    assert_eq!(app.world.entity(e).contains::<Canceled>(), true);
    assert_eq!(app.world.entity(e).contains::<Paused>(), false);
    assert_eq!(app.world.entity(e).contains::<Dropped>(), true);
    assert_eq!(app.world.entity(e).contains::<Done>(), true);
    assert_eq!(app.world.entity(e).contains::<Skipped>(), false);
    assert_eq!(app.world.entity(e).contains::<Cleared>(), false);
}

#[test]
fn pause() {
    let mut app = TestApp::default();
    let a = app.spawn_agent();

    // app.actions(a).add(TestCountdownAction::new(1)).pause();
    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .add(TestCountdownAction::new(1))
        .pause();

    app.update();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).len() == 1);

    let e = app
        .world
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Finished>(), false);
    assert_eq!(app.world.entity(e).contains::<Canceled>(), false);
    assert_eq!(app.world.entity(e).contains::<Paused>(), true);
    assert_eq!(app.world.entity(e).contains::<Dropped>(), false);
    assert_eq!(app.world.entity(e).contains::<Done>(), false);
    assert_eq!(app.world.entity(e).contains::<Skipped>(), false);
    assert_eq!(app.world.entity(e).contains::<Cleared>(), false);
}

#[test]
fn skip() {
    let mut app = TestApp::default();
    let a = app.spawn_agent();

    // app.actions(a)
    //     .start(false)
    //     .add(TestCountdownAction::new(0))
    //     .skip();

    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .start(false)
        .add(TestCountdownAction::new(0))
        .skip();

    app.update();

    assert!(app.action_queue(a).is_empty());

    let e = app
        .world
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Added>(), true);
    assert_eq!(app.world.entity(e).contains::<Started>(), false);
    assert_eq!(app.world.entity(e).contains::<Stopped>(), false);
    assert_eq!(app.world.entity(e).contains::<Removed>(), true);
    assert_eq!(app.world.entity(e).contains::<Dropped>(), true);
    assert_eq!(app.world.entity(e).contains::<Done>(), false);
    assert_eq!(app.world.entity(e).contains::<Skipped>(), true);
    assert_eq!(app.world.entity(e).contains::<Cleared>(), false);
}

#[test]
fn clear() {
    let mut app = TestApp::default();
    let a = app.spawn_agent();

    // app.actions(a)
    //     .add_many(actions![
    //         TestCountdownAction::new(1),
    //         TestCountdownAction::new(1)
    //     ])
    //     .clear();
    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .add_many(actions![
            TestCountdownAction::new(1),
            TestCountdownAction::new(1),
        ])
        .clear();

    app.update();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());
    assert_eq!(
        app.world
            .query_filtered::<Entity, With<Canceled>>()
            .iter(&app.world)
            .len(),
        1
    );
    assert_eq!(
        app.world
            .query_filtered::<Entity, With<Added>>()
            .iter(&app.world)
            .len(),
        2
    );
    assert_eq!(
        app.world
            .query_filtered::<Entity, With<Removed>>()
            .iter(&app.world)
            .len(),
        2
    );
    assert_eq!(
        app.world
            .query_filtered::<Entity, With<Dropped>>()
            .iter(&app.world)
            .len(),
        2
    );
    assert_eq!(
        app.world
            .query_filtered::<Entity, With<Cleared>>()
            .iter(&app.world)
            .len(),
        2
    );
}

#[test]
fn lifecycle() {
    let mut app = TestApp::default();
    let a = app.spawn_agent();

    // app.actions(a).start(false).add(TestCountdownAction::new(1));
    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .start(false)
        .add(TestCountdownAction::new(10));

    app.update();

    let e = app
        .world
        .query_filtered::<Entity, With<CountdownMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Added>(), true);
    assert_eq!(app.world.entity(e).contains::<Started>(), false);
    assert_eq!(app.world.entity(e).contains::<Stopped>(), false);
    assert_eq!(app.world.entity(e).contains::<Removed>(), false);
    assert_eq!(app.world.entity(e).contains::<Dropped>(), false);

    // app.actions(a).execute();
    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .execute();

    app.update();

    assert_eq!(app.world.entity(e).contains::<Added>(), true);
    assert_eq!(app.world.entity(e).contains::<Started>(), true);
    assert_eq!(app.world.entity(e).contains::<Stopped>(), false);
    assert_eq!(app.world.entity(e).contains::<Removed>(), false);
    assert_eq!(app.world.entity(e).contains::<Dropped>(), false);

    // app.actions(a).pause();
    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .pause();

    app.update();

    assert_eq!(app.world.entity(e).contains::<Added>(), true);
    assert_eq!(app.world.entity(e).contains::<Started>(), true);
    assert_eq!(app.world.entity(e).contains::<Stopped>(), true);
    assert_eq!(app.world.entity(e).contains::<Removed>(), false);
    assert_eq!(app.world.entity(e).contains::<Dropped>(), false);

    // app.actions(a).clear();
    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .clear();

    app.update();

    assert_eq!(app.world.entity(e).contains::<Added>(), true);
    assert_eq!(app.world.entity(e).contains::<Started>(), true);
    assert_eq!(app.world.entity(e).contains::<Stopped>(), true);
    assert_eq!(app.world.entity(e).contains::<Removed>(), true);
    assert_eq!(app.world.entity(e).contains::<Dropped>(), true);
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
        fn on_stop(&mut self, agent: Entity, world: &mut World, _reason: StopReason) {
            world.entity_mut(agent).remove::<M>();
        }
    }

    #[derive(Default, Component)]
    struct A;
    #[derive(Default, Component)]
    struct B;
    #[derive(Default, Component)]
    struct C;

    let mut app = TestApp::default();
    let agent = app.spawn_agent();

    // Back
    // app.actions(a).add_many(actions![
    //     MarkerAction::<A>::new(),
    //     MarkerAction::<B>::new(),
    //     MarkerAction::<C>::new(),
    // ]);
    app.actions(agent, |mut builder| {
        builder.add_many(actions![
            MarkerAction::<A>::new(),
            MarkerAction::<B>::new(),
            MarkerAction::<C>::new(),
        ]);
    });

    app.update();

    assert_eq!(app.world.entity(agent).contains::<A>(), true);
    assert_eq!(app.world.entity(agent).contains::<B>(), false);
    assert_eq!(app.world.entity(agent).contains::<C>(), false);

    app.update();

    assert_eq!(app.world.entity(agent).contains::<A>(), false);
    assert_eq!(app.world.entity(agent).contains::<B>(), true);
    assert_eq!(app.world.entity(agent).contains::<C>(), false);

    app.update();

    assert_eq!(app.world.entity(agent).contains::<A>(), false);
    assert_eq!(app.world.entity(agent).contains::<B>(), false);
    assert_eq!(app.world.entity(agent).contains::<C>(), true);

    // Front
    // app.actions(a)
    //     .clear()
    //     .start(false)
    //     .add(TestCountdownAction::new(0))
    //     .order(AddOrder::Front)
    //     .add_many(actions![
    //         MarkerAction::<A>::new(),
    //         MarkerAction::<B>::new(),
    //         MarkerAction::<C>::new(),
    //     ])
    //     .execute();

    app.actions(agent, |mut builder| {
        builder
            .clear()
            .order(AddOrder::Front)
            .add_many(actions![
                MarkerAction::<A>::new(),
                MarkerAction::<B>::new(),
                MarkerAction::<C>::new(),
            ])
            .execute();
    });

    app.update();

    assert_eq!(app.world.entity(agent).contains::<A>(), true);
    assert_eq!(app.world.entity(agent).contains::<B>(), false);
    assert_eq!(app.world.entity(agent).contains::<C>(), false);

    app.update();

    assert_eq!(app.world.entity(agent).contains::<A>(), false);
    assert_eq!(app.world.entity(agent).contains::<B>(), true);
    assert_eq!(app.world.entity(agent).contains::<C>(), false);

    app.update();

    assert_eq!(app.world.entity(agent).contains::<A>(), false);
    assert_eq!(app.world.entity(agent).contains::<B>(), false);
    assert_eq!(app.world.entity(agent).contains::<C>(), true);
}

#[test]
fn pause_resume() {
    let mut app = TestApp::default();
    let a = app.spawn_agent();

    fn countdown_value(app: &mut TestApp) -> i32 {
        app.world.query::<&Countdown>().single(&app.world).0
    }

    // app.actions(a).add(TestCountdownAction::new(10));
    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .add(TestCountdownAction::new(10));

    app.update();

    assert_eq!(countdown_value(&mut app), 10);

    app.update();

    assert_eq!(countdown_value(&mut app), 9);

    // app.actions(a)
    //     .pause()
    //     .order(AddOrder::Front)
    //     .add(TestCountdownAction::new(1));
    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .pause()
        .order(AddOrder::Front)
        .add(TestCountdownAction::new(1));

    app.update();

    assert_eq!(countdown_value(&mut app), 1);

    app.update();

    assert_eq!(countdown_value(&mut app), 8);
}

#[test]
fn despawn() {
    struct DespawnAction;
    impl Action for DespawnAction {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            false
        }

        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            world.despawn(agent);
            false
        }

        fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
    }

    let mut app = TestApp::default();
    let a = app.spawn_agent();

    // app.actions(a).add_many(actions![
    //     TestCountdownAction::new(1),
    //     DespawnAction,
    //     TestCountdownAction::new(0),
    // ]);

    app.world
        .resource_mut::<SequentialActions>()
        .entity(a)
        .add_many(actions![
            TestCountdownAction::new(0),
            DespawnAction,
            TestCountdownAction::new(0),
        ]);

    app.update();

    assert!(app.world.get_entity(a).is_none());
}
