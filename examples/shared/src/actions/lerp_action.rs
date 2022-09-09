use bevy::prelude::*;
use bevy_sequential_actions::*;

use super::CHECK_ACTIONS_STAGE;

pub(super) struct LerpActionPlugin;

impl Plugin for LerpActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(lerp_system);
        // .add_system_to_stage(CHECK_ACTIONS_STAGE, check_lerp_status);
    }
}

pub struct LerpAction {
    target: Entity,
    lerp_type: LerpType,
    duration: f32,
    paused_bundle: Option<LerpBundle>,
}

impl LerpAction {
    pub fn new(target: Entity, lerp_type: LerpType, duration: f32) -> Self {
        Self {
            target,
            lerp_type,
            duration,
            paused_bundle: None,
        }
    }
}

pub enum LerpType {
    Position(Vec3),
    Rotation(Quat),
    Transform(Transform),
}

impl Action for LerpAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        if let Some(bundle) = self.paused_bundle.take() {
            world.entity_mut(entity).insert_bundle(bundle);
            return;
        }

        let lerp = match self.lerp_type {
            LerpType::Position(target) => {
                let start = world.get::<Transform>(self.target).unwrap().translation;
                Lerp::Position(start, target)
            }
            LerpType::Rotation(target) => {
                let start = world.get::<Transform>(self.target).unwrap().rotation;
                Lerp::Rotation(start, target)
            }
            LerpType::Transform(target) => {
                let start = world.get::<Transform>(self.target).unwrap();
                Lerp::Transform(start.clone(), target)
            }
        };

        world.entity_mut(entity).insert_bundle(LerpBundle {
            lerp,
            target: LerpTarget(self.target),
            timer: LerpTimer(Timer::from_seconds(self.duration, false)),
        });
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
        let bundle = world.entity_mut(entity).remove_bundle::<LerpBundle>();
        if let StopReason::Paused = reason {
            self.paused_bundle = bundle;
        }
    }
}

#[derive(Bundle)]
struct LerpBundle {
    lerp: Lerp,
    target: LerpTarget,
    timer: LerpTimer,
}

#[derive(Component)]
struct LerpTarget(Entity);

#[derive(Component)]
struct LerpTimer(Timer);

#[derive(Component)]
enum Lerp {
    Position(Vec3, Vec3),
    Rotation(Quat, Quat),
    Transform(Transform, Transform),
}

fn lerp_system(
    mut lerp_q: Query<(&mut LerpTimer, &LerpTarget, &Lerp, &mut ActionFinished)>,
    mut transform_q: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (mut timer, target, lerp, mut finished) in lerp_q.iter_mut() {
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
                finished.confirm();
            }
        } else {
            finished.confirm();
        }
    }
}

fn check_lerp_status(
    lerp_q: Query<(Entity, &LerpTimer, &LerpTarget)>,
    transform_q: Query<&Transform>,
    mut commands: Commands,
) {
    for (entity, timer, target) in lerp_q.iter() {
        if timer.0.finished() {
            commands.actions(entity).finish();
            continue;
        }

        if transform_q.get(target.0).is_err() {
            commands.actions(entity).next();
        }
    }
}
