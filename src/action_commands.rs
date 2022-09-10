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

    fn add<T>(self, action: T) -> Self
    where
        T: IntoBoxedAction,
    {
        self.commands.push(move |world| {
            world.actions(self.entity).config(self.config).add(action);
        });
        self
    }

    fn add_many<T>(self, mode: ExecutionMode, actions: T) -> Self
    where
        T: BoxedActionIter,
    {
        self.commands.push(move |world| {
            world
                .actions(self.entity)
                .config(self.config)
                .add_many(mode, actions);
        });
        self
    }

    fn next(self) -> Self {
        self.commands.push(move |world| {
            world.actions(self.entity).config(self.config).next();
        });
        self
    }

    fn pause(self) -> Self {
        self.commands.push(move |world| {
            world.actions(self.entity).config(self.config).pause();
        });
        self
    }

    fn skip(self) -> Self {
        self.commands.push(move |world| {
            world.actions(self.entity).config(self.config).skip();
        });
        self
    }

    fn clear(self) -> Self {
        self.commands.push(move |world| {
            world.actions(self.entity).config(self.config).clear();
        });
        self
    }
}
