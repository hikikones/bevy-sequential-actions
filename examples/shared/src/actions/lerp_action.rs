use bevy::prelude::*;
use bevy_sequential_actions::*;

pub(super) struct LerpActionPlugin;

impl Plugin for LerpActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(lerp_system);
    }
}

pub struct LerpAction {
    target: Entity,
    lerp_type: LerpType,
    duration: f32,
    executor: Option<Entity>,
    current: Option<LerpBundle>,
}

impl LerpAction {
    pub fn new(target: Entity, lerp_type: LerpType, duration: f32) -> Self {
        Self {
            target,
            lerp_type,
            duration,
            executor: None,
            current: None,
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
        let lerp_bundle = if let Some(bundle) = self.current.take() {
            bundle
        } else {
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

            LerpBundle {
                lerp,
                target: LerpTarget(self.target),
                timer: LerpTimer(Timer::from_seconds(self.duration, false)),
                agent: Agent(entity),
            }
        };

        let executor = world.spawn().insert_bundle(lerp_bundle).id();
        self.executor = Some(executor);
    }

    fn on_stop(&mut self, _entity: Entity, world: &mut World, reason: StopReason) {
        let executor = self.executor.unwrap();

        let bundle = world.entity_mut(executor).remove_bundle::<LerpBundle>();
        if let StopReason::Paused = reason {
            self.current = bundle;
        }

        world.despawn(executor);
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

fn lerp_system(
    mut lerp_q: Query<(&mut LerpTimer, &LerpTarget, &Lerp, &Agent)>,
    mut transform_q: Query<&mut Transform>,
    mut action_finished_q: Query<&mut ActionFinished>,
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
                action_finished_q.get_mut(agent.0).unwrap().confirm();
            }
        } else {
            action_finished_q.get_mut(agent.0).unwrap().confirm();
        }
    }
}

// fn lerp_system(
//     mut lerp_q: Query<(&mut LerpTimer, &LerpTarget, &Lerp, &mut ActionFinished)>,
//     mut transform_q: Query<&mut Transform>,
//     time: Res<Time>,
// ) {
//     for (mut timer, target, lerp, mut finished) in lerp_q.iter_mut() {
//         if let Ok(mut transform) = transform_q.get_mut(target.0) {
//             timer.0.tick(time.delta());

//             let t = timer.0.percent();
//             let smoothstep = 3.0 * t * t - 2.0 * t * t * t;

//             match *lerp {
//                 Lerp::Position(start, end) => {
//                     transform.translation = start.lerp(end, smoothstep);
//                 }
//                 Lerp::Rotation(start, end) => {
//                     transform.rotation = start.slerp(end, smoothstep);
//                 }
//                 Lerp::Transform(start, end) => {
//                     transform.translation = start.translation.lerp(end.translation, smoothstep);
//                     transform.rotation = start.rotation.slerp(end.rotation, smoothstep);
//                 }
//             }

//             if timer.0.finished() {
//                 finished.confirm();
//             }
//         } else {
//             finished.confirm();
//         }
//     }
// }
