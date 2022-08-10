use std::marker::PhantomData;

use bevy::prelude::*;

use crate::*;

struct Ecs {
    world: World,
    schedule: Schedule,
}

impl Ecs {
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
struct Paused;

impl Action for CountdownAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world
            .entity_mut(entity)
            .insert(Countdown(self.current.unwrap_or(self.count)));
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
        let mut e = world.entity_mut(entity);
        let count = e.remove::<Countdown>();

        match reason {
            StopReason::Finished => {
                self.current = None;
                e.insert(Finished);
            }
            StopReason::Canceled => {
                self.current = None;
                e.insert(Canceled);
            }
            StopReason::Paused => {
                self.current = Some(count.unwrap().0);
                e.insert(Paused);
            }
        }
    }
}

fn countdown_system(mut countdown_q: Query<(Entity, &mut Countdown)>, mut commands: Commands) {
    for (entity, mut countdown) in countdown_q.iter_mut() {
        countdown.0 = countdown.0.saturating_sub(1);
        if countdown.0 == 0 {
            commands.actions(entity).finish();
        }
    }
}

struct EmptyAction;

impl Action for EmptyAction {
    fn on_start(&mut self, entity: Entity, _world: &mut World, commands: &mut ActionCommands) {
        commands.actions(entity).finish();
    }

    fn on_stop(&mut self, _entity: Entity, _world: &mut World, _reason: StopReason) {}
}

#[test]
fn add() {
    let mut ecs = Ecs::new();
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
#[should_panic]
fn add_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.world.spawn().id();
    ecs.actions(e).add(EmptyAction);
}

#[test]
fn next() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Countdown>());
    assert!(!ecs.world.entity(e).contains::<Canceled>());

    ecs.actions(e).next();

    assert!(!ecs.world.entity(e).contains::<Countdown>());
    assert!(ecs.world.entity(e).contains::<Canceled>());
}

#[test]
#[should_panic]
fn next_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.world.spawn().id();
    ecs.actions(e).next();
}

#[test]
fn finish() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Countdown>());
    assert!(!ecs.world.entity(e).contains::<Finished>());

    ecs.actions(e).finish();

    assert!(!ecs.world.entity(e).contains::<Countdown>());
    assert!(ecs.world.entity(e).contains::<Finished>());
}

#[test]
#[should_panic]
fn finish_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.world.spawn().id();
    ecs.actions(e).finish();
}

#[test]
fn pause() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e).add(CountdownAction::new(0));

    assert!(ecs.world.entity(e).contains::<Countdown>());
    assert!(!ecs.world.entity(e).contains::<Paused>());

    ecs.actions(e).pause();

    assert!(!ecs.world.entity(e).contains::<Countdown>());
    assert!(ecs.world.entity(e).contains::<Paused>());
}

#[test]
#[should_panic]
fn pause_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.world.spawn().id();
    ecs.actions(e).stop(StopReason::Paused);
}

#[test]
fn skip() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .config(AddConfig {
            order: AddOrder::Back,
            start: false,
            repeat: false,
        })
        .add(EmptyAction)
        .add(EmptyAction)
        .config(AddConfig {
            order: AddOrder::Back,
            start: false,
            repeat: true,
        })
        .add(EmptyAction);

    assert!(ecs.get_action_queue(e).len() == 3);

    ecs.actions(e).skip();

    assert!(ecs.get_action_queue(e).len() == 2);

    ecs.actions(e).skip();

    assert!(ecs.get_action_queue(e).len() == 1);

    ecs.actions(e).skip();

    assert!(ecs.get_action_queue(e).len() == 1);
}

#[test]
fn clear() {
    let mut ecs = Ecs::new();
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
#[should_panic]
fn clear_panic() {
    let mut ecs = Ecs::new();
    let e = ecs.world.spawn().id();
    ecs.actions(e).clear();
}

#[test]
fn push() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .builder()
        .push(EmptyAction)
        .push(EmptyAction)
        .push(EmptyAction);

    assert!(ecs.get_current_action(e).is_none());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.actions(e)
        .builder()
        .push(EmptyAction)
        .push(EmptyAction)
        .push(EmptyAction)
        .submit();

    assert!(ecs.get_current_action(e).is_none());
    assert!(ecs.get_action_queue(e).len() == 0);

    ecs.actions(e)
        .builder()
        .push(CountdownAction::new(0))
        .push(CountdownAction::new(0))
        .push(CountdownAction::new(0))
        .submit();

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 2);
}

#[test]
fn repeat() {
    let mut ecs = Ecs::new();
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

    ecs.run();

    assert!(ecs.get_current_action(e).is_some());
    assert!(ecs.get_action_queue(e).len() == 0);
}

#[test]
#[should_panic]
fn despawn() {
    struct DespawnAction;
    impl Action for DespawnAction {
        fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
            world.despawn(entity);
        }
        fn on_stop(&mut self, _entity: Entity, _world: &mut World, _reason: StopReason) {}
    }

    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    ecs.actions(e)
        .add(CountdownAction::new(0))
        .add(DespawnAction)
        .add(EmptyAction);

    ecs.run();

    assert!(ecs.world.get_entity(e).is_none());

    ecs.actions(e).add(EmptyAction);
}

#[test]
fn order() {
    #[derive(Default)]
    struct Order<T: Default + Component>(PhantomData<T>);
    impl<T: Default + Component> Action for Order<T> {
        fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
            world.entity_mut(entity).insert(T::default());
        }
        fn on_stop(&mut self, entity: Entity, world: &mut World, _reason: StopReason) {
            world.entity_mut(entity).remove::<T>();
        }
    }

    #[derive(Default, Component)]
    struct A;
    #[derive(Default, Component)]
    struct B;
    #[derive(Default, Component)]
    struct C;

    let mut ecs = Ecs::new();
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
        .builder()
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
        .builder()
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
    let mut ecs = Ecs::new();
    let e = ecs.spawn_action_entity();

    fn get_first_countdown_value(world: &mut World) -> usize {
        world.query::<&Countdown>().iter(world).next().unwrap().0
    }

    ecs.actions(e).add(CountdownAction::new(100));

    ecs.run();

    assert!(get_first_countdown_value(&mut ecs.world) == 99);

    ecs.actions(e)
        .pause()
        .config(AddConfig {
            order: AddOrder::Front,
            start: true,
            repeat: false,
        })
        .add(CountdownAction::new(2));

    ecs.run();
    ecs.run();
    ecs.run();

    assert!(get_first_countdown_value(&mut ecs.world) == 98);
}
