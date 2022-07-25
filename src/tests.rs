use bevy::prelude::*;

use crate::*;

struct EmptyAction;
impl Action for EmptyAction {
    fn start(&mut self, _entity: Entity, _world: &mut World, _commands: &mut ActionCommands) {}
    fn stop(&mut self, _entity: Entity, _world: &mut World) {}
}

#[test]
fn add() {
    let mut world = World::new();

    let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

    world
        .actions(e)
        .add(EmptyAction)
        .add(EmptyAction)
        .add(EmptyAction);

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 2);

    world.actions(e).next();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);

    world.actions(e).next();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

    world.actions(e).next();

    assert!(world.get::<CurrentAction>(e).unwrap().is_none());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
}

#[test]
fn push() {
    let mut world = World::new();

    let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

    world
        .actions(e)
        .push(EmptyAction)
        .push(EmptyAction)
        .push(EmptyAction);

    assert!(world.get::<CurrentAction>(e).unwrap().is_none());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

    world
        .actions(e)
        .push(EmptyAction)
        .push(EmptyAction)
        .push(EmptyAction)
        .submit();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 2);

    world.actions(e).next();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);

    world.actions(e).next();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

    world.actions(e).next();

    assert!(world.get::<CurrentAction>(e).unwrap().is_none());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
}

#[test]
fn stop() {
    let mut world = World::new();

    let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

    world.actions(e).add(EmptyAction);

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

    world.actions(e).stop();

    assert!(world.get::<CurrentAction>(e).unwrap().is_none());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);

    world.actions(e).next();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

    world.actions(e).next();

    assert!(world.get::<CurrentAction>(e).unwrap().is_none());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
}

#[test]
fn clear() {
    let mut world = World::new();

    let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

    world
        .actions(e)
        .add(EmptyAction)
        .add(EmptyAction)
        .add(EmptyAction)
        .add(EmptyAction)
        .add(EmptyAction);

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 4);

    world.actions(e).clear();

    assert!(world.get::<CurrentAction>(e).unwrap().is_none());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
}

#[test]
fn repeat() {
    let mut world = World::new();

    let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

    world
        .actions(e)
        .config(AddConfig {
            order: AddOrder::Back,
            start: true,
            repeat: true,
        })
        .add(EmptyAction);

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

    world.actions(e).next();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

    world.actions(e).next();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
}

#[test]
fn despawn() {
    struct DespawnAction;
    impl Action for DespawnAction {
        fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
            world.despawn(entity);
        }
        fn stop(&mut self, _entity: Entity, _world: &mut World) {}
    }

    let mut world = World::new();

    let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

    world.actions(e).add(DespawnAction);

    assert!(world.get_entity(e).is_none());
}

#[test]
fn order() {
    #[derive(Component)]
    struct A;
    #[derive(Component)]
    struct B;
    #[derive(Component)]
    struct C;

    impl Action for A {
        fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
            world.entity_mut(entity).insert(A);
        }
        fn stop(&mut self, entity: Entity, world: &mut World) {
            world.entity_mut(entity).remove::<A>();
        }
    }
    impl Action for B {
        fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
            world.entity_mut(entity).insert(B);
        }
        fn stop(&mut self, entity: Entity, world: &mut World) {
            world.entity_mut(entity).remove::<B>();
        }
    }
    impl Action for C {
        fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
            world.entity_mut(entity).insert(C);
        }
        fn stop(&mut self, entity: Entity, world: &mut World) {
            world.entity_mut(entity).remove::<C>();
        }
    }

    let mut world = World::new();

    let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

    world.actions(e).add(A).add(B).add(C);

    assert!(world.entity(e).contains::<A>());

    world.actions(e).next();

    assert!(world.entity(e).contains::<B>());

    world.actions(e).next();

    assert!(world.entity(e).contains::<C>());

    world
        .actions(e)
        .clear()
        .config(AddConfig {
            order: AddOrder::Front,
            start: false,
            repeat: false,
        })
        .push(A)
        .push(B)
        .push(C)
        .submit()
        .next();

    assert!(world.entity(e).contains::<C>());

    world.actions(e).next();

    assert!(world.entity(e).contains::<B>());

    world.actions(e).next();

    assert!(world.entity(e).contains::<A>());
}
