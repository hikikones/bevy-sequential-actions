use bevy_ecs::prelude::*;

use crate::*;

/// Commands for modifying actions inside the [`Action`] trait.
#[derive(Default)]
pub struct ActionCommands(Vec<ActionCommand>);

impl<'a> ActionsProxy<'a> for ActionCommands {
    type Modifier = EntityActions<'a>;

    fn actions(&'a mut self, entity: Entity) -> EntityActions<'a> {
        EntityActions {
            entity,
            config: AddConfig::default(),
            commands: self,
        }
    }
}

/// Modify actions using [`ActionCommands`].
pub struct EntityActions<'a> {
    entity: Entity,
    config: AddConfig,
    commands: &'a mut ActionCommands,
}

enum ActionCommand {
    Add(Entity, Box<dyn Action>, AddConfig),
    Next(Entity),
    Finish(Entity),
    Pause(Entity),
    Stop(Entity, StopReason),
    Clear(Entity),
}

impl<'a> ModifyActions for EntityActions<'a> {
    type Builder = ActionsBuilder<'a>;

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

    fn finish(self) -> Self {
        self.commands.0.push(ActionCommand::Finish(self.entity));
        self
    }

    fn pause(self) -> Self {
        self.commands.0.push(ActionCommand::Pause(self.entity));
        self
    }

    fn stop(self, reason: StopReason) -> Self {
        self.commands
            .0
            .push(ActionCommand::Stop(self.entity, reason));
        self
    }

    fn clear(self) -> Self {
        self.commands.0.push(ActionCommand::Clear(self.entity));
        self
    }

    fn builder(self) -> Self::Builder {
        ActionsBuilder {
            entity: self.entity,
            config: self.config,
            actions: Vec::new(),
            commands: self.commands,
        }
    }
}

pub struct ActionsBuilder<'a> {
    entity: Entity,
    config: AddConfig,
    actions: Vec<(Box<dyn Action>, AddConfig)>,
    commands: &'a mut ActionCommands,
}

impl<'a> ActionBuilder for ActionsBuilder<'a> {
    type Modifier = EntityActions<'a>;

    fn push(mut self, action: impl IntoAction) -> Self {
        self.actions.push((action.into_boxed(), self.config));
        self
    }

    fn reverse(mut self) -> Self {
        self.actions.reverse();
        self
    }

    fn submit(self) -> Self::Modifier {
        for (action, config) in self.actions {
            self.commands
                .0
                .push(ActionCommand::Add(self.entity, action, config));
        }

        EntityActions {
            entity: self.entity,
            config: self.config,
            commands: self.commands,
        }
    }
}

impl ActionCommands {
    pub(super) fn apply(self, world: &mut World) {
        for cmd in self.0 {
            match cmd {
                ActionCommand::Add(entity, action, config) => {
                    world.actions(entity).config(config).add(action);
                }
                ActionCommand::Next(entity) => {
                    world.actions(entity).next();
                }
                ActionCommand::Finish(entity) => {
                    world.actions(entity).finish();
                }
                ActionCommand::Pause(entity) => {
                    world.actions(entity).pause();
                }
                ActionCommand::Stop(entity, reason) => {
                    world.actions(entity).stop(reason);
                }
                ActionCommand::Clear(entity) => {
                    world.actions(entity).clear();
                }
            }
        }
    }
}
