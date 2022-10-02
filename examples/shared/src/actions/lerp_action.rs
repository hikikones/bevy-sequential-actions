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
    executor: Option<Entity>,
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
            executor: None,
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
    fn on_start(&mut self, state: &mut WorldState, _commands: &mut ActionCommands) {
        let lerp_bundle = self.bundle.take().unwrap_or_else(|| {
            let lerp_type = match self.config.lerp_type {
                LerpType::Position(target) => {
                    let start = state
                        .world
                        .get::<Transform>(self.config.target)
                        .unwrap()
                        .translation;
                    Lerp::Position(start, target)
                }
                LerpType::Rotation(target) => {
                    let start = state
                        .world
                        .get::<Transform>(self.config.target)
                        .unwrap()
                        .rotation;
                    Lerp::Rotation(start, target)
                }
                LerpType::Transform(target) => {
                    let start = state.world.get::<Transform>(self.config.target).unwrap();
                    Lerp::Transform(start.clone(), target)
                }
            };

            LerpBundle {
                lerp: lerp_type,
                target: LerpTarget(self.config.target),
                timer: LerpTimer(Timer::from_seconds(self.config.duration.value(), false)),
                agent: Agent(state.agent),
            }
        });

        self.executor = Some(state.world.spawn().insert_bundle(lerp_bundle).id());
    }

    fn on_stop(&mut self, state: &mut WorldState, reason: StopReason) {
        let executor = self.executor.unwrap();

        if let StopReason::Paused = reason {
            self.bundle = state
                .world
                .entity_mut(executor)
                .remove_bundle::<LerpBundle>();
        }

        state.world.despawn(executor);
    }
}

#[derive(Bundle)]
struct LerpBundle {
    lerp: Lerp,
    target: LerpTarget,
    timer: LerpTimer,
    agent: Agent,
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
struct LerpTimer(Timer);

#[derive(Component)]
struct Agent(Entity);

fn lerp(
    mut lerp_q: Query<(&mut LerpTimer, &LerpTarget, &Lerp, &Agent)>,
    mut transform_q: Query<&mut Transform>,
    mut finished_q: Query<&mut ActionFinished>,
    time: Res<Time>,
) {
    for (mut timer, target, lerp, agent) in lerp_q.iter_mut() {
        if let Ok(mut transform) = transform_q.get_mut(target.0) {
            timer.0.tick(time.delta());

            let t = timer.0.percent();
            let smoothstep = 3.0 * t * t - 2.0 * t * t * t;

            match *lerp {
                Lerp::Position(start, end) => {
                    transform.translation = start.lerp(end, smoothstep);
                }
                Lerp::Rotation(start, end) => {
                    transform.rotation = start.slerp(end, smoothstep);
                }
                Lerp::Transform(start, end) => {
                    transform.translation = start.translation.lerp(end.translation, smoothstep);
                    transform.rotation = start.rotation.slerp(end.rotation, smoothstep);
                }
            }

            if timer.0.finished() {
                finished_q.get_mut(agent.0).unwrap().confirm();
            }
        } else {
            finished_q.get_mut(agent.0).unwrap().confirm();
        }
    }
}
