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
        app.init_resource::<Hooks>()
            .add_plugins(SequentialActionsPlugin)
            .add_systems(Update, (countdown, countup));

        Self(app)
    }

    fn spawn_agent(&mut self) -> Entity {
        self.world_mut().spawn(ActionsBundle::new()).id()
    }

    fn actions(&mut self, agent: Entity) -> AgentActions<'_> {
        self.world_mut().actions(agent)
    }

    fn hooks(&self) -> &Hooks {
        self.world().resource::<Hooks>()
    }

    fn hooks_mut(&mut self) -> Mut<'_, Hooks> {
        self.world_mut().resource_mut::<Hooks>()
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

    fn reset(&mut self) -> &mut Self {
        self.world_mut().clear_entities();
        self.world_mut().resource_mut::<Hooks>().clear();
        self
    }
}

#[derive(Debug, Default, Resource, Deref, DerefMut)]
struct Hooks(Vec<Hook>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hook {
    Add(Name, Entity),
    Start(Name, Entity),
    Stop(Name, Option<Entity>, StopReason),
    Remove(Name, Option<Entity>),
    Drop(Name, Option<Entity>, DropReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Name {
    Countdown,
    Countup,
    Despawn,
    GoodAdd,
    BadAdd,
}

impl Name {
    fn on_add(self, agent: Entity, world: &mut World) {
        world.resource_mut::<Hooks>().push(Hook::Add(self, agent));
    }

    fn on_start(self, agent: Entity, world: &mut World) {
        world.resource_mut::<Hooks>().push(Hook::Start(self, agent));
    }

    fn on_stop(self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        world
            .resource_mut::<Hooks>()
            .push(Hook::Stop(self, agent, reason));
    }

    fn on_remove(self, agent: Option<Entity>, world: &mut World) {
        world
            .resource_mut::<Hooks>()
            .push(Hook::Remove(self, agent));
    }

    fn on_drop(self, agent: Option<Entity>, world: &mut World, reason: DropReason) {
        world
            .resource_mut::<Hooks>()
            .push(Hook::Drop(self, agent, reason));
    }
}

struct CountdownAction {
    count: i32,
    current: Option<i32>,
}

impl CountdownAction {
    const fn new(count: i32) -> Self {
        Self {
            count,
            current: None,
        }
    }
}

impl Action for CountdownAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        world.get::<Countdown>(agent).unwrap().0 <= 0
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        Name::Countdown.on_add(agent, world);
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        Name::Countdown.on_start(agent, world);

        let count = self.current.take().unwrap_or(self.count);
        world.entity_mut(agent).insert(Countdown(count));

        self.is_finished(agent, world)
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        Name::Countdown.on_stop(agent, world, reason);

        let Some(agent) = agent else { return };

        let countdown = world.entity_mut(agent).take::<Countdown>();

        if reason == StopReason::Paused {
            self.current = Some(countdown.unwrap().0);
        }
    }

    fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
        Name::Countdown.on_remove(agent, world);
    }

    fn on_drop(self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
        Name::Countdown.on_drop(agent, world, reason);
    }
}

#[derive(Component)]
struct Countdown(i32);

fn countdown(mut countdown_q: Query<&mut Countdown>) {
    for mut countdown in &mut countdown_q {
        countdown.0 -= 1;
    }
}

struct CountupAction {
    count: i32,
    current: Option<i32>,
}

impl CountupAction {
    const fn new(count: i32) -> Self {
        Self {
            count,
            current: None,
        }
    }
}

impl Action for CountupAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        world.get::<Countup>(agent).unwrap().0 >= self.count
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        Name::Countup.on_add(agent, world);
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        Name::Countup.on_start(agent, world);

        let count = self.current.take().unwrap_or_default();
        world.entity_mut(agent).insert(Countup(count));

        self.is_finished(agent, world)
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        Name::Countup.on_stop(agent, world, reason);

        let Some(agent) = agent else { return };

        let countup = world.entity_mut(agent).take::<Countup>();

        if reason == StopReason::Paused {
            self.current = Some(countup.unwrap().0);
        }
    }

    fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
        Name::Countup.on_remove(agent, world);
    }

    fn on_drop(self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
        Name::Countup.on_drop(agent, world, reason);
    }
}

#[derive(Component)]
struct Countup(i32);

fn countup(mut countup_q: Query<&mut Countup>) {
    for mut countup in &mut countup_q {
        countup.0 += 1;
    }
}

#[test]
fn add() {
    let mut app = TestApp::new();

    let a = app.spawn_agent();
    app.actions(a).start(false).add(CountdownAction::new(0));

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 1);
    assert_eq!(
        app.hooks().deref().clone(),
        vec![Hook::Add(Name::Countdown, a)]
    );

    app.actions(a).clear().add(CountdownAction::new(0));

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 0);

    app.actions(a).clear().add(CountdownAction::new(1));

    assert!(app.current_action(a).is_some());
    assert_eq!(app.action_queue(a).len(), 0);

    app.reset().actions(a).add(CountdownAction::new(1));

    assert!(app.get_entity(a).is_none());
    assert_eq!(app.hooks().deref().clone(), vec![]);
}

#[test]
fn add_many() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a)
        .start(false)
        .add_many(actions![CountdownAction::new(0), CountupAction::new(0)]);

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 2);
    assert_eq!(
        app.hooks().deref().clone(),
        vec![Hook::Add(Name::Countdown, a), Hook::Add(Name::Countup, a)]
    );

    app.actions(a)
        .clear()
        .add_many(actions![CountdownAction::new(0), CountupAction::new(0)]);

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 0);

    app.actions(a)
        .add_many(actions![CountdownAction::new(1), CountupAction::new(1)]);

    assert!(app.current_action(a).is_some());
    assert_eq!(app.action_queue(a).len(), 1);

    app.actions(a).add_many(actions![]);

    assert!(app.current_action(a).is_some());
    assert_eq!(app.action_queue(a).len(), 1);

    app.reset()
        .actions(a)
        .add_many(actions![CountdownAction::new(1), CountupAction::new(1)]);

    assert!(app.get_entity(a).is_none());
    assert_eq!(app.hooks().deref().clone(), vec![]);
}

#[test]
fn finish() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(CountdownAction::new(0));

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Countdown, a),
            Hook::Start(Name::Countdown, a),
            Hook::Stop(Name::Countdown, Some(a), StopReason::Finished),
            Hook::Remove(Name::Countdown, Some(a)),
            Hook::Drop(Name::Countdown, Some(a), DropReason::Done)
        ]
    );

    let a = app.reset().spawn_agent();
    app.actions(a).add(CountdownAction::new(1));
    app.update();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Countdown, a),
            Hook::Start(Name::Countdown, a),
            Hook::Stop(Name::Countdown, Some(a), StopReason::Finished),
            Hook::Remove(Name::Countdown, Some(a)),
            Hook::Drop(Name::Countdown, Some(a), DropReason::Done)
        ]
    );

    let a = app.reset().spawn_agent();
    app.actions(a)
        .add_many(actions![CountdownAction::new(0), CountupAction::new(0)]);

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Countdown, a),
            Hook::Add(Name::Countup, a),
            Hook::Start(Name::Countdown, a),
            Hook::Stop(Name::Countdown, Some(a), StopReason::Finished),
            Hook::Remove(Name::Countdown, Some(a)),
            Hook::Drop(Name::Countdown, Some(a), DropReason::Done),
            Hook::Start(Name::Countup, a),
            Hook::Stop(Name::Countup, Some(a), StopReason::Finished),
            Hook::Remove(Name::Countup, Some(a)),
            Hook::Drop(Name::Countup, Some(a), DropReason::Done)
        ]
    );

    let a = app.reset().spawn_agent();
    app.actions(a)
        .add_many(actions![CountdownAction::new(1), CountupAction::new(1)]);
    app.update();

    assert!(app.current_action(a).is_some());
    assert_eq!(app.action_queue(a).len(), 0);
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Countdown, a),
            Hook::Add(Name::Countup, a),
            Hook::Start(Name::Countdown, a),
            Hook::Stop(Name::Countdown, Some(a), StopReason::Finished),
            Hook::Remove(Name::Countdown, Some(a)),
            Hook::Drop(Name::Countdown, Some(a), DropReason::Done),
            Hook::Start(Name::Countup, a)
        ]
    );
}

#[test]
fn cancel() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(CountdownAction::new(1)).cancel();

    assert!(app.current_action(a).is_none());
    assert!(app.action_queue(a).is_empty());
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Countdown, a),
            Hook::Start(Name::Countdown, a),
            Hook::Stop(Name::Countdown, Some(a), StopReason::Canceled),
            Hook::Remove(Name::Countdown, Some(a)),
            Hook::Drop(Name::Countdown, Some(a), DropReason::Done)
        ]
    );

    app.reset().actions(a).cancel();
}

#[test]
fn pause() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(CountdownAction::new(1)).pause();

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 1);
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Countdown, a),
            Hook::Start(Name::Countdown, a),
            Hook::Stop(Name::Countdown, Some(a), StopReason::Paused)
        ]
    );

    app.reset().actions(a).pause();
}

#[test]
fn next() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a).add(CountdownAction::new(1)).next();

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 0);
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Countdown, a),
            Hook::Start(Name::Countdown, a),
            Hook::Stop(Name::Countdown, Some(a), StopReason::Canceled),
            Hook::Remove(Name::Countdown, Some(a)),
            Hook::Drop(Name::Countdown, Some(a), DropReason::Done)
        ]
    );

    app.reset().actions(a).next();
}

#[test]
fn skip() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a)
        .start(false)
        .add(CountdownAction::new(1))
        .skip();

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 0);
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Countdown, a),
            Hook::Remove(Name::Countdown, Some(a)),
            Hook::Drop(Name::Countdown, Some(a), DropReason::Skipped)
        ]
    );

    app.reset().actions(a).skip();
}

#[test]
fn clear() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a)
        .add_many(actions![CountdownAction::new(1), CountupAction::new(1)])
        .clear();

    assert!(app.current_action(a).is_none());
    assert_eq!(app.action_queue(a).len(), 0);
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Countdown, a),
            Hook::Add(Name::Countup, a),
            Hook::Start(Name::Countdown, a),
            Hook::Stop(Name::Countdown, Some(a), StopReason::Canceled),
            Hook::Remove(Name::Countdown, Some(a)),
            Hook::Drop(Name::Countdown, Some(a), DropReason::Cleared),
            Hook::Remove(Name::Countup, Some(a)),
            Hook::Drop(Name::Countup, Some(a), DropReason::Cleared)
        ]
    );

    app.reset().actions(a).clear();
}

#[test]
fn order() {
    #[derive(Default)]
    struct MarkerAction<M: Default + Component>(PhantomData<M>);
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
        MarkerAction::<A>::default(),
        MarkerAction::<B>::default(),
        MarkerAction::<C>::default(),
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
        .order(AddOrder::Front)
        .add_many(actions![
            MarkerAction::<A>::default(),
            MarkerAction::<B>::default(),
            MarkerAction::<C>::default(),
        ])
        .execute();

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

    app.actions(a).add(CountdownAction::new(10));

    assert_eq!(app.entity(a).get::<Countdown>().unwrap().0, 10);

    app.update();

    assert_eq!(app.entity(a).get::<Countdown>().unwrap().0, 9);

    app.actions(a)
        .pause()
        .order(AddOrder::Front)
        .add(CountdownAction::new(1));

    assert_eq!(app.entity(a).get::<Countdown>().unwrap().0, 1);

    app.update();

    assert_eq!(app.entity(a).get::<Countdown>().unwrap().0, 9);
}

#[test]
fn despawn() {
    let mut app = TestApp::new();
    let a = app.spawn_agent();

    app.actions(a)
        .add_many(actions![CountdownAction::new(10), CountupAction::new(10)]);
    app.update();
    app.despawn(a);

    assert!(app.get_entity(a).is_none());
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Countdown, a),
            Hook::Add(Name::Countup, a),
            Hook::Start(Name::Countdown, a),
            Hook::Stop(Name::Countdown, None, StopReason::Canceled),
            Hook::Remove(Name::Countdown, None),
            Hook::Drop(Name::Countdown, None, DropReason::Done),
            Hook::Remove(Name::Countup, None),
            Hook::Drop(Name::Countup, None, DropReason::Cleared)
        ]
    );
}

#[test]
fn despawn_action() {
    struct DespawnAction<const B: bool>;
    impl<const B: bool> Action for DespawnAction<B> {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            true
        }
        fn on_add(&mut self, agent: Entity, world: &mut World) {
            Name::Despawn.on_add(agent, world);
        }
        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            Name::Despawn.on_start(agent, world);
            world.despawn(agent);
            B
        }
        fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
            Name::Despawn.on_stop(agent, world, reason);
        }
        fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
            Name::Despawn.on_remove(agent, world);
        }
        fn on_drop(self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
            Name::Despawn.on_drop(agent, world, reason);
        }
    }

    let mut app = TestApp::new();

    let a = app.spawn_agent();
    app.actions(a).add(DespawnAction::<true>);

    assert!(app.get_entity(a).is_none());
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Despawn, a),
            Hook::Start(Name::Despawn, a),
            Hook::Stop(Name::Despawn, None, StopReason::Finished),
            Hook::Remove(Name::Despawn, None),
            Hook::Drop(Name::Despawn, None, DropReason::Done)
        ]
    );

    let a = app.reset().spawn_agent();
    app.actions(a).add(DespawnAction::<false>);

    assert!(app.get_entity(a).is_none());
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Despawn, a),
            Hook::Start(Name::Despawn, a),
            Hook::Stop(Name::Despawn, None, StopReason::Canceled),
            Hook::Remove(Name::Despawn, None),
            Hook::Drop(Name::Despawn, None, DropReason::Done)
        ]
    );

    let a = app.reset().spawn_agent();
    app.actions(a).add_many(actions![
        DespawnAction::<true>,
        CountdownAction::new(1),
        CountupAction::new(1)
    ]);

    assert!(app.get_entity(a).is_none());
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Despawn, a),
            Hook::Add(Name::Countdown, a),
            Hook::Add(Name::Countup, a),
            Hook::Start(Name::Despawn, a),
            // After despawn, the bevy ecs on_remove component hook is triggered
            Hook::Remove(Name::Countdown, None),
            Hook::Drop(Name::Countdown, None, DropReason::Cleared),
            Hook::Remove(Name::Countup, None),
            Hook::Drop(Name::Countup, None, DropReason::Cleared),
            // Back to DespawnAction
            Hook::Stop(Name::Despawn, None, StopReason::Finished),
            Hook::Remove(Name::Despawn, None),
            Hook::Drop(Name::Despawn, None, DropReason::Done)
        ]
    );

    let a = app.reset().spawn_agent();
    app.actions(a).add_many(actions![
        DespawnAction::<false>,
        CountdownAction::new(1),
        CountupAction::new(1)
    ]);

    assert!(app.get_entity(a).is_none());
    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::Despawn, a),
            Hook::Add(Name::Countdown, a),
            Hook::Add(Name::Countup, a),
            Hook::Start(Name::Despawn, a),
            // After despawn, the bevy ecs on_remove component hook is triggered
            Hook::Remove(Name::Countdown, None),
            Hook::Drop(Name::Countdown, None, DropReason::Cleared),
            Hook::Remove(Name::Countup, None),
            Hook::Drop(Name::Countup, None, DropReason::Cleared),
            // Back to DespawnAction
            Hook::Stop(Name::Despawn, None, StopReason::Canceled),
            Hook::Remove(Name::Despawn, None),
            Hook::Drop(Name::Despawn, None, DropReason::Done)
        ]
    );
}

#[test]
fn good_add_action() {
    struct GoodAddAction;
    impl Action for GoodAddAction {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            true
        }
        fn on_add(&mut self, agent: Entity, world: &mut World) {
            Name::GoodAdd.on_add(agent, world);
        }
        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            Name::GoodAdd.on_start(agent, world);
            world
                .actions(agent)
                .start(false)
                .add(CountdownAction::new(1));
            true
        }
        fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
            Name::GoodAdd.on_stop(agent, world, reason);
        }
        fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
            Name::GoodAdd.on_remove(agent, world);
        }
        fn on_drop(self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
            Name::GoodAdd.on_drop(agent, world, reason);
        }
    }

    let mut app = TestApp::new();

    let a = app.spawn_agent();
    app.actions(a).add(GoodAddAction);

    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::GoodAdd, a),
            Hook::Start(Name::GoodAdd, a),
            Hook::Add(Name::Countdown, a),
            Hook::Stop(Name::GoodAdd, Some(a), StopReason::Finished),
            Hook::Remove(Name::GoodAdd, Some(a)),
            Hook::Drop(Name::GoodAdd, Some(a), DropReason::Done),
            Hook::Start(Name::Countdown, a)
        ]
    );
}

#[test]
fn bad_add_action() {
    struct BadAddAction;
    impl Action for BadAddAction {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            true
        }
        fn on_add(&mut self, agent: Entity, world: &mut World) {
            Name::BadAdd.on_add(agent, world);
        }
        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            Name::BadAdd.on_start(agent, world);
            true
        }
        fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
            Name::BadAdd.on_stop(agent, world, reason);
            world.actions(agent.unwrap()).add(CountdownAction::new(1));
        }
        fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
            Name::BadAdd.on_remove(agent, world);
        }
        fn on_drop(self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
            Name::BadAdd.on_drop(agent, world, reason);
        }
    }

    let mut app = TestApp::new();

    let a = app.spawn_agent();
    app.actions(a).add(BadAddAction);

    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Add(Name::BadAdd, a),
            Hook::Start(Name::BadAdd, a),
            Hook::Stop(Name::BadAdd, Some(a), StopReason::Finished),
            Hook::Add(Name::Countdown, a),
            Hook::Start(Name::Countdown, a),
            Hook::Remove(Name::BadAdd, Some(a)),
            Hook::Drop(Name::BadAdd, Some(a), DropReason::Done)
        ]
    );

    app.hooks_mut().clear();
    app.update();

    assert_eq!(
        app.hooks().deref().clone(),
        vec![
            Hook::Stop(Name::Countdown, Some(a), StopReason::Finished),
            Hook::Remove(Name::Countdown, Some(a)),
            Hook::Drop(Name::Countdown, Some(a), DropReason::Done)
        ]
    );
}

#[test]
#[should_panic]
fn forever_action() {
    struct ForeverAction;
    impl Action for ForeverAction {
        fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
            true
        }
        fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
            true
        }
        fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {}
        fn on_drop(self: Box<Self>, agent: Option<Entity>, world: &mut World, _reason: DropReason) {
            world
                .actions(agent.unwrap())
                .start(false)
                .add(self as BoxedAction);
        }
    }

    let mut app = TestApp::new();
    let a = app.spawn_agent();
    app.actions(a).add(ForeverAction);
}
