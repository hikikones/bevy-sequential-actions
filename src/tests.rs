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

struct CountdownAction(usize);

impl Action for CountdownAction {
    fn on_start(
        &mut self,
        state: StartState,
        entity: Entity,
        world: &mut World,
        _commands: &mut ActionCommands,
    ) {
        match state {
            StartState::Start => {
                world.entity_mut(entity).insert(Countdown(self.0));
            }
            StartState::Resume => {
                world.entity_mut(entity).remove::<Paused>();
            }
        }
    }

    fn on_stop(&mut self, reason: StopReason, entity: Entity, world: &mut World) {
        match reason {
            StopReason::Finished => {
                world.entity_mut(entity).remove::<Countdown>();
            }
            StopReason::Canceled => {
                world.entity_mut(entity).remove::<Countdown>();
            }
            StopReason::Paused => {
                world.entity_mut(entity).insert(Paused);
            }
        }
    }
}

#[derive(Component)]
struct Countdown(usize);

#[derive(Component)]
struct Paused;

fn countdown_system(
    mut countdown_q: Query<(Entity, &mut Countdown), Without<Paused>>,
    mut commands: Commands,
) {
    for (entity, mut countdown) in countdown_q.iter_mut() {
        if countdown.0 == 0 {
            commands.actions(entity).finish();
            continue;
        }

        countdown.0 -= 1;
    }
}

struct EmptyAction;

impl Action for EmptyAction {
    fn on_start(
        &mut self,
        _state: StartState,
        entity: Entity,
        _world: &mut World,
        commands: &mut ActionCommands,
    ) {
        commands.actions(entity).finish();
    }

    fn on_stop(&mut self, _reason: StopReason, _entity: Entity, _world: &mut World) {}
}

#[test]
fn add2() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction(0));

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.actions(e).add(CountdownAction(0));

    assert!(ecs.get_action_queue(e).len() == 1);

    ecs.actions(e).add(CountdownAction(0));

    assert!(ecs.get_action_queue(e).len() == 2);
}

#[test]
fn push_without_submit() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .push(EmptyAction)
        .push(EmptyAction)
        .push(EmptyAction);

    // Must call submit for actions to be added
    ecs.run();

    assert!(ecs.get_current_action(e).is_none());
    assert!(ecs.get_action_queue(e).len() == 0);
}

#[test]
fn push_empty() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .push(EmptyAction)
        .push(EmptyAction)
        .push(EmptyAction)
        .submit();

    // Empty actions are recursively finished.
    ecs.run();

    assert!(ecs.get_current_action(e).is_none());
    assert!(ecs.get_action_queue(e).len() == 0);
}

#[test]
fn push2() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .push(CountdownAction(0)) // Finished after first run
        .push(CountdownAction(0)) // Current action after first run
        .push(CountdownAction(0)) // In action queue after first run
        .submit();

    ecs.run();

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 1);
}

#[test]
fn cancel2() {
    let mut ecs = ECS::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction(5));

    ecs.run();

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);
    assert!(ecs.world.entity(e).contains::<Countdown>());

    ecs.actions(e).stop(StopReason::Canceled);

    ecs.run();

    assert!(ecs.get_current_action(e).is_none());
    assert!(ecs.get_action_queue(e).len() == 0);
    assert!(!ecs.world.entity(e).contains::<Countdown>());
}

/////////////////////////////////////////////////////////////////////////////////////

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

    world.actions(e).finish();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);

    world.actions(e).finish();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

    world.actions(e).finish();

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

    world.actions(e).finish();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);

    world.actions(e).finish();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

    world.actions(e).finish();

    assert!(world.get::<CurrentAction>(e).unwrap().is_none());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
}

#[test]
fn cancel() {
    let mut world = World::new();

    let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

    world.actions(e).add(EmptyAction).add(EmptyAction);

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);

    world.actions(e).stop(StopReason::Canceled);

    assert!(world.get::<CurrentAction>(e).unwrap().is_none());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);

    world.actions(e).next();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

    world.actions(e).stop(StopReason::Canceled);

    assert!(world.get::<CurrentAction>(e).unwrap().is_none());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
}

#[test]
fn pause() {
    #[derive(Component)]
    struct Paused;

    struct PauseAction;
    impl Action for PauseAction {
        fn on_start(
            &mut self,
            state: StartState,
            entity: Entity,
            world: &mut World,
            _commands: &mut ActionCommands,
        ) {
            match state {
                StartState::Start => {
                    world.entity_mut(entity).insert(Paused);
                }
                StartState::Resume => {
                    world.entity_mut(entity).remove::<Paused>();
                }
            }
        }

        fn on_stop(&mut self, _reason: StopReason, _entity: Entity, _world: &mut World) {}
    }

    let mut world = World::new();

    let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

    world.actions(e).add(PauseAction);

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());

    world.actions(e).stop(StopReason::Paused);

    assert!(world.get::<CurrentAction>(e).unwrap().is_none());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);
    assert!(world.entity(e).contains::<Paused>());

    world
        .actions(e)
        .config(AddConfig {
            order: AddOrder::Front,
            start: true,
            repeat: false,
        })
        .add(EmptyAction);

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 1);
    assert!(world.entity(e).contains::<Paused>());

    world.actions(e).finish();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
    assert!(!world.entity(e).contains::<Paused>());

    world.actions(e).finish();

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

    world.actions(e).finish();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);

    world.actions(e).finish();

    assert!(world.get::<CurrentAction>(e).unwrap().is_some());
    assert!(world.get::<ActionQueue>(e).unwrap().len() == 0);
}

#[test]
fn despawn() {
    struct DespawnAction;
    impl Action for DespawnAction {
        fn on_start(
            &mut self,
            _state: StartState,
            entity: Entity,
            world: &mut World,
            _commands: &mut ActionCommands,
        ) {
            world.despawn(entity);
        }

        fn on_stop(&mut self, _reason: StopReason, _entity: Entity, _world: &mut World) {}
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
        fn on_start(
            &mut self,
            _state: StartState,
            entity: Entity,
            world: &mut World,
            _commands: &mut ActionCommands,
        ) {
            world.entity_mut(entity).insert(A);
        }

        fn on_stop(&mut self, _reason: StopReason, entity: Entity, world: &mut World) {
            world.entity_mut(entity).remove::<A>();
        }
    }
    impl Action for B {
        fn on_start(
            &mut self,
            _state: StartState,
            entity: Entity,
            world: &mut World,
            _commands: &mut ActionCommands,
        ) {
            world.entity_mut(entity).insert(B);
        }

        fn on_stop(&mut self, _reason: StopReason, entity: Entity, world: &mut World) {
            world.entity_mut(entity).remove::<B>();
        }
    }
    impl Action for C {
        fn on_start(
            &mut self,
            _state: StartState,
            entity: Entity,
            world: &mut World,
            _commands: &mut ActionCommands,
        ) {
            world.entity_mut(entity).insert(C);
        }

        fn on_stop(&mut self, _reason: StopReason, entity: Entity, world: &mut World) {
            world.entity_mut(entity).remove::<C>();
        }
    }

    let mut world = World::new();

    let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

    // A, B, C
    world.actions(e).add(A).add(B).add(C);

    assert!(world.entity(e).contains::<A>());

    world.actions(e).finish();

    assert!(world.entity(e).contains::<B>());

    world.actions(e).finish();

    assert!(world.entity(e).contains::<C>());

    // C, B, A
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

    world.actions(e).finish();

    assert!(world.entity(e).contains::<B>());

    world.actions(e).finish();

    assert!(world.entity(e).contains::<A>());

    // A, B, C
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
        .reverse()
        .submit()
        .next();

    assert!(world.entity(e).contains::<A>());

    world.actions(e).finish();

    assert!(world.entity(e).contains::<B>());

    world.actions(e).finish();

    assert!(world.entity(e).contains::<C>());
}
