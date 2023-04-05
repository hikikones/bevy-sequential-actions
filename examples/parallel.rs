use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugin(ScheduleRunnerPlugin)
        .add_plugin(SequentialActionsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(ActionsBundle::new()).id();
    commands.actions(agent).add_many(actions![
        A,
        B,
        C,
        AB { a: A, b: B },
        ABC { a: A, b: B, c: C },
        |_, world: &mut World| { world.send_event(AppExit) }
    ]);
}

struct A;
impl Action for A {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) {
        println!("A");
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

struct B;
impl Action for B {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) {
        println!("B");
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

struct C;
impl Action for C {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) {
        println!("C");
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

struct AB {
    a: A,
    b: B,
}
impl Action for AB {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.a.is_finished(agent, world) && self.b.is_finished(agent, world)
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) {
        self.a.on_start(agent, world);
        self.b.on_start(agent, world);
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        self.a.on_stop(agent, world, reason);
        self.b.on_stop(agent, world, reason);
    }
}

struct ABC {
    a: A,
    b: B,
    c: C,
}
impl Action for ABC {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.a.is_finished(agent, world)
            && self.b.is_finished(agent, world)
            && self.c.is_finished(agent, world)
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) {
        self.a.on_start(agent, world);
        self.b.on_start(agent, world);
        self.c.on_start(agent, world);
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        self.a.on_stop(agent, world, reason);
        self.b.on_stop(agent, world, reason);
        self.c.on_stop(agent, world, reason);
    }
}
