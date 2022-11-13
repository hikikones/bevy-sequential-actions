use bevy::prelude::*;
use bevy_sequential_actions::*;

use super::IntoValue;

pub(super) struct LerpActionPlugin;

impl Plugin for LerpActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(lerp);
    }
}

pub struct LerpAction<F>
where
    F: IntoValue<f32>,
{
    config: LerpConfig<F>,
    entity: Option<Entity>,
    bundle: Option<LerpBundle>,
}

pub struct LerpConfig<F>
where
    F: IntoValue<f32>,
{
    pub target: Entity,
    pub lerp_type: LerpType,
    pub duration: F,
}

impl<F> LerpAction<F>
where
    F: IntoValue<f32>,
{
    pub fn new(config: LerpConfig<F>) -> Self {
        Self {
            config,
            entity: None,
            bundle: None,
        }
    }
}

pub enum LerpType {
    Position(Vec3),
    Rotation(Quat),
    Transform(Transform),
}

impl<F> Action for LerpAction<F>
where
    F: IntoValue<f32>,
{
    fn on_start(&mut self, agent: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let lerp_bundle = self.bundle.take().unwrap_or_else(|| {
            let lerp_type = match self.config.lerp_type {
                LerpType::Position(target) => {
                    let start = world
                        .get::<Transform>(self.config.target)
                        .unwrap()
                        .translation;
                    Lerp::Position(start, target)
                }
                LerpType::Rotation(target) => {
                    let start = world.get::<Transform>(self.config.target).unwrap().rotation;
                    Lerp::Rotation(start, target)
                }
                LerpType::Transform(target) => {
                    let start = world.get::<Transform>(self.config.target).unwrap();
                    Lerp::Transform(start.clone(), target)
                }
            };

            LerpBundle {
                lerp: lerp_type,
                target: LerpTarget(self.config.target),
                agent: LerpAgent(agent),
                timer: LerpTimer(Timer::from_seconds(
                    self.config.duration.value(),
                    TimerMode::Once,
                )),
            }
        });

        self.entity = Some(world.spawn(lerp_bundle).id());
    }

    fn on_stop(&mut self, _agent: Entity, world: &mut World, reason: StopReason) {
        let entity = self.entity.unwrap();

        if let StopReason::Paused = reason {
            self.bundle = world.entity_mut(entity).remove::<LerpBundle>();
        }

        world.despawn(entity);
    }
}

#[derive(Bundle)]
struct LerpBundle {
    lerp: Lerp,
    target: LerpTarget,
    agent: LerpAgent,
    timer: LerpTimer,
}

#[derive(Component)]
enum Lerp {
    Position(Vec3, Vec3),
    Rotation(Quat, Quat),
    Transform(Transform, Transform),
}

#[derive(Component)]
struct LerpTarget(Entity);

#[derive(Component)]
struct LerpAgent(Entity);

#[derive(Component)]
struct LerpTimer(Timer);

fn lerp(
    mut lerp_q: Query<(&mut LerpTimer, &LerpTarget, &Lerp, &LerpAgent)>,
    mut transform_q: Query<&mut Transform>,
    mut finished_q: Query<&mut ActionFinished>,
    time: Res<Time>,
) {
    for (mut timer, target, lerp, agent) in lerp_q.iter_mut() {
        if let Ok(mut transform) = transform_q.get_mut(target.0) {
            timer.0.tick(time.delta());

            let t = timer.0.percent();
            let smoothstep = 3.0 * t * t - 2.0 * t * t * t;

            match lerp {
                Lerp::Position(start, end) => {
                    transform.translation = start.lerp(*end, smoothstep);
                }
                Lerp::Rotation(start, end) => {
                    transform.rotation = start.slerp(*end, smoothstep);
                }
                Lerp::Transform(start, end) => {
                    transform.translation = start.translation.lerp(end.translation, smoothstep);
                    transform.rotation = start.rotation.slerp(end.rotation, smoothstep);
                }
            }

            if timer.0.finished() {
                finished_q.get_mut(agent.0).unwrap().confirm_and_reset();
            }
        } else {
            finished_q.get_mut(agent.0).unwrap().confirm_and_reset();
        }
    }
}
