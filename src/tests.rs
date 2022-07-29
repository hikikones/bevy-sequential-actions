use std::marker::PhantomData;

use bevy::prelude::*;

use crate::*;

struct ECS {
    world: World,
    schedule: Schedule,
}

impl ECS {
    fn new() -> Self {
        let world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_stage("update", SystemStage::parallel());

        let mut ecs = Self { world, schedule };
        ecs.add_system(countdown_system);
        ecs
    }

    fn add_system<Param, S: IntoSystem<(), (), Param>>(&mut self, system: S) {
        self.schedule.add_system_to_stage("update", system);
    }

    fn run(&mut self) {
        self.schedule.run(&mut self.world);
    }

    fn spawn_action_entity(&mut self) -> Entity {
        self.world
            .spawn()
            .insert_bundle(ActionsBundle::default())
            .id()
    }

    fn actions(&mut self, entity: Entity) -> EntityWorldActions {
        self.world.actions(entity)
    }

    fn get_current_action(&self, entity: Entity) -> &CurrentAction {
        self.world.get::<CurrentAction>(entity).unwrap()
    }

    fn get_action_queue(&self, entity: Entity) -> &ActionQueue {
        self.world.get::<ActionQueue>(entity).unwrap()
    }
}

struct CountdownAction {
    count: usize,
    current: Option<usize>,
}

impl CountdownAction {
    fn new(count: usize) -> Self {
        Self {
            count,
            current: None,
        }
    }
}

#[derive(Component)]
struct Countdown(usize);

#[derive(Component)]
struct Finished;

#[derive(Component)]
struct Canceled;

#[derive(Component)]
struct Stopped;

impl Action for CountdownAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world
            .entity_mut(entity)
            .insert(Countdown(self.current.unwrap_or(self.count)));
    }

    fn on_finish(&mut self, entity: Entity, world: &mut World) {
        world
            .entity_mut(entity)
            .insert(Finished)
            .remove::<Countdown>();
        self.current = None;
    }

    fn on_cancel(&mut self, entity: Entity, world: &mut World) {
        world
            .entity_mut(entity)
            .insert(Canceled)
            .remove::<Countdown>();
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World) {
        let count = world
            .entity_mut(entity)
            .insert(Stopped)
            .remove::<Countdown>()
            .unwrap();
        self.current = Some(count.0);
    }
}

fn countdown_system(mut countdown_q: Query<(Entity, &mut Countdown)>, mut commands: Commands) {
    for (entity, mut countdown) in countdown_q.iter_mut() {
        countdown.0 = countdown.0.saturating_sub(1);
        if countdown.0 == 0 {
            commands.actions(entity).finish();
            continue;
        }
    }
}

struct EmptyAction;

impl Action for EmptyAction {
    fn on_start(&mut self, entity: Entity, _world: &mut World, commands: &mut ActionCommands) {
        commands.actions(entity).finish();
    }

    fn on_finish(&mut self, _entity: Entity, _world: &mut World) {}
    fn on_cancel(&mut self, _entity: Entity, _world: &mut World) {}
    fn on_stop(&mut self, _entity: Entity, _world: &mut World) {}
}

#[test]
fn add() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.get_action_queue(e).len() == 1);

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.get_action_queue(e).len() == 2);
}

#[test]
fn next() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Countdown>());
    assert!(!ecs.world.entity(e).contains::<Canceled>());

    ecs.actions(e).next();

    assert!(!ecs.world.entity(e).contains::<Countdown>());
    assert!(ecs.world.entity(e).contains::<Canceled>());
}

#[test]
fn finish() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Countdown>());
    assert!(!ecs.world.entity(e).contains::<Finished>());

    ecs.actions(e).finish();

    assert!(!ecs.world.entity(e).contains::<Countdown>());
    assert!(ecs.world.entity(e).contains::<Finished>());
}

#[test]
fn stop() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Countdown>());
    assert!(!ecs.world.entity(e).contains::<Stopped>());

    ecs.actions(e).stop();

    assert!(!ecs.world.entity(e).contains::<Countdown>());
    assert!(ecs.world.entity(e).contains::<Stopped>());
}

#[test]
fn clear() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .add(CountdownAction::new(0))
        .add(CountdownAction::new(0))
        .add(CountdownAction::new(0))
        .clear();

    assert!(ecs.get_current_action(e).is_none());
    assert!(ecs.get_action_queue(e).len() == 0);
    assert!(ecs.world.entity(e).contains::<Canceled>());
}

#[test]
fn push() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .push(EmptyAction)
        .push(EmptyAction)
        .push(EmptyAction);

    assert!(ecs.get_current_action(e).is_none());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.actions(e)
        .push(EmptyAction)
        .push(EmptyAction)
        .push(EmptyAction)
        .submit();

    assert!(ecs.get_current_action(e).is_none());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.actions(e)
        .push(CountdownAction::new(0))
        .push(CountdownAction::new(0))
        .push(CountdownAction::new(0))
        .submit();

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 2);
}

#[test]
fn repeat() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .config(AddConfig {
            order: AddOrder::Back,
            start: true,
            repeat: true,
        })
        .add(CountdownAction::new(0));

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.run();

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);
}

#[test]
fn despawn() {
    struct DespawnAction;
    impl Action for DespawnAction {
        fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
            world.despawn(entity);
        }

        fn on_finish(&mut self, _entity: Entity, _world: &mut World) {}
        fn on_cancel(&mut self, _entity: Entity, _world: &mut World) {}
        fn on_stop(&mut self, _entity: Entity, _world: &mut World) {}
    }

    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .add(CountdownAction::new(0))
        .add(DespawnAction);

    ecs.run();

    assert!(ecs.world.get_entity(e).is_none());
}

#[test]
fn order() {
    #[derive(Default)]
    struct Order<T: Default + Component>(PhantomData<T>);
    impl<T: Default + Component> Action for Order<T> {
        fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
            world.entity_mut(entity).insert(T::default());
        }
        fn on_finish(&mut self, entity: Entity, world: &mut World) {
            world.entity_mut(entity).remove::<T>();
        }
        fn on_cancel(&mut self, _entity: Entity, _world: &mut World) {}
        fn on_stop(&mut self, _entity: Entity, _world: &mut World) {}
    }

    #[derive(Default, Component)]
    struct A;
    #[derive(Default, Component)]
    struct B;
    #[derive(Default, Component)]
    struct C;

    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    // A, B, C
    ecs.actions(e)
        .add(Order::<A>::default())
        .add(Order::<B>::default())
        .add(Order::<C>::default());

    assert!(ecs.world.entity(e).contains::<A>());

    ecs.actions(e).finish();

    assert!(ecs.world.entity(e).contains::<B>());

    ecs.actions(e).finish();

    assert!(ecs.world.entity(e).contains::<C>());

    // C, B, A
    ecs.actions(e)
        .clear()
        .config(AddConfig {
            order: AddOrder::Front,
            start: false,
            repeat: false,
        })
        .push(Order::<A>::default())
        .push(Order::<B>::default())
        .push(Order::<C>::default())
        .submit()
        .next();

    assert!(ecs.world.entity(e).contains::<C>());

    ecs.actions(e).finish();

    assert!(ecs.world.entity(e).contains::<B>());

    ecs.actions(e).finish();

    assert!(ecs.world.entity(e).contains::<A>());

    // A, B, C
    ecs.actions(e)
        .clear()
        .config(AddConfig {
            order: AddOrder::Front,
            start: false,
            repeat: false,
        })
        .push(Order::<A>::default())
        .push(Order::<B>::default())
        .push(Order::<C>::default())
        .reverse()
        .submit()
        .next();

    assert!(ecs.world.entity(e).contains::<A>());

    ecs.actions(e).finish();

    assert!(ecs.world.entity(e).contains::<B>());

    ecs.actions(e).finish();

    assert!(ecs.world.entity(e).contains::<C>());
}

#[test]
fn pause_resume() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    fn get_first_countdown_value(world: &mut World) -> usize {
        world.query::<&Countdown>().iter(world).next().unwrap().0
    }

    ecs.actions(e).add(CountdownAction::new(100));

    ecs.run();

    assert!(get_first_countdown_value(&mut ecs.world) == 99);

    ecs.actions(e)
        .stop()
        .config(AddConfig {
            order: AddOrder::Front,
            start: true,
            repeat: false,
        })
        .add(CountdownAction::new(2));

    ecs.run();
    ecs.run();

    assert!(get_first_countdown_value(&mut ecs.world) == 99);
}

// #[test]
// fn pause2() {
//     let mut ecs = ECS::new();
//     let e = ecs.spawn_action_entity();

//     ecs.actions(e).add(CountdownAction::new(0));

//     assert!(ecs.get_current_action(e).is_some());
//     assert!(ecs.world.entity(e).contains::<Countdown>());
//     assert!(!ecs.world.entity(e).contains::<Stopped>());

//     ecs.actions(e).stop(StopReason::Paused);

//     assert!(ecs.get_current_action(e).is_none());
//     assert!(!ecs.world.entity(e).contains::<Countdown>());
//     assert!(ecs.world.entity(e).contains::<Stopped>());
// }

// #[test]
// fn pause_resume() {
//     let mut ecs = ECS::new();
//     let e = ecs.spawn_action_entity();

//     fn get_first_countdown_value(world: &mut World) -> usize {
//         world.query::<&Countdown>().iter(world).next().unwrap().0
//     }

//     ecs.actions(e).add(CountdownAction::new(100));

//     ecs.run();

//     assert!(get_first_countdown_value(&mut ecs.world) == 99);

//     ecs.actions(e).stop(StopReason::Paused);

//     assert!(ecs.get_current_action(e).is_none());
//     assert!(ecs.world.entity(e).contains::<Stopped>());

//     ecs.actions(e)
//         .config(AddConfig {
//             order: AddOrder::Front,
//             start: true,
//             repeat: false,
//         })
//         .add(CountdownAction::new(2));

//     ecs.run();
//     ecs.run();

//     assert!(get_first_countdown_value(&mut ecs.world) == 99);
// }

// TODO: Hmmm how to fix
// #[test]
// fn pause_front() {
//     let mut ecs = ECS::new();
//     let e = ecs.spawn_action_entity();

//     ecs.actions(e)
//         .add(CountdownAction(5))
//         .stop(StopReason::Paused);

//     ecs.run();

//     assert!(ecs.get_current_action(e).is_none());
//     assert!(ecs.get_action_queue(e).len() == 1);
//     assert!(ecs.world.entity(e).contains::<Countdown>());
//     assert!(ecs.world.entity(e).contains::<Paused>());

//     // Add another action to the front of queue
//     ecs.actions(e)
//         .config(AddConfig {
//             order: AddOrder::Front,
//             start: true,
//             repeat: false,
//         })
//         .add(CountdownAction(0));

//     ecs.run();

//     // assert!(ecs.get_current_action(e).is_some());
//     assert!(ecs.get_current_action(e).is_some());
//     assert!(ecs.get_action_queue(e).len() == 1);
//     assert!(ecs.world.entity(e).contains::<Countdown>());
//     assert!(ecs.world.entity(e).contains::<Paused>());

//     ecs.run();
//     ecs.run();
//     ecs.run();
//     ecs.run();
//     ecs.run();
//     ecs.run();

//     assert!(ecs.get_action_queue(e).len() == 0);
//     assert!(!ecs.world.entity(e).contains::<Paused>());
// }

/////////////////////////////////////////////////////////////////////////////////////

// #[test]
// fn add() {
//     let mut world = World::new();

//     let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

//     world
//         .actions(e)
//         .add(EmptyAction)
//         .add(EmptyAction)
//         .add(EmptyAction);

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 2);

//     world.actions(e).finish();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);

//     world.actions(e).finish();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

//     world.actions(e).finish();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_none());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
// }

// #[test]
// fn push() {
//     let mut world = World::new();

//     let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

//     world
//         .actions(e)
//         .push(EmptyAction)
//         .push(EmptyAction)
//         .push(EmptyAction);

//     assert!(world.get::<CurrentAction>(e).unwrap().is_none());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

//     world
//         .actions(e)
//         .push(EmptyAction)
//         .push(EmptyAction)
//         .push(EmptyAction)
//         .submit();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 2);

//     world.actions(e).finish();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);

//     world.actions(e).finish();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

//     world.actions(e).finish();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_none());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
// }

// #[test]
// fn cancel() {
//     let mut world = World::new();

//     let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

//     world.actions(e).add(EmptyAction).add(EmptyAction);

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);

//     world.actions(e).stop(StopReason::Canceled);

//     assert!(world.get::<CurrentAction>(e).unwrap().is_none());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);

//     world.actions(e).next();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

//     world.actions(e).stop(StopReason::Canceled);

//     assert!(world.get::<CurrentAction>(e).unwrap().is_none());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
// }

// #[test]
// fn pause() {
//     #[derive(Component)]
//     struct Paused;

//     struct PauseAction;
//     impl Action for PauseAction {
//         fn on_start(
//             &mut self,
//             state: StartState,
//             entity: Entity,
//             world: &mut World,
//             _commands: &mut ActionCommands,
//         ) {
//             match state {
//                 StartState::Start => {
//                     world.entity_mut(entity).insert(Paused);
//                 }
//                 StartState::Resume => {
//                     world.entity_mut(entity).remove::<Paused>();
//                 }
//             }
//         }

//         fn on_stop(&mut self, _reason: StopReason, _entity: Entity, _world: &mut World) {}
//     }

//     let mut world = World::new();

//     let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

//     world.actions(e).add(PauseAction);

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());

//     world.actions(e).stop(StopReason::Paused);

//     assert!(world.get::<CurrentAction>(e).unwrap().is_none());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);
//     assert!(world.entity(e).contains::<Paused>());

//     world
//         .actions(e)
//         .config(AddConfig {
//             order: AddOrder::Front,
//             start: true,
//             repeat: false,
//         })
//         .add(EmptyAction);

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);
//     assert!(world.entity(e).contains::<Paused>());

//     world.actions(e).finish();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
//     assert!(!world.entity(e).contains::<Paused>());

//     world.actions(e).finish();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_none());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
// }

// #[test]
// fn clear() {
//     let mut world = World::new();

//     let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

//     world
//         .actions(e)
//         .add(EmptyAction)
//         .add(EmptyAction)
//         .add(EmptyAction)
//         .add(EmptyAction)
//         .add(EmptyAction);

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 4);

//     world.actions(e).clear();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_none());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
// }

// #[test]
// fn repeat() {
//     let mut world = World::new();

//     let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

//     world
//         .actions(e)
//         .config(AddConfig {
//             order: AddOrder::Back,
//             start: true,
//             repeat: true,
//         })
//         .add(EmptyAction);

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

//     world.actions(e).finish();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

//     world.actions(e).finish();

//     assert!(world.get::<CurrentAction>(e).unwrap().is_some());
//     assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
// }

// #[test]
// fn despawn() {
//     struct DespawnAction;
//     impl Action for DespawnAction {
//         fn on_start(
//             &mut self,
//             _state: StartState,
//             entity: Entity,
//             world: &mut World,
//             _commands: &mut ActionCommands,
//         ) {
//             world.despawn(entity);
//         }

//         fn on_stop(&mut self, _reason: StopReason, _entity: Entity, _world: &mut World) {}
//     }

//     let mut world = World::new();

//     let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

//     world.actions(e).add(DespawnAction);

//     assert!(world.get_entity(e).is_none());
// }

// #[test]
// fn order() {
//     #[derive(Component)]
//     struct A;
//     #[derive(Component)]
//     struct B;
//     #[derive(Component)]
//     struct C;

//     impl Action for A {
//         fn on_start(
//             &mut self,
//             _state: StartState,
//             entity: Entity,
//             world: &mut World,
//             _commands: &mut ActionCommands,
//         ) {
//             world.entity_mut(entity).insert(A);
//         }

//         fn on_stop(&mut self, _reason: StopReason, entity: Entity, world: &mut World) {
//             world.entity_mut(entity).remove::<A>();
//         }
//     }
//     impl Action for B {
//         fn on_start(
//             &mut self,
//             _state: StartState,
//             entity: Entity,
//             world: &mut World,
//             _commands: &mut ActionCommands,
//         ) {
//             world.entity_mut(entity).insert(B);
//         }

//         fn on_stop(&mut self, _reason: StopReason, entity: Entity, world: &mut World) {
//             world.entity_mut(entity).remove::<B>();
//         }
//     }
//     impl Action for C {
//         fn on_start(
//             &mut self,
//             _state: StartState,
//             entity: Entity,
//             world: &mut World,
//             _commands: &mut ActionCommands,
//         ) {
//             world.entity_mut(entity).insert(C);
//         }

//         fn on_stop(&mut self, _reason: StopReason, entity: Entity, world: &mut World) {
//             world.entity_mut(entity).remove::<C>();
//         }
//     }

//     let mut world = World::new();

//     let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

//     // A, B, C
//     world.actions(e).add(A).add(B).add(C);

//     assert!(world.entity(e).contains::<A>());

//     world.actions(e).finish();

//     assert!(world.entity(e).contains::<B>());

//     world.actions(e).finish();

//     assert!(world.entity(e).contains::<C>());

//     // C, B, A
//     world
//         .actions(e)
//         .clear()
//         .config(AddConfig {
//             order: AddOrder::Front,
//             start: false,
//             repeat: false,
//         })
//         .push(A)
//         .push(B)
//         .push(C)
//         .submit()
//         .next();

//     assert!(world.entity(e).contains::<C>());

//     world.actions(e).finish();

//     assert!(world.entity(e).contains::<B>());

//     world.actions(e).finish();

//     assert!(world.entity(e).contains::<A>());

//     // A, B, C
//     world
//         .actions(e)
//         .clear()
//         .config(AddConfig {
//             order: AddOrder::Front,
//             start: false,
//             repeat: false,
//         })
//         .push(A)
//         .push(B)
//         .push(C)
//         .reverse()
//         .submit()
//         .next();

//     assert!(world.entity(e).contains::<A>());

//     world.actions(e).finish();

//     assert!(world.entity(e).contains::<B>());

//     world.actions(e).finish();

//     assert!(world.entity(e).contains::<C>());
// }
