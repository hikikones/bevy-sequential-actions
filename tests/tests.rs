use std::{marker::PhantomData, ops::Deref};

use bevy_app::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

#[derive(Deref, DerefMut)]
struct TestApp(App);

impl TestApp {
    fn new() -> Self {
        let mut app = App::new();
        app.init_resource::<Lifecycles>()
            .add_plugins(SequentialActionsPlugin)
            .add_systems(Update, countdown);

        Self(app)
    }

    fn spawn_agent(&mut self) -> Entity {
        self.world_mut().spawn(ActionsBundle::new()).id()
    }

    fn actions(&mut self, agent: Entity) -> AgentActions<'_> {
        self.world_mut().actions(agent)
    }

    fn lifecycles(&self) -> &Lifecycles {
        self.world().resource::<Lifecycles>()
    }

    fn lifecycles_mut(&mut self) -> Mut<'_, Lifecycles> {
        self.world_mut().resource_mut::<Lifecycles>()
    }

    fn entity(&self, entity: Entity) -> EntityRef<'_> {
        self.world().entity(entity)
    }

    fn get_entity(&self, entity: Entity) -> Option<EntityRef<'_>> {
        self.world().get_entity(entity)
    }

    fn current_action(&self, agent: Entity) -> &CurrentAction {
        self.world().get::<CurrentAction>(agent).unwrap()
    }

    fn action_queue(&self, agent: Entity) -> &ActionQueue {
        self.world().get::<ActionQueue>(agent).unwrap()
    }

    fn despawn(&mut self, entity: Entity) -> bool {
        self.world_mut().despawn(entity)
    }

    fn reset(&mut self) {
        self.0.world_mut().clear_all();
        self.0.init_resource::<Lifecycles>();
    }
}

struct TestCountdownAction {
    count: i32,
    current: Option<i32>,
}

impl TestCountdownAction {
    const fn new(count: i32) -> Self {
        Self {
            count,
            current: None,
        }
    }
}

impl Action for TestCountdownAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        world.get::<Countdown>(agent).unwrap().0 <= 0
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        world
            .resource_mut::<Lifecycles>()
            .push(Lifecycle::Add(agent));
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        let count = self.current.take().unwrap_or(self.count);
        world.entity_mut(agent).insert(Countdown(count));
        world
            .resource_mut::<Lifecycles>()
            .push(Lifecycle::Start(agent));

        self.is_finished(agent, world)
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        world
            .resource_mut::<Lifecycles>()
            .push(Lifecycle::Stop(agent, reason));

        let Some(agent) = agent else { return };

        let countdown = world.entity_mut(agent).take::<Countdown>();

        if reason == StopReason::Paused {
            self.current = Some(countdown.unwrap().0);
        }
    }

    fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
        world
            .resource_mut::<Lifecycles>()
            .push(Lifecycle::Remove(agent));
    }

    fn on_drop(self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
        world
            .resource_mut::<Lifecycles>()
            .push(Lifecycle::Drop(agent, reason));
    }
}

#[derive(Component)]
struct Countdown(i32);

fn countdown(mut countdown_q: Query<&mut Countdown>) {
    for mut countdown in &mut countdown_q {
        countdown.0 -= 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Add(Entity),
    Start(Entity),
    Stop(Option<Entity>, StopReason),
    Remove(Option<Entity>),
    Drop(Option<Entity>, DropReason),
}

#[derive(Debug, Default, Resource, Deref, DerefMut)]
struct Lifecycles(Vec<Lifecycle>);

#[test]
fn add() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).start(false).add(TestCountdownAction::new(0));

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 1);

    app.actions(a).clear().add(TestCountdownAction::new(0));

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 0);

    app.actions(a).clear().add(TestCountdownAction::new(1));

    assert!(app.current_action(a).is_some());
    assert_eq!(app.action_queue(a).len(), 0);

    app.reset();

    app.actions(a).add(TestCountdownAction::new(1));

    // TODO: Resulting lifecycles should probably be empty vec[]
    assert_eq!(app.lifecycles().deref().clone(), vec![Lifecycle::Add(a)]);
}

#[test]
fn add_many() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).start(false).add_many(actions![
        TestCountdownAction::new(0),
        TestCountdownAction::new(0)
    ]);

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 2);

    app.actions(a).clear().add_many(actions![
        TestCountdownAction::new(0),
        TestCountdownAction::new(0)
    ]);

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 0);

    app.actions(a).add_many(actions![
        TestCountdownAction::new(1),
        TestCountdownAction::new(1)
    ]);

    assert!(app.current_action(a).is_some());
    assert_eq!(app.action_queue(a).len(), 1);

    app.actions(a).add_many(actions![]);

    assert!(app.current_action(a).is_some());
    assert_eq!(app.action_queue(a).len(), 1);
}

#[test]
fn finish() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(TestCountdownAction::new(1));
    app.update();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![
            Lifecycle::Add(a),
            Lifecycle::Start(a),
            Lifecycle::Stop(Some(a), StopReason::Finished),
            Lifecycle::Remove(Some(a)),
            Lifecycle::Drop(Some(a), DropReason::Done)
        ]
    );
}

#[test]
fn cancel() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(TestCountdownAction::new(1)).cancel();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![
            Lifecycle::Add(a),
            Lifecycle::Start(a),
            Lifecycle::Stop(Some(a), StopReason::Canceled),
            Lifecycle::Remove(Some(a)),
            Lifecycle::Drop(Some(a), DropReason::Done)
        ]
    );
}

#[test]
fn pause() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(TestCountdownAction::new(1)).pause();

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 1);
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![
            Lifecycle::Add(a),
            Lifecycle::Start(a),
            Lifecycle::Stop(Some(a), StopReason::Paused)
        ]
    );
}

#[test]
fn next() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(TestCountdownAction::new(1)).next();

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 0);
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![
            Lifecycle::Add(a),
            Lifecycle::Start(a),
            Lifecycle::Stop(Some(a), StopReason::Canceled),
            Lifecycle::Remove(Some(a)),
            Lifecycle::Drop(Some(a), DropReason::Done)
        ]
    );
}

#[test]
fn skip() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a)
        .start(false)
        .add(TestCountdownAction::new(1))
        .skip();

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 0);
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![
            Lifecycle::Add(a),
            Lifecycle::Remove(Some(a)),
            Lifecycle::Drop(Some(a), DropReason::Skipped)
        ]
    );
}

#[test]
fn clear() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a)
        .add_many(actions![
            TestCountdownAction::new(1),
            TestCountdownAction::new(1)
        ])
        .clear();

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 0);
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![
            Lifecycle::Add(a),
            Lifecycle::Add(a),
            Lifecycle::Start(a),
            Lifecycle::Stop(Some(a), StopReason::Canceled),
            Lifecycle::Remove(Some(a)),
            Lifecycle::Drop(Some(a), DropReason::Cleared),
            Lifecycle::Remove(Some(a)),
            Lifecycle::Drop(Some(a), DropReason::Cleared)
        ]
    );
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
    app.actions(a).add_many(actions![
        MarkerAction::<A>::new(),
        MarkerAction::<B>::new(),
        MarkerAction::<C>::new(),
    ]);

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
    app.actions(a)
        .clear()
        .start(false)
        .add(|_a, _w: &mut World| false)
        .config(AddConfig::new(true, AddOrder::Front))
        .add_many(actions![
            MarkerAction::<A>::new(),
            MarkerAction::<B>::new(),
            MarkerAction::<C>::new(),
        ]);

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

    app.actions(a).add(TestCountdownAction::new(10));

    assert_eq!(app.entity(a).get::<Countdown>().unwrap().0, 10);

    app.update();

    assert_eq!(app.entity(a).get::<Countdown>().unwrap().0, 9);

    app.actions(a)
        .pause()
        .order(AddOrder::Front)
        .add(TestCountdownAction::new(1));

    assert_eq!(app.entity(a).get::<Countdown>().unwrap().0, 1);

    app.update();

    assert_eq!(app.entity(a).get::<Countdown>().unwrap().0, 9);
}

#[test]
fn despawn() {
    struct DespawnAction<const B: bool>;
    impl<const B: bool> Action for DespawnAction<B> {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            true
        }
        fn on_add(&mut self, agent: Entity, world: &mut World) {
            world
                .resource_mut::<Lifecycles>()
                .push(Lifecycle::Add(agent));
        }
        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            world
                .resource_mut::<Lifecycles>()
                .push(Lifecycle::Start(agent));
            world.despawn(agent);
            B
        }
        fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
            world
                .resource_mut::<Lifecycles>()
                .push(Lifecycle::Stop(agent, reason));
        }
        fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
            world
                .resource_mut::<Lifecycles>()
                .push(Lifecycle::Remove(agent));
        }
        fn on_drop(self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
            world
                .resource_mut::<Lifecycles>()
                .push(Lifecycle::Drop(agent, reason));
        }
    }

    let mut app = TestApp::new();

    let a = app.spawn_agent();
    app.actions(a).add(DespawnAction::<true>);

    assert!(app.get_entity(a).is_none());
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![
            Lifecycle::Add(a),
            Lifecycle::Start(a),
            Lifecycle::Stop(None, StopReason::Finished),
            Lifecycle::Remove(None),
            Lifecycle::Drop(None, DropReason::Done)
        ]
    );

    app.reset();

    let a = app.spawn_agent();
    app.actions(a).add(DespawnAction::<false>);

    assert!(app.get_entity(a).is_none());
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![
            Lifecycle::Add(a),
            Lifecycle::Start(a),
            Lifecycle::Stop(None, StopReason::Canceled),
            Lifecycle::Remove(None),
            Lifecycle::Drop(None, DropReason::Done)
        ]
    );

    app.reset();

    let a = app.spawn_agent();
    app.actions(a)
        .add_many(actions![DespawnAction::<true>, TestCountdownAction::new(1)]);

    assert!(app.get_entity(a).is_none());
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![
            Lifecycle::Add(a),
            Lifecycle::Add(a),
            Lifecycle::Start(a),
            // After despawn, the on_remove component lifecycle hook
            // is triggered for ActionQueue containing TestCountdownAction
            Lifecycle::Remove(None),
            Lifecycle::Drop(None, DropReason::Cleared),
            // Back to DespawnAction
            Lifecycle::Stop(None, StopReason::Finished),
            Lifecycle::Remove(None),
            Lifecycle::Drop(None, DropReason::Done)
        ]
    );

    app.reset();
    let a = app.spawn_agent();
    app.actions(a).add_many(actions![
        DespawnAction::<false>,
        TestCountdownAction::new(1)
    ]);

    assert!(app.get_entity(a).is_none());
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![
            Lifecycle::Add(a),
            Lifecycle::Add(a),
            Lifecycle::Start(a),
            // After despawn, the on_remove component lifecycle hook
            // is triggered for ActionQueue containing TestCountdownAction
            Lifecycle::Remove(None),
            Lifecycle::Drop(None, DropReason::Cleared),
            // Back to DespawnAction
            Lifecycle::Stop(None, StopReason::Canceled),
            Lifecycle::Remove(None),
            Lifecycle::Drop(None, DropReason::Done)
        ]
    );
}

#[test]
fn despawn_running() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add_many(actions![
        TestCountdownAction::new(10),
        TestCountdownAction::new(1)
    ]);

    app.update();

    assert!(app.get_entity(a).is_some());
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![Lifecycle::Add(a), Lifecycle::Add(a), Lifecycle::Start(a)]
    );

    app.despawn(a);

    assert!(app.get_entity(a).is_none());
    assert_eq!(
        app.lifecycles().deref().clone(),
        vec![
            Lifecycle::Add(a),
            Lifecycle::Add(a),
            Lifecycle::Start(a),
            Lifecycle::Stop(None, StopReason::Canceled),
            Lifecycle::Remove(None),
            Lifecycle::Drop(None, DropReason::Done),
            Lifecycle::Remove(None),
            Lifecycle::Drop(None, DropReason::Cleared)
        ]
    );
}
