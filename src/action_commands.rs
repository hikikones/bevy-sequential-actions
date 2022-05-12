use bevy_ecs::prelude::*;

use crate::*;

/// Commands for modifying actions inside the [`Action`] trait.
#[derive(Default)]
pub struct ActionCommands(Vec<ActionCommand>);

impl ActionCommands {
    /// Returns an [`EntityActions`] for the requested [`Entity`].
    pub fn action(&mut self, entity: Entity) -> EntityActions {
        EntityActions {
            entity,
            config: AddConfig::default(),
            actions: Vec::new(),
            commands: self,
        }
    }
}

/// Modify actions using [`ActionCommands`].
pub struct EntityActions<'a> {
    entity: Entity,
    config: AddConfig,
    actions: Vec<(Box<dyn Action>, AddConfig)>,
    commands: &'a mut ActionCommands,
}

enum ActionCommand {
    Add(Entity, Box<dyn Action>, AddConfig),
    Next(Entity),
    Stop(Entity),
    Clear(Entity),
}

impl ModifyActionsExt for EntityActions<'_> {
    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add(self, action: impl IntoAction) -> Self {
        self.commands.0.push(ActionCommand::Add(
            self.entity,
            action.into_boxed(),
            self.config,
        ));
        self
    }

    fn next(self) -> Self {
        self.commands.0.push(ActionCommand::Next(self.entity));
        self
    }

    fn stop(self) -> Self {
        self.commands.0.push(ActionCommand::Stop(self.entity));
        self
    }

    fn clear(self) -> Self {
        self.commands.0.push(ActionCommand::Clear(self.entity));
        self
    }

    fn push(mut self, action: impl IntoAction) -> Self {
        self.actions.push((action.into_boxed(), self.config));
        self
    }

    fn reverse(mut self) -> Self {
        self.actions.reverse();
        self
    }

    fn submit(mut self) -> Self {
        for (action, config) in self.actions.drain(..) {
            self.commands
                .0
                .push(ActionCommand::Add(self.entity, action, config));
        }
        self
    }
}

impl ActionCommands {
    pub(super) fn apply(self, world: &mut World) {
        for cmd in self.0 {
            match cmd {
                ActionCommand::Add(entity, action, config) => {
                    world.action(entity).config(config).add(action);
                }
                ActionCommand::Next(entity) => {
                    world.action(entity).next();
                }
                ActionCommand::Stop(entity) => {
                    world.action(entity).stop();
                }
                ActionCommand::Clear(entity) => {
                    world.action(entity).clear();
                }
            }
        }
    }
}
