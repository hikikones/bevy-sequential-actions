use crate::*;

impl<'c, 'w: 'c, 's: 'c> ActionsProxy<'c> for Commands<'w, 's> {
    type Modifier = AgentCommandsActions<'c, 'w, 's>;

    fn actions(&'c mut self, agent: Entity) -> AgentCommandsActions<'c, 'w, 's> {
        AgentCommandsActions {
            agent,
            config: AddConfig::default(),
            commands: self,
        }
    }
}

/// Modify actions using [`Commands`].
pub struct AgentCommandsActions<'c, 'w, 's> {
    agent: Entity,
    config: AddConfig,
    commands: &'c mut Commands<'w, 's>,
}

impl ModifyActions for AgentCommandsActions<'_, '_, '_> {
    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add(self, action: impl IntoBoxedAction) -> Self {
        self.commands.add(move |world: &mut World| {
            world.add_action(self.agent, self.config, action);
        });
        self
    }

    fn add_many(self, mode: ExecutionMode, actions: impl BoxedActionIter) -> Self {
        self.commands.add(move |world: &mut World| {
            world.add_actions(self.agent, self.config, mode, actions);
        });
        self
    }

    fn next(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.next_action(self.agent);
        });
        self
    }

    fn cancel(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.cancel_action(self.agent);
        });
        self
    }

    fn pause(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.pause_action(self.agent);
        });
        self
    }

    fn skip(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.skip_action(self.agent);
        });
        self
    }

    fn clear(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.clear_actions(self.agent);
        });
        self
    }
}
