use bevy::{
    ecs::system::{Command, CommandQueue},
    prelude::*,
};
use bevy_sequential_actions::*;

pub struct CommandAction<T: Command> {
    cmd: Option<T>,
}

impl<T: Command> CommandAction<T> {
    pub fn new(cmd: T) -> Self {
        Self { cmd: Some(cmd) }
    }
}

impl<T: Command> Action for CommandAction<T> {
    fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
        let mut queue = CommandQueue::default();
        let mut cmds = Commands::new(&mut queue, world);
        cmds.add(self.cmd.take().unwrap());
        queue.apply(world);

        commands.actions(entity).finish();
    }

    fn finish(&mut self, _entity: Entity, _world: &mut World) {}
}
