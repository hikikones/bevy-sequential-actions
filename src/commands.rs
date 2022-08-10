use bevy_ecs::prelude::*;

use crate::*;

impl<'w: 'a, 's: 'a, 'a> ActionsProxy<'a> for Commands<'w, 's> {
    type Modifier = EntityCommandsActions<'w, 's, 'a>;

    fn actions(&'a mut self, entity: Entity) -> EntityCommandsActions<'w, 's, 'a> {
        EntityCommandsActions {
            entity,
            config: AddConfig::default(),
            commands: self,
        }
    }
}

/// Modify actions using [`Commands`].
pub struct EntityCommandsActions<'w, 's, 'a> {
    entity: Entity,
    config: AddConfig,
    commands: &'a mut Commands<'w, 's>,
}

impl<'w, 's, 'a> ModifyActions for EntityCommandsActions<'w, 's, 'a> {
    type Builder = ActionCommandsBuilder<'w, 's, 'a>;

    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add<T: IntoAction>(self, action: T) -> Self {
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

    fn builder(self) -> Self::Builder {
        ActionCommandsBuilder {
            config: AddConfig::default(),
            actions: Vec::new(),
            modifier: self,
        }
    }
}

/// Build a list of actions using [`Commands`].
pub struct ActionCommandsBuilder<'w, 's, 'a> {
    config: AddConfig,
    actions: Vec<(Box<dyn Action>, AddConfig)>,
    modifier: EntityCommandsActions<'w, 's, 'a>,
}

impl<'w, 's, 'a> ActionBuilder for ActionCommandsBuilder<'w, 's, 'a> {
    type Modifier = EntityCommandsActions<'w, 's, 'a>;

    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn push<T: IntoAction>(mut self, action: T) -> Self {
        self.actions.push((action.into_boxed(), self.config));
        self
    }

    fn reverse(mut self) -> Self {
        self.actions.reverse();
        self
    }

    fn submit(self) -> Self::Modifier {
        for (action, config) in self.actions {
            self.modifier.commands.add(move |world: &mut World| {
                world
                    .actions(self.modifier.entity)
                    .config(config)
                    .add(action);
            });
        }

        self.modifier
    }
}
