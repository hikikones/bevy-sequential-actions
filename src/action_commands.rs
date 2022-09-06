use bevy_ecs::prelude::*;

use crate::*;

/// Commands for modifying actions inside the [`Action`] trait.
#[derive(Default)]
pub struct ActionCommands(Vec<Box<dyn FnOnce(&mut World)>>);

impl ActionCommands {
    fn push<F>(&mut self, f: F)
    where
        F: FnOnce(&mut World) + 'static,
    {
        self.0.push(Box::new(f));
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

// pub struct MyActionCommands(Vec<Box<dyn FnOnce(&mut World)>>);

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
        self.commands.push(move |w| {
            w.actions(self.entity).config(self.config).add(action);
        });
        self
    }

    fn add_many<T>(self, actions: T) -> Self
    where
        T: BoxedActionIter,
    {
        self.commands.push(move |w| {
            w.actions(self.entity).config(self.config).add_many(actions);
        });
        self
    }

    fn next(self) -> Self {
        self.commands.push(move |w| {
            w.actions(self.entity).config(self.config).next();
        });
        self
    }

    fn finish(self) -> Self {
        self.commands.push(move |w| {
            w.actions(self.entity).config(self.config).finish();
        });
        self
    }

    fn pause(self) -> Self {
        self.commands.push(move |w| {
            w.actions(self.entity).config(self.config).pause();
        });
        self
    }

    fn stop(self, reason: StopReason) -> Self {
        self.commands.push(move |w| {
            w.actions(self.entity).config(self.config).stop(reason);
        });
        self
    }

    fn skip(self) -> Self {
        self.commands.push(move |w| {
            w.actions(self.entity).config(self.config).skip();
        });
        self
    }

    fn clear(self) -> Self {
        self.commands.push(move |w| {
            w.actions(self.entity).config(self.config).clear();
        });
        self
    }
}

// enum ActionCommand {
//     Add(Entity, AddConfig, BoxedAction),
//     AddMany(Entity, AddConfig, Box<dyn BoxedActionIter>),
//     Next(Entity),
//     Finish(Entity),
//     Pause(Entity),
//     Stop(Entity, StopReason),
//     Skip(Entity),
//     Clear(Entity),
//     Custom(Box<dyn FnOnce(&mut World)>),
// }

impl ActionCommands {
    pub(super) fn apply(self, world: &mut World) {
        for cmd in self.0 {
            cmd(world);
        }
        // for cmd in self.0 {
        //     match cmd {
        //         ActionCommand::Add(entity, config, action) => {
        //             world.actions(entity).config(config).add(action);
        //         }
        //         ActionCommand::AddMany(entity, config, actions) => {
        //             world.actions(entity).config(config).add_many(actions);
        //         }
        //         ActionCommand::Next(entity) => {
        //             world.actions(entity).next();
        //         }
        //         ActionCommand::Finish(entity) => {
        //             world.actions(entity).finish();
        //         }
        //         ActionCommand::Pause(entity) => {
        //             world.actions(entity).pause();
        //         }
        //         ActionCommand::Stop(entity, reason) => {
        //             world.actions(entity).stop(reason);
        //         }
        //         ActionCommand::Skip(entity) => {
        //             world.actions(entity).skip();
        //         }
        //         ActionCommand::Clear(entity) => {
        //             world.actions(entity).clear();
        //         }
        //         ActionCommand::Custom(f) => {
        //             f(world);
        //         }
        //     }
        // }
    }
}
