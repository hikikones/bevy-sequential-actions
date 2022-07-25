use std::marker::PhantomData;

use bevy::ecs::{
    system::{Command, Commands, Resource},
    world::{Mut, World},
};

pub trait ResourceScopeExt {
    fn resource_scope<T: Resource>(
        &mut self,
        f: impl FnOnce(&mut World, &mut T) + Send + Sync + 'static,
    );
}

pub struct ResourceScopeCommand<T: Resource, F: FnOnce(&mut World, &mut T) + Send + Sync + 'static>
{
    func: F,
    _res: PhantomData<T>,
}

impl<T: Resource, F: FnOnce(&mut World, &mut T) + Send + Sync> Command
    for ResourceScopeCommand<T, F>
{
    fn write(self, world: &mut World) {
        world.resource_scope(|world, mut res: Mut<T>| {
            (self.func)(world, res.as_mut());
        });
    }
}

impl ResourceScopeExt for Commands<'_, '_> {
    fn resource_scope<T: Resource>(
        &mut self,
        f: impl FnOnce(&mut World, &mut T) + Send + Sync + 'static,
    ) {
        self.add(ResourceScopeCommand {
            func: f,
            _res: PhantomData,
        });
    }
}
