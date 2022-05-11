use bevy_ecs::{prelude::*, system::Command};

use crate::{world::ActionsWorldExt, *};

pub trait EntityActionsExt<'w, 's> {
    fn action<'a>(&'a mut self, entity: Entity) -> EntityActions<'w, 's, 'a>;
}

impl<'w, 's> EntityActionsExt<'w, 's> for Commands<'w, 's> {
    fn action<'a>(&'a mut self, entity: Entity) -> EntityActions<'w, 's, 'a> {
        EntityActions {
            entity,
            config: AddConfig::default(),
            actions: Vec::new(),
            commands: self,
        }
    }
}

pub struct EntityActions<'w, 's, 'a> {
    entity: Entity,
    config: AddConfig,
    actions: Vec<(Box<dyn Action>, AddConfig)>,
    commands: &'a mut Commands<'w, 's>,
}

impl<'w, 's, 'a> ActionsExt for EntityActions<'w, 's, 'a> {
    fn config(mut self, cfg: AddConfig) -> Self {
        self.config = cfg;
        self
    }

    fn add(self, action: impl IntoAction) -> Self {
        self.commands.add(AddAction {
            actor: self.entity,
            config: self.config,
            action: action.into_boxed(),
        });
        self
    }

    fn next(self) -> Self {
        self.commands.add(NextAction { actor: self.entity });
        self
    }

    fn stop(self) -> Self {
        self.commands.add(StopAction { actor: self.entity });
        self
    }

    fn clear(self) -> Self {
        self.commands.add(ClearActions { actor: self.entity });
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
                actor: self.entity,
                config,
                action,
            });
        }
        self
    }
}

struct AddAction {
    actor: Entity,
    config: AddConfig,
    action: Box<dyn Action>,
}

struct StopAction {
    actor: Entity,
}

struct NextAction {
    actor: Entity,
}

struct ClearActions {
    actor: Entity,
}

impl Command for AddAction {
    fn write(self, world: &mut World) {
        world.add_action(self.actor, self.action, self.config);
    }
}

impl Command for StopAction {
    fn write(self, world: &mut World) {
        world.stop_action(self.actor);
    }
}

impl Command for NextAction {
    fn write(self, world: &mut World) {
        world.next_action(self.actor);
    }
}

impl Command for ClearActions {
    fn write(self, world: &mut World) {
        world.clear_actions(self.actor);
    }
}
