use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::{FromLookExt, MoveTowardsTransformExt};

use super::IntoValue;

pub struct MoveActionPlugin;

impl Plugin for MoveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement).add_system(rotation);
    }
}

pub struct MoveAction<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    config: MoveConfig<V, F>,
    bundle: Option<MoveBundle>,
}

pub struct MoveConfig<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    pub target: V,
    pub speed: F,
    pub rotate: bool,
}

impl<V, F> MoveAction<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    pub fn new(config: MoveConfig<V, F>) -> Self {
        Self {
            config,
            bundle: None,
        }
    }
}

impl<V, F> Action for MoveAction<V, F>
where
    V: IntoValue<Vec3>,
    F: IntoValue<f32>,
{
    fn on_start(&mut self, agent: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let move_bundle = self.bundle.take().unwrap_or(MoveBundle {
            target: Target(self.config.target.value()),
            speed: Speed(self.config.speed.value()),
        });

        let mut agent = world.entity_mut(agent);

        if self.config.rotate {
            let start = agent.get::<Transform>().unwrap().translation;
            let dir = (move_bundle.target.0 - start).normalize_or_zero();
            if dir != Vec3::ZERO {
                agent.insert(Rotate(Quat::from_look(dir, Vec3::Y)));
            }
        }

        agent.insert(move_bundle);
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        let mut agent = world.entity_mut(agent);
        let bundle = agent.remove::<MoveBundle>();

        if self.config.rotate {
            agent.remove::<Rotate>();
        }

        if let StopReason::Paused = reason {
            self.bundle = bundle;
        }
    }
}

#[derive(Bundle)]
struct MoveBundle {
    target: Target,
    speed: Speed,
}

#[derive(Component)]
struct Target(Vec3);

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct Rotate(Quat);

fn movement(
    mut move_q: Query<(&mut Transform, &Target, &Speed, &mut ActionFinished)>,
    time: Res<Time>,
) {
    for (mut transform, target, speed, mut finished) in move_q.iter_mut() {
        transform.move_towards(target.0, speed.0 * time.delta_seconds());

        if transform.translation == target.0 {
            finished.confirm_and_reset();
        }
    }
}

fn rotation(mut rot_q: Query<(&mut Transform, &Speed, &Rotate)>, time: Res<Time>) {
    for (mut transform, speed, rotate) in rot_q.iter_mut() {
        transform.rotation = Quat::slerp(
            transform.rotation,
            rotate.0,
            speed.0 * 2.0 * time.delta_seconds(),
        );
    }
}
