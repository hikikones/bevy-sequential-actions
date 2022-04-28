use bevy::{ecs::system::*, ecs::world::World};

pub trait RunSystemExt {
    fn run_system<Params: Send + Sync + 'static>(
        &mut self,
        system: impl IntoSystem<(), (), Params>,
    );
}

impl RunSystemExt for World {
    fn run_system<Params: Send + Sync + 'static>(
        &mut self,
        system: impl IntoSystem<(), (), Params>,
    ) {
        let mut system = IntoSystem::into_system(system);
        system.initialize(self);
        system.run((), self);
        system.apply_buffers(self);
    }
}

struct RunSystemCommand(Box<dyn System<In = (), Out = ()>>);

impl Command for RunSystemCommand {
    fn write(mut self, world: &mut World) {
        self.0.initialize(world);
        self.0.run((), world);
        self.0.apply_buffers(world);
    }
}

impl RunSystemExt for Commands<'_, '_> {
    fn run_system<Params: Send + Sync + 'static>(
        &mut self,
        system: impl IntoSystem<(), (), Params>,
    ) {
        self.add(RunSystemCommand(Box::new(IntoSystem::into_system(system))));
    }
}
