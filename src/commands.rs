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
    fn config(&mut self, config: AddConfig) -> &mut Self {
        self.config = config;
        self
    }

    fn add(&mut self, action: impl IntoBoxedAction) -> &mut Self {
        let agent = self.agent;
        let config = self.config;
        self.commands.add(move |world: &mut World| {
            world.add_action(agent, config, action);
        });
        self
    }

    fn add_many(&mut self, mode: ExecutionMode, actions: impl BoxedActionIter) -> &mut Self {
        let agent = self.agent;
        let config = self.config;
        self.commands.add(move |world: &mut World| {
            world.add_actions(agent, config, mode, actions);
        });
        self
    }

    fn next(&mut self) -> &mut Self {
        let agent = self.agent;
        self.commands.add(move |world: &mut World| {
            world.next_action(agent);
        });
        self
    }

    fn cancel(&mut self) -> &mut Self {
        let agent = self.agent;
        self.commands.add(move |world: &mut World| {
            world.cancel_action(agent);
        });
        self
    }

    fn pause(&mut self) -> &mut Self {
        let agent = self.agent;
        self.commands.add(move |world: &mut World| {
            world.pause_action(agent);
        });
        self
    }

    fn skip(&mut self) -> &mut Self {
        let agent = self.agent;
        self.commands.add(move |world: &mut World| {
            world.skip_action(agent);
        });
        self
    }

    fn clear(&mut self) -> &mut Self {
        let agent = self.agent;
        self.commands.add(move |world: &mut World| {
            world.clear_actions(agent);
        });
        self
    }
}
