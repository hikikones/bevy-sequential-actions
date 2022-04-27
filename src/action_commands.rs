use bevy::prelude::*;

use crate::{world::ActionsWorldExt, *};

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

pub struct ActionCommandBuilder<'a> {
    actor: Entity,
    config: AddConfig,
    actions: Vec<Box<dyn Action>>,
    commands: &'a mut ActionCommands,
}

impl<'a> ActionCommandBuilder<'a> {
    pub fn add(mut self, action: impl IntoAction) -> Self {
        self.actions.push(action.into_boxed());
        self
    }

    pub fn reverse(mut self) -> Self {
        self.actions.reverse();
        self
    }

    pub fn submit(self) {
        for action in self.actions {
            self.commands
                .0
                .push(ActionCommand::Add(self.actor, action, self.config));
        }
    }
}
