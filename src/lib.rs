use std::collections::VecDeque;

use bevy_ecs::prelude::*;

mod action_commands;
mod commands;
mod traits;

pub mod world;

pub use action_commands::*;
pub use commands::*;
pub use traits::*;

pub trait Action: Send + Sync {
    fn add(&mut self, actor: Entity, world: &mut World, commands: &mut ActionCommands);
    fn remove(&mut self, actor: Entity, world: &mut World);
    fn stop(&mut self, actor: Entity, world: &mut World);
}

#[derive(Default, Bundle)]
pub struct ActionsBundle {
    queue: ActionQueue,
    current: CurrentAction,
}

#[derive(Clone, Copy)]
pub enum AddOrder {
    Back,
    Front,
}

#[derive(Clone, Copy)]
pub struct AddConfig {
    pub order: AddOrder,
    pub start: bool,
    pub repeat: bool,
}

impl Default for AddConfig {
    fn default() -> Self {
        Self {
            order: AddOrder::Back,
            start: true,
            repeat: false,
        }
    }
}

#[derive(Default, Component)]
struct ActionQueue(VecDeque<(Box<dyn Action>, ActionConfig)>);

#[derive(Default, Component)]
struct CurrentAction(Option<(Box<dyn Action>, ActionConfig)>);

struct ActionConfig {
    repeat: bool,
}

impl Into<ActionConfig> for AddConfig {
    fn into(self) -> ActionConfig {
        ActionConfig {
            repeat: self.repeat,
        }
    }
}
