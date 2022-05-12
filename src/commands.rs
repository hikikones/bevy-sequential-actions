use bevy_ecs::{prelude::*, system::Command};

use crate::{world::EntityWorldActionsExt, *};

/// Extension method on [`Commands`] for modifying actions.
pub trait EntityCommandsActionsExt<'w, 's> {
    /// Returns an [`EntityCommandsActions`] for the requested [`Entity`].
    fn action(&mut self, entity: Entity) -> EntityCommandsActions<'w, 's, '_>;
}

impl<'w, 's> EntityCommandsActionsExt<'w, 's> for Commands<'w, 's> {
    fn action(&mut self, entity: Entity) -> EntityCommandsActions<'w, 's, '_> {
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

impl<'w, 's> ModifyActionsExt for EntityCommandsActions<'w, 's, '_> {
    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add(self, action: impl IntoAction) -> Self {
        self.commands.add(AddAction {
            entity: self.entity,
            config: self.config,
            action: action.into_boxed(),
        });
        self
    }

    fn next(self) -> Self {
        self.commands.add(NextAction {
            entity: self.entity,
        });
        self
    }

    fn stop(self) -> Self {
        self.commands.add(StopAction {
            entity: self.entity,
        });
        self
    }

    fn clear(self) -> Self {
        self.commands.add(ClearActions {
            entity: self.entity,
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
            self.commands.add(AddAction {
                entity: self.entity,
                config,
                action,
            });
        }
        self
    }
}

struct AddAction {
    entity: Entity,
    config: AddConfig,
    action: Box<dyn Action>,
}

struct NextAction {
    entity: Entity,
}

struct StopAction {
    entity: Entity,
}

struct ClearActions {
    entity: Entity,
}

impl Command for AddAction {
    fn write(self, world: &mut World) {
        world
            .action(self.entity)
            .config(self.config)
            .add(self.action);
    }
}

impl Command for NextAction {
    fn write(self, world: &mut World) {
        world.action(self.entity).next();
    }
}

impl Command for StopAction {
    fn write(self, world: &mut World) {
        world.action(self.entity).stop();
    }
}

impl Command for ClearActions {
    fn write(self, world: &mut World) {
        world.action(self.entity).clear();
    }
}
