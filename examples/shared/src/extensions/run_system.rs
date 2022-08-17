use bevy::{
    ecs::system::{IntoSystem, System},
    ecs::world::World,
};

pub trait RunSystemExt {
    fn run_system<S, P>(&mut self, system: S)
    where
        S: IntoSystem<(), (), P>,
        P: Send + Sync + 'static;
}

impl RunSystemExt for World {
    fn run_system<S, P>(&mut self, system: S)
    where
        S: IntoSystem<(), (), P>,
        P: Send + Sync + 'static,
    {
        let mut s = S::into_system(system);
        s.initialize(self);
        s.run((), self);
        s.apply_buffers(self);
    }
}
