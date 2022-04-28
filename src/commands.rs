use bevy_ecs::{
    prelude::*,
    system::{Command, CommandQueue},
};

use crate::{world::ActionsWorldExt, *};

//
// Trait impls
//

struct AddAction {
    actor: Entity,
    config: AddConfig,
    action: Box<dyn Action>,
}

struct StopAction {
    actor: Entity,
}

struct NextAction {
    actor: Entity,
}

struct ClearActions {
    actor: Entity,
}

impl Command for AddAction {
    fn write(self, world: &mut World) {
        world.add_action(self.actor, self.action, self.config);
    }
}

impl Command for StopAction {
    fn write(self, world: &mut World) {
        world.stop_action(self.actor);
    }
}

impl Command for NextAction {
    fn write(self, world: &mut World) {
        world.next_action(self.actor);
    }
}

impl Command for ClearActions {
    fn write(self, world: &mut World) {
        world.clear_actions(self.actor);
    }
}

impl AddActionExt for Commands<'_, '_> {
    fn add_action(&mut self, actor: Entity, action: impl IntoAction, config: AddConfig) {
        self.add(AddAction {
            actor,
            config,
            action: action.into_boxed(),
        });
    }
}

impl StopActionExt for Commands<'_, '_> {
    fn stop_action(&mut self, actor: Entity) {
        self.add(StopAction { actor });
    }
}

impl NextActionExt for Commands<'_, '_> {
    fn next_action(&mut self, actor: Entity) {
        self.add(NextAction { actor });
    }
}

impl ClearActionsExt for Commands<'_, '_> {
    fn clear_actions(&mut self, actor: Entity) {
        self.add(ClearActions { actor });
    }
}

//
// Action builder
//

pub trait ActionBuilderCommandsExt<'w, 's, 'c> {
    fn action_builder(
        &'c mut self,
        actor: Entity,
        config: AddConfig,
    ) -> ActionBuilderCommands<'w, 's, 'c>;
}

impl<'w, 's, 'c> ActionBuilderCommandsExt<'w, 's, 'c> for Commands<'w, 's> {
    fn action_builder(
        &'c mut self,
        actor: Entity,
        config: AddConfig,
    ) -> ActionBuilderCommands<'w, 's, 'c> {
        ActionBuilderCommands {
            actor,
            config,
            actions: Vec::default(),
            commands: self,
        }
    }
}

pub struct ActionBuilderCommands<'w, 's, 'c> {
    actor: Entity,
    config: AddConfig,
    actions: Vec<Box<dyn Action>>,
    commands: &'c mut Commands<'w, 's>,
}

impl<'w, 's, 'c> ActionBuilderCommands<'w, 's, 'c> {
    pub fn add(mut self, action: impl IntoAction) -> Self {
        self.actions.push(action.into_boxed());
        self
    }

    pub fn reverse(mut self) -> Self {
        self.actions.reverse();
        self
    }

    pub fn submit(self) {
        self.commands.add(SubmitActions {
            actor: self.actor,
            config: self.config,
            actions: self.actions,
        });
    }
}

struct SubmitActions {
    actor: Entity,
    config: AddConfig,
    actions: Vec<Box<dyn Action>>,
}

impl Command for SubmitActions {
    fn write(self, world: &mut World) {
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, world);

        let actor = self.actor;
        let config = self.config;

        for action in self.actions {
            commands.add(AddAction {
                actor,
                config,
                action,
            });
        }

        command_queue.apply(world);
    }
}
