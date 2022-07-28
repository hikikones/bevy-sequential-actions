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
    Finish(Entity),
    Stop(Entity, StopReason),
    Clear(Entity),
}

impl ModifyActions for EntityActions<'_> {
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
                    world.actions(entity).config(config).add(action);
                }
                ActionCommand::Next(entity) => {
                    world.actions(entity).next();
                }
                ActionCommand::Finish(entity) => {
                    world.actions(entity).finish();
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
