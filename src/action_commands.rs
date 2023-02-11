use crate::*;

/// Commands for modifying actions inside the [`Action`] trait.
#[allow(clippy::type_complexity)]
pub struct ActionCommands(Vec<Box<dyn FnOnce(&mut World)>>);

impl ActionCommands {
    pub(super) fn new() -> Self {
        Self(Vec::new())
    }

    pub(super) fn apply(self, world: &mut World) {
        for cmd in self.0 {
            cmd(world);
        }
    }

    /// Adds a custom command to the command queue.
    /// Used for modifying actions in a deferred way using [`World`] inside the [`Action`] trait.
    pub fn add(&mut self, f: impl FnOnce(&mut World) + 'static) -> &mut Self {
        self.0.push(Box::new(f));
        self
    }
}

impl<'a> ActionsProxy<'a> for ActionCommands {
    type Modifier = AgentActions<'a>;

    fn actions(&'a mut self, agent: Entity) -> AgentActions<'a> {
        AgentActions {
            agent,
            config: AddConfig::default(),
            commands: self,
        }
    }
}

/// Modify actions using [`ActionCommands`].
pub struct AgentActions<'a> {
    agent: Entity,
    config: AddConfig,
    commands: &'a mut ActionCommands,
}

impl ModifyActions for AgentActions<'_> {
    fn config(&mut self, config: AddConfig) -> &mut Self {
        self.config = config;
        self
    }

    fn add(&mut self, action: impl Into<ActionType>) -> &mut Self {
        let agent = self.agent;
        let config = self.config;
        let action = action.into();

        self.commands.add(move |world| {
            world.add_action(agent, config, action);
        });

        self
    }

    fn add_linked(&mut self, f: impl FnOnce(&mut LinkedActionsBuilder)) -> &mut Self {
        let agent = self.agent;
        let config = self.config;

        let mut actions = LinkedActionsBuilder::new();
        f(&mut actions);

        self.commands.add(move |world: &mut World| {
            world.add_linked_actions(agent, config, actions);
        });

        self
    }

    fn next(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world| {
            world.next_action(agent);
        });

        self
    }

    fn cancel(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world| {
            world.cancel_action(agent);
        });

        self
    }

    fn pause(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world| {
            world.pause_action(agent);
        });

        self
    }

    fn skip(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world| {
            world.skip_action(agent);
        });

        self
    }

    fn clear(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world| {
            world.clear_actions(agent);
        });

        self
    }
}
