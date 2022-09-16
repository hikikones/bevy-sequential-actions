use crate::*;

/// Commands for modifying actions inside the [`Action`] trait.
#[derive(Default)]
#[allow(clippy::type_complexity)]
pub struct ActionCommands(Vec<Box<dyn FnOnce(&mut World)>>);

impl ActionCommands {
    fn push<F>(&mut self, f: F)
    where
        F: FnOnce(&mut World) + 'static,
    {
        self.0.push(Box::new(f));
    }

    pub(super) fn apply(self, world: &mut World) {
        for cmd in self.0 {
            cmd(world);
        }
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

impl AgentActions<'_> {
    /// Mutate [`World`] with `f` after [`Action::on_start`] has been called.
    /// Used for modifying actions in a deferred way using [`World`] inside the [`Action`] trait.
    pub fn custom<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut World) + 'static,
    {
        self.commands.push(f);
        self
    }
}

impl ModifyActions for AgentActions<'_> {
    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add(self, action: impl IntoBoxedAction) -> Self {
        self.commands.push(move |world| {
            world.add_action(self.agent, self.config, action);
        });
        self
    }

    fn add_many(self, mode: ExecutionMode, actions: impl BoxedActionIter) -> Self {
        self.commands.push(move |world| {
            world.add_actions(self.agent, self.config, mode, actions);
        });
        self
    }

    fn next(self) -> Self {
        self.commands.push(move |world| {
            world.next_action(self.agent);
        });
        self
    }

    fn cancel(self) -> Self {
        self.commands.push(move |world| {
            world.cancel_action(self.agent);
        });
        self
    }

    fn pause(self) -> Self {
        self.commands.push(move |world| {
            world.pause_action(self.agent);
        });
        self
    }

    fn skip(self) -> Self {
        self.commands.push(move |world| {
            world.skip_action(self.agent);
        });
        self
    }

    fn clear(self) -> Self {
        self.commands.push(move |world| {
            world.clear_actions(self.agent);
        });
        self
    }
}
