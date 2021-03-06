use bevy::{ecs::system::Command, prelude::*};

pub trait BevyState
where
    Self:
        core::fmt::Debug + Clone + Copy + PartialEq + Eq + core::hash::Hash + Sync + Send + 'static,
{
}

impl<T> BevyState for T where
    T: core::fmt::Debug + Clone + Copy + PartialEq + Eq + core::hash::Hash + Sync + Send + 'static
{
}

pub trait SetStateExt {
    fn set_state<T: BevyState>(&mut self, state: T);
}

impl SetStateExt for World {
    fn set_state<T: BevyState>(&mut self, state: T) {
        self.resource_mut::<State<T>>().set(state).unwrap();
    }
}

pub struct SetStateCommand<T: BevyState> {
    state: T,
}

impl<T: BevyState> SetStateCommand<T> {
    pub fn new(state: T) -> Self {
        Self { state }
    }
}

impl<T: BevyState> Command for SetStateCommand<T> {
    fn write(self, world: &mut World) {
        world.set_state(self.state);
    }
}

impl SetStateExt for Commands<'_, '_> {
    fn set_state<T: BevyState>(&mut self, state: T) {
        self.add(SetStateCommand { state })
    }
}
