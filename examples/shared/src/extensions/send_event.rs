use bevy::{
    ecs::{
        event::Events,
        system::{Command, Resource},
    },
    prelude::*,
};

pub trait SendEventExt {
    fn send_event<T: Resource>(&mut self, event: T);
}

impl SendEventExt for World {
    fn send_event<T: Resource>(&mut self, event: T) {
        self.resource_mut::<Events<T>>().send(event);
    }
}

pub struct SendEventCommand<T: Resource> {
    event: T,
}

impl<T: Resource> Command for SendEventCommand<T> {
    fn write(self, world: &mut World) {
        world.send_event(self.event);
    }
}

impl SendEventExt for Commands<'_, '_> {
    fn send_event<T: Resource>(&mut self, event: T) {
        self.add(SendEventCommand { event });
    }
}
