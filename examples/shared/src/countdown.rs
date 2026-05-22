use bevy::{
    app::{Plugin, Update},
    ecs::{component::Component, entity::Entity, system::Query, world::World},
};
use bevy_sequential_actions::*;

pub(crate) struct CountdownActionPlugin;

impl Plugin for CountdownActionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, countdown);
    }
}

pub struct CountdownAction {
    count: u32,
    remaining: Option<u32>,
}

impl CountdownAction {
    pub const fn new(count: u32) -> Self {
        Self {
            count,
            remaining: None,
        }
    }
}

impl Action for CountdownAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        let current_count = world.get::<Countdown>(agent).unwrap().0;

        // Determine if countdown has reached zero.
        current_count == 0
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        // Take remaining count (if paused), or use full count.
        let count = self.remaining.take().unwrap_or(self.count);

        // Run the countdown system on the agent.
        world.entity_mut(agent).insert(Countdown(count));

        // Is action already finished?
        self.is_finished(agent, world)
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        // Do nothing if agent has been despawned.
        let Some(agent) = agent else { return };

        // Take the countdown component from the agent.
        let countdown = world.entity_mut(agent).take::<Countdown>();

        // Store remaining count when paused.
        if reason == StopReason::Paused {
            self.remaining = countdown.unwrap().0.into();
        }
    }
}

#[derive(Component)]
pub struct Countdown(u32);

impl Countdown {
    pub const fn current_count(&self) -> u32 {
        self.0
    }
}

fn countdown(mut countdown_q: Query<&mut Countdown>) {
    for mut countdown in &mut countdown_q {
        countdown.0 = countdown.0.saturating_sub(1);
        println!("Countdown: {}", countdown.0);
    }
}
