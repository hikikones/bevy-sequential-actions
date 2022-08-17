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

impl<'a> EntityActions<'a> {
    /// Run a custom function `f` after [`Action::on_start`] has been called.
    pub fn custom<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut World) + 'static,
    {
        self.commands.0.push(ActionCommand::Custom(Box::new(f)));
        self
    }
}

impl<'a> ModifyActions for EntityActions<'a> {
    type Builder = ActionsBuilder<'a>;

    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add<T: IntoAction>(self, action: T) -> Self {
        self.commands.0.push(ActionCommand::Add(
            self.entity,
            self.config,
            action.into_boxed(),
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

    fn skip(self) -> Self {
        self.commands.0.push(ActionCommand::Skip(self.entity));
        self
    }

    fn clear(self) -> Self {
        self.commands.0.push(ActionCommand::Clear(self.entity));
        self
    }

    fn builder(self) -> Self::Builder {
        ActionsBuilder {
            config: AddConfig::default(),
            actions: Vec::new(),
            modifier: self,
        }
    }
}

/// Build a list of actions using [`ActionCommands`].
pub struct ActionsBuilder<'a> {
    config: AddConfig,
    actions: Vec<(Box<dyn Action>, AddConfig)>,
    modifier: EntityActions<'a>,
}

impl<'a> ActionBuilder for ActionsBuilder<'a> {
    type Modifier = EntityActions<'a>;

    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn push<T: IntoAction>(mut self, action: T) -> Self {
        self.actions.push((action.into_boxed(), self.config));
        self
    }

    fn reverse(mut self) -> Self {
        self.actions.reverse();
        self
    }

    fn submit(self) -> Self::Modifier {
        for (action, config) in self.actions {
            self.modifier
                .commands
                .0
                .push(ActionCommand::Add(self.modifier.entity, config, action));
        }

        self.modifier
    }
}

enum ActionCommand {
    Add(Entity, AddConfig, Box<dyn Action>),
    Next(Entity),
    Finish(Entity),
    Pause(Entity),
    Stop(Entity, StopReason),
    Skip(Entity),
    Clear(Entity),
    Custom(Box<dyn FnOnce(&mut World)>),
}

impl ActionCommands {
    pub(super) fn apply(self, world: &mut World) {
        for cmd in self.0 {
            match cmd {
                ActionCommand::Add(entity, config, action) => {
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
                ActionCommand::Skip(entity) => {
                    world.actions(entity).skip();
                }
                ActionCommand::Clear(entity) => {
                    world.actions(entity).clear();
                }
                ActionCommand::Custom(f) => {
                    f(world);
                }
            }
        }
    }
}
