use bevy::prelude::*;
use bevy_sequential_actions::*;

use crate::extensions::{FromLookExt, MoveTowardsTransformExt};

use super::IntoValue;

pub struct MoveActionPlugin;

impl Plugin for MoveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement)
            .add_system(check_movement.after(movement))
            .add_system(rotation);
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

        let mut agent = world.entity_mut(id.agent());

        if self.config.rotate {
            let start = agent.get::<Transform>().unwrap().translation;
            let dir = (move_bundle.target.0 - start).normalize_or_zero();
            if dir != Vec3::ZERO {
                agent.insert(Rotate(Quat::from_look(dir, Vec3::Y)));
            }
        }

        agent.insert_bundle(move_bundle);
        world.entity_mut(id.status()).insert(MoveMarker);
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        let bundle = world.entity_mut(id.agent()).remove_bundle::<MoveBundle>();

        if self.config.rotate {
            world.entity_mut(id.agent()).remove::<Rotate>();
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

#[derive(Component)]
struct MoveMarker;

fn movement(mut move_q: Query<(&mut Transform, &Target, &Speed)>, time: Res<Time>) {
    for (mut transform, target, speed) in move_q.iter_mut() {
        transform.move_towards(target.0, speed.0 * time.delta_seconds());
    }
}

fn check_movement(
    mut check_q: Query<(&ActionAgent, &mut ActionFinished), With<MoveMarker>>,
    transform_q: Query<(&Transform, &Target)>,
) {
    for (agent, mut finished) in check_q.iter_mut() {
        let (transform, target) = transform_q.get(agent.id()).unwrap();
        finished.set(transform.translation == target.0);
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
