use std::marker::PhantomData;

use bevy_app::App;

use crate::*;

#[derive(Deref, DerefMut)]
struct TestApp(App);

impl TestApp {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugin(SequentialActionsPlugin::default());

        Self(app)
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

struct TestAction(Option<Entity>);

impl TestAction {
    const fn new() -> Self {
        Self(None)
    }
}

#[derive(Component)]
struct TestMarker;

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

impl Action for TestAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, world: &mut World) {
        world.entity_mut(self.0.unwrap()).insert(Started);
    }

    fn on_stop(&mut self, _agent: Entity, world: &mut World, reason: StopReason) {
        let mut e = world.entity_mut(self.0.unwrap());
        e.insert(Stopped);

        match reason {
            StopReason::Finished => e.insert(Finished),
            StopReason::Canceled => e.insert(Canceled),
            StopReason::Paused => e.insert(Paused),
        };
    }

    fn on_add(&mut self, _agent: Entity, world: &mut World) {
        self.0 = world.spawn((TestMarker, Added)).id().into();
    }

    fn on_remove(self: Box<Self>, _agent: Entity, world: &mut World) {
        world.entity_mut(self.0.unwrap()).insert(Removed);
    }
}

#[test]
fn add() {
    let mut app = TestApp::new();
    let a: Entity = app.spawn_agent();
    app.actions(a).start(false).add(TestAction::new());

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).len() == 1);

    app.actions(a).start(true).add(TestAction::new());

    assert!(app.current_action(a).is_some());
    assert!(app.action_queue(a).len() == 1);
}

#[test]
fn add_many() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();
    app.actions(a)
        .start(false)
        .add_many(actions![TestAction::new(), TestAction::new()]);

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).len() == 2);

    app.actions(a)
        .start(true)
        .add_many(actions![TestAction::new(), TestAction::new()]);

    assert!(app.current_action(a).is_some());
    assert!(app.action_queue(a).len() == 3);

    app.actions(a).add_many(actions![]);

    assert!(app.current_action(a).is_some());
    assert!(app.action_queue(a).len() == 3);
}

#[test]
fn next() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(TestAction::new());

    let e = app
        .world
        .query_filtered::<Entity, With<TestMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Started>(), true);
    assert_eq!(app.world.entity(e).contains::<Canceled>(), false);

    app.actions(a).next();

    assert_eq!(app.world.entity(e).contains::<Started>(), true);
    assert_eq!(app.world.entity(e).contains::<Canceled>(), true);
}

#[test]
fn finish() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(TestAction::new());
    app.update();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());

    let e = app
        .world
        .query_filtered::<Entity, With<TestMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Finished>(), true);
    assert_eq!(app.world.entity(e).contains::<Canceled>(), false);
    assert_eq!(app.world.entity(e).contains::<Paused>(), false);
}

#[test]
fn cancel() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(TestAction::new()).cancel();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());

    let e = app
        .world
        .query_filtered::<Entity, With<TestMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Finished>(), false);
    assert_eq!(app.world.entity(e).contains::<Canceled>(), true);
    assert_eq!(app.world.entity(e).contains::<Paused>(), false);
}

#[test]
fn pause() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(TestAction::new()).pause();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).len() == 1);

    let e = app
        .world
        .query_filtered::<Entity, With<TestMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Finished>(), false);
    assert_eq!(app.world.entity(e).contains::<Canceled>(), false);
    assert_eq!(app.world.entity(e).contains::<Paused>(), true);
}

#[test]
fn skip() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).start(false).add(TestAction::new()).skip();

    assert!(app.action_queue(a).is_empty());

    let e = app
        .world
        .query_filtered::<Entity, With<TestMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Added>(), true);
    assert_eq!(app.world.entity(e).contains::<Started>(), false);
    assert_eq!(app.world.entity(e).contains::<Stopped>(), false);
    assert_eq!(app.world.entity(e).contains::<Removed>(), true);
}

#[test]
fn clear() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a)
        .add_many(actions![TestAction::new(), TestAction::new()])
        .clear();

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
            .query_filtered::<Entity, With<Removed>>()
            .iter(&app.world)
            .len(),
        2
    );
}

#[test]
fn lifecycle() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).start(false).add(TestAction::new());

    let e = app
        .world
        .query_filtered::<Entity, With<TestMarker>>()
        .single(&app.world);

    assert_eq!(app.world.entity(e).contains::<Added>(), true);
    assert_eq!(app.world.entity(e).contains::<Started>(), false);
    assert_eq!(app.world.entity(e).contains::<Stopped>(), false);
    assert_eq!(app.world.entity(e).contains::<Removed>(), false);

    app.actions(a).execute();

    assert_eq!(app.world.entity(e).contains::<Added>(), true);
    assert_eq!(app.world.entity(e).contains::<Started>(), true);
    assert_eq!(app.world.entity(e).contains::<Stopped>(), false);
    assert_eq!(app.world.entity(e).contains::<Removed>(), false);

    app.actions(a).pause();

    assert_eq!(app.world.entity(e).contains::<Added>(), true);
    assert_eq!(app.world.entity(e).contains::<Started>(), true);
    assert_eq!(app.world.entity(e).contains::<Stopped>(), true);
    assert_eq!(app.world.entity(e).contains::<Removed>(), false);

    app.actions(a).clear();

    assert_eq!(app.world.entity(e).contains::<Added>(), true);
    assert_eq!(app.world.entity(e).contains::<Started>(), true);
    assert_eq!(app.world.entity(e).contains::<Stopped>(), true);
    assert_eq!(app.world.entity(e).contains::<Removed>(), true);
}

#[test]
fn order() {
    #[derive(Default)]
    struct MarkerAction<M: Default + Component>(PhantomData<M>);
    impl<M: Default + Component> Action for MarkerAction<M> {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            true
        }
        fn on_start(&mut self, agent: Entity, world: &mut World) {
            world.entity_mut(agent).insert(M::default());
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

    let mut app = TestApp::new();
    let a = app.spawn_agent();

    // Back
    app.actions(a).add_many(actions![
        MarkerAction::<A>::default(),
        MarkerAction::<B>::default(),
        MarkerAction::<C>::default(),
    ]);

    assert_eq!(app.world.entity(a).contains::<A>(), true);
    assert_eq!(app.world.entity(a).contains::<B>(), false);
    assert_eq!(app.world.entity(a).contains::<C>(), false);

    app.update();

    assert_eq!(app.world.entity(a).contains::<A>(), false);
    assert_eq!(app.world.entity(a).contains::<B>(), true);
    assert_eq!(app.world.entity(a).contains::<C>(), false);

    app.update();

    assert_eq!(app.world.entity(a).contains::<A>(), false);
    assert_eq!(app.world.entity(a).contains::<B>(), false);
    assert_eq!(app.world.entity(a).contains::<C>(), true);

    // Front
    app.actions(a)
        .clear()
        .start(false)
        .add(TestAction::new())
        .order(AddOrder::Front)
        .add_many(actions![
            MarkerAction::<A>::default(),
            MarkerAction::<B>::default(),
            MarkerAction::<C>::default(),
        ])
        .execute();

    assert_eq!(app.world.entity(a).contains::<A>(), true);
    assert_eq!(app.world.entity(a).contains::<B>(), false);
    assert_eq!(app.world.entity(a).contains::<C>(), false);

    app.update();

    assert_eq!(app.world.entity(a).contains::<A>(), false);
    assert_eq!(app.world.entity(a).contains::<B>(), true);
    assert_eq!(app.world.entity(a).contains::<C>(), false);

    app.update();

    assert_eq!(app.world.entity(a).contains::<A>(), false);
    assert_eq!(app.world.entity(a).contains::<B>(), false);
    assert_eq!(app.world.entity(a).contains::<C>(), true);
}
