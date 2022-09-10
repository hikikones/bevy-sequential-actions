use bevy_ecs::prelude::*;

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

impl EntityActions<'_> {
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

impl ModifyActions for EntityActions<'_> {
    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add(self, action: impl IntoBoxedAction) -> Self {
        self.commands.push(move |world| {
            world.add_action(self.entity, self.config, action);
        });
        self
    }

    fn add_many(self, mode: ExecutionMode, actions: impl BoxedActionIter) -> Self {
        self.commands.push(move |world| {
            world.add_actions(self.entity, self.config, mode, actions);
        });
        self
    }

    fn next(self) -> Self {
        self.commands.push(move |world| {
            world.next_action(self.entity);
        });
        self
    }

    fn cancel(self) -> Self {
        self.commands.push(move |world| {
            world.cancel_action(self.entity);
        });
        self
    }

    fn pause(self) -> Self {
        self.commands.push(move |world| {
            world.pause_action(self.entity);
        });
        self
    }

    fn skip(self) -> Self {
        self.commands.push(move |world| {
            world.skip_action(self.entity);
        });
        self
    }

    fn clear(self) -> Self {
        self.commands.push(move |world| {
            world.clear_actions(self.entity);
        });
        self
    }
}
