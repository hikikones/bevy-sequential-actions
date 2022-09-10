use bevy_ecs::prelude::*;

use crate::*;

impl<'c, 'w: 'c, 's: 'c> ActionsProxy<'c> for Commands<'w, 's> {
    type Modifier = EntityCommandsActions<'c, 'w, 's>;

    fn actions(&'c mut self, entity: Entity) -> EntityCommandsActions<'c, 'w, 's> {
        EntityCommandsActions {
            entity,
            config: AddConfig::default(),
            commands: self,
        }
    }
}

/// Modify actions using [`Commands`].
pub struct EntityCommandsActions<'c, 'w, 's> {
    entity: Entity,
    config: AddConfig,
    commands: &'c mut Commands<'w, 's>,
}

impl ModifyActions for EntityCommandsActions<'_, '_, '_> {
    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add<T>(self, action: T) -> Self
    where
        T: IntoBoxedAction,
    {
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).config(self.config).add(action);
        });
        self
    }

    fn add_many<T>(self, mode: ExecutionMode, actions: T) -> Self
    where
        T: BoxedActionIter,
    {
        self.commands.add(move |world: &mut World| {
            world
                .actions(self.entity)
                .config(self.config)
                .add_many(mode, actions);
        });
        self
    }

    fn next(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).next();
        });
        self
    }

    fn cancel(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).cancel();
        });
        self
    }

    fn pause(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).pause();
        });
        self
    }

    fn skip(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).skip();
        });
        self
    }

    fn clear(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).clear();
        });
        self
    }
}
