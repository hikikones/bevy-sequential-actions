use bevy_ecs::prelude::*;

use crate::{world::ActionsWorldExt, *};

/// Commands for modifying actions in the [`Action`] trait.
#[derive(Default)]
pub struct ActionCommands(Vec<ActionCommand>);

enum ActionCommand {
    Add(Entity, Box<dyn Action>, AddConfig),
    Stop(Entity),
    Next(Entity),
    Clear(Entity),
}

impl AddActionExt for ActionCommands {
    fn add_action(&mut self, actor: Entity, action: impl IntoAction, config: AddConfig) {
        self.0
            .push(ActionCommand::Add(actor, action.into_boxed(), config));
    }
}

impl StopActionExt for ActionCommands {
    fn stop_action(&mut self, actor: Entity) {
        self.0.push(ActionCommand::Stop(actor));
    }
}

impl NextActionExt for ActionCommands {
    fn next_action(&mut self, actor: Entity) {
        self.0.push(ActionCommand::Next(actor));
    }
}

impl ClearActionsExt for ActionCommands {
    fn clear_actions(&mut self, actor: Entity) {
        self.0.push(ActionCommand::Clear(actor));
    }
}

impl ActionCommands {
    pub(super) fn apply(self, world: &mut World) {
        for cmd in self.0 {
            match cmd {
                ActionCommand::Add(actor, action, config) => {
                    world.add_action(actor, action, config);
                }
                ActionCommand::Stop(actor) => {
                    world.stop_action(actor);
                }
                ActionCommand::Next(actor) => {
                    world.next_action(actor);
                }
                ActionCommand::Clear(actor) => {
                    world.clear_actions(actor);
                }
            }
        }
    }

    pub fn action_builder(&mut self, actor: Entity, config: AddConfig) -> ActionCommandBuilder {
        ActionCommandBuilder {
            actor,
            config,
            actions: Vec::default(),
            commands: self,
        }
    }
}

/// [`Action`] builder struct for [`ActionCommands`].
pub struct ActionCommandBuilder<'a> {
    actor: Entity,
    config: AddConfig,
    actions: Vec<Box<dyn Action>>,
    commands: &'a mut ActionCommands,
}

impl<'a> ActionCommandBuilder<'a> {
    /// Push an [`Action`] to the builder list.
    /// No [`Action`] will be applied until [`ActionCommandBuilder::submit`] is called.
    pub fn push(mut self, action: impl IntoAction) -> Self {
        self.actions.push(action.into_boxed());
        self
    }

    /// Reverse the order for the currently pushed actions.
    pub fn reverse(mut self) -> Self {
        self.actions.reverse();
        self
    }

    /// Submit the pushed actions.
    pub fn submit(self) {
        for action in self.actions {
            self.commands
                .0
                .push(ActionCommand::Add(self.actor, action, self.config));
        }
    }
}
