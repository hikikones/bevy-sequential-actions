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

    fn add(self, action: impl IntoBoxedAction) -> Self {
        self.commands.add(move |world: &mut World| {
            world.add_action(self.entity, self.config, action);
        });
        self
    }

    fn add_many(self, mode: ExecutionMode, actions: impl BoxedActionIter) -> Self {
        self.commands.add(move |world: &mut World| {
            world.add_actions(self.entity, self.config, mode, actions);
        });
        self
    }

    fn next(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.next_action(self.entity);
        });
        self
    }

    fn cancel(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.cancel_action(self.entity);
        });
        self
    }

    fn pause(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.pause_action(self.entity);
        });
        self
    }

    fn skip(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.skip_action(self.entity);
        });
        self
    }

    fn clear(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.clear_actions(self.entity);
        });
        self
    }
}
