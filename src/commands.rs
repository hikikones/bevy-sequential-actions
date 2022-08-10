use bevy_ecs::prelude::*;

use crate::*;

impl<'w: 'a, 's: 'a, 'a> ActionsProxy<'a> for Commands<'w, 's> {
    type Modifier = EntityCommandsActions<'w, 's, 'a>;

    fn actions(&'a mut self, entity: Entity) -> EntityCommandsActions<'w, 's, 'a> {
        EntityCommandsActions {
            entity,
            config: AddConfig::default(),
            actions: Vec::new(),
            commands: self,
        }
    }
}

/// Modify actions using [`Commands`].
pub struct EntityCommandsActions<'w, 's, 'a> {
    entity: Entity,
    config: AddConfig,
    actions: Vec<(Box<dyn Action>, AddConfig)>,
    commands: &'a mut Commands<'w, 's>,
}

impl<'w, 's> ModifyActions for EntityCommandsActions<'w, 's, '_> {
    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add(self, action: impl IntoAction) -> Self {
        let action = action.into_boxed();
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).config(self.config).add(action);
        });
        self
    }

    fn next(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).next();
        });
        self
    }

    fn finish(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).finish();
        });
        self
    }

    fn pause(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).pause();
        });
        self
    }

    fn stop(self, reason: StopReason) -> Self {
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).stop(reason);
        });
        self
    }

    fn clear(self) -> Self {
        self.commands.add(move |world: &mut World| {
            world.actions(self.entity).clear();
        });
        self
    }

    fn push(mut self, action: impl IntoAction) -> Self {
        self.actions.push((action.into_boxed(), self.config));
        self
    }

    fn reverse(mut self) -> Self {
        self.actions.reverse();
        self
    }

    fn submit(mut self) -> Self {
        for (action, config) in self.actions.drain(..) {
            self.commands.add(move |world: &mut World| {
                world.actions(self.entity).config(config).add(action);
            });
        }
        self
    }
}

trait Builder {
    type Modifier: ModifyActions;

    fn submit(self) -> Self::Modifier;
}

pub struct CommandsActionsBuilder<'w, 's, 'a> {
    entity: Entity,
    config: AddConfig,
    actions: Vec<(Box<dyn Action>, AddConfig)>,
    commands: &'a mut Commands<'w, 's>,
}

impl<'w, 's, 'a> Builder for CommandsActionsBuilder<'w, 's, 'a> {
    type Modifier = EntityCommandsActions<'a, 's, 'a>;

    fn submit(self) -> Self::Modifier {
        EntityCommandsActions {
            entity: todo!(),
            config: todo!(),
            actions: todo!(),
            commands: todo!(),
        }
    }
}
