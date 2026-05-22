use bevy::prelude::*;
use bevy_sequential_actions::*;

pub(crate) struct WaitActionPlugin;

impl Plugin for WaitActionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, wait);
    }
}

pub struct WaitAction {
    duration: f32,          // Seconds
    remaining: Option<f32>, // None
}

impl WaitAction {
    pub const fn new(duration: f32) -> Self {
        Self {
            duration,
            remaining: None,
        }
    }
}

impl Action for WaitAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        // Determine if wait timer has reached zero.
        world.get::<WaitTimer>(agent).unwrap().0 <= 0.0
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        // Take remaining time (if paused), or use full duration.
        let duration = self.remaining.take().unwrap_or(self.duration);

        // Run the wait timer system on the agent.
        world.entity_mut(agent).insert(WaitTimer(duration));

        // Is action already finished?
        // Returning true here will immediately advance the action queue.
        self.is_finished(agent, world)
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        // Do nothing if agent has been despawned.
        let Some(agent) = agent else { return };

        // Take the wait timer component from the agent.
        let wait_timer = world.entity_mut(agent).take::<WaitTimer>();

        // Store remaining time when paused.
        if reason == StopReason::Paused {
            self.remaining = Some(wait_timer.unwrap().0);
        }
    }
}

#[derive(Component)]
pub struct WaitTimer(f32);

impl WaitTimer {
    pub const fn remaining_secs(&self) -> f32 {
        self.0
    }
}

fn wait(mut wait_timer_q: Query<&mut WaitTimer>, time: Res<Time>) {
    for mut wait_timer in &mut wait_timer_q {
        wait_timer.0 -= time.delta_secs();
        println!("Wait seconds: {}", wait_timer.0);
    }
}
