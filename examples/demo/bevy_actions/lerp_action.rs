use bevy::prelude::*;

use bevy_sequential_actions::*;

use crate::bevy_extensions::LookRotationExt;

pub(super) struct LerpActionPlugin;

impl Plugin for LerpActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(lerp);
    }
}

pub struct LerpAction {
    target: Entity,
    lerp_type: LerpType,
    duration: f32,
    executor: Option<Entity>,
}

impl LerpAction {
    pub fn new(target: Entity, lerp_type: LerpType, duration: f32) -> Self {
        Self {
            target,
            lerp_type,
            duration,
            executor: None,
        }
    }
}

pub enum LerpType {
    Position(Vec3),
    Rotation(Quat),
    Move(Entity),
    Face(Entity),
}

impl Action for LerpAction {
    fn add(&mut self, actor: Entity, world: &mut World, commands: &mut ActionCommands) {
        let lerp = match self.lerp_type {
            LerpType::Position(target) => {
                let start = world.get::<Transform>(self.target).unwrap().translation;
                Lerp::Position(start, target)
            }
            LerpType::Rotation(target) => {
                let start = world.get::<Transform>(self.target).unwrap().rotation;
                Lerp::Rotation(start, target)
            }
            LerpType::Move(entity) => {
                let start = world.get::<Transform>(self.target).unwrap().translation;
                let target = world.get::<GlobalTransform>(entity).unwrap().translation;
                Lerp::Position(start, target)
            }
            LerpType::Face(entity) => {
                let a = world.get::<Transform>(self.target).unwrap();
                let b = world.get::<GlobalTransform>(entity).unwrap();
                let dir = b.translation - a.translation;

                if dir == Vec3::ZERO {
                    commands.next_action(actor);
                    return;
                }

                let look = Quat::look_rotation(Vec3::new(dir.x, 0.0, dir.z), Vec3::Y);
                Lerp::Rotation(a.rotation, look)
            }
        };

        let executor = world
            .spawn()
            .insert_bundle(LerpBundle {
                lerp,
                target: Target(self.target),
                timer: LerpTimer(Timer::from_seconds(self.duration, false)),
                actor: Actor(actor),
            })
            .id();
        self.executor = Some(executor);
    }

    fn remove(&mut self, _actor: Entity, world: &mut World) {
        if let Some(executor) = self.executor {
            world.despawn(executor);
        }
    }

    fn stop(&mut self, _actor: Entity, _world: &mut World) {
        todo!()
    }
}

#[derive(Bundle)]
struct LerpBundle {
    lerp: Lerp,
    target: Target,
    timer: LerpTimer,
    actor: Actor,
}

#[derive(Component)]
struct Target(Entity);

#[derive(Component)]
struct Actor(Entity);

#[derive(Component)]
struct LerpTimer(Timer);

#[derive(Component)]
enum Lerp {
    Position(Vec3, Vec3),
    Rotation(Quat, Quat),
}

fn lerp(
    mut lerp_q: Query<(&mut LerpTimer, &Target, &Lerp, &Actor)>,
    mut target_q: Query<&mut Transform>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut timer, target, lerp, actor) in lerp_q.iter_mut() {
        if let Ok(mut transform) = target_q.get_mut(target.0) {
            timer.0.tick(time.delta());

            let t = timer.0.percent();
            let smoothstep = 3.0 * t * t - 2.0 * t * t * t;

            match *lerp {
                Lerp::Position(start, end) => {
                    transform.translation = Vec3::lerp(start, end, smoothstep);
                }
                Lerp::Rotation(start, end) => {
                    transform.rotation = Quat::slerp(start, end, smoothstep);
                }
            }

            if timer.0.finished() {
                commands.action(actor.0).next();
            }
        } else {
            commands.action(actor.0).next();
        }
    }
}
