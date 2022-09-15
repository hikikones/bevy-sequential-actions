use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*, extensions::LookRotationExt};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .add_system(wait)
        .run();
}

// struct MyAction<F, V>
// where
//     F: IntoValue<f32>,
//     V: IntoValue<Vec3>,
// {
//     float: F,
//     vec3: V,
// }

// impl<F, V> Action for MyAction<F, V>
// where
//     F: IntoValue<f32> + Send + Sync + Clone + Copy + 'static,
//     V: IntoValue<Vec3> + Send + Sync + Clone + Copy + 'static,
// {
//     fn on_start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
//         todo!()
//     }

//     fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
//         todo!()
//     }
// }

struct MyAction<F, V>
where
    F: IntoValue<f32>,
    V: IntoValue<Vec3>,
{
    float: F,
    vec3: V,
}

impl<F, V> Action for MyAction<F, V>
where
    F: IntoValue<f32>,
    V: IntoValue<Vec3>,
{
    fn on_start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
        println!("FLOAT: {}", self.float.value());
        println!("VEC3: {}", self.vec3.value());
        commands.actions(entity).next();
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {}
}

fn setup(mut commands: Commands, camera_q: Query<Entity, With<CameraMain>>) {
    let actor = commands.spawn_actor(Vec3::ZERO, Quat::IDENTITY);

    let camera = camera_q.single();

    commands
        .actions(actor)
        .add(MyAction {
            float: 5.0,
            vec3: Vec3::new(1.0, 2.0, 3.0),
        })
        .add(MyAction {
            float: Random::new(0.0, 5.0),
            vec3: Random::new(Vec3::ZERO, Vec3::splat(5.0)),
        });

    commands
        .actions(actor)
        .add(UniqueWaitAction::new(0.5))
        .add_many(
            ExecutionMode::Parallel,
            [
                MoveAction::new(Vec3::X).into_boxed(),
                UniqueWaitAction::new(1.0).into_boxed(),
                UniqueWaitAction::new(2.0).into_boxed(),
                UniqueWaitAction::new(3.0).into_boxed(),
                LerpAction::new(
                    camera,
                    LerpType::Position(CAMERA_OFFSET * 0.5 + Vec3::X),
                    4.0,
                )
                .into_boxed(),
            ]
            .into_iter(),
        )
        .add(WaitAction::new(1.0))
        .add_many(
            ExecutionMode::Parallel,
            [
                LerpAction::new(
                    actor,
                    LerpType::Rotation(Quat::look_rotation(Vec3::Z, Vec3::Y)),
                    3.0,
                )
                .into_boxed(),
                LerpAction::new(camera, LerpType::Position(CAMERA_OFFSET + Vec3::X), 4.0)
                    .into_boxed(),
            ]
            .into_iter(),
        )
        .add(WaitAction::new(1.0))
        .add(QuitAction);
}

pub struct UniqueWaitAction {
    duration: f32,
    executor: Option<Entity>,
    current: Option<f32>,
}

impl UniqueWaitAction {
    pub fn new(seconds: f32) -> Self {
        Self {
            duration: seconds,
            executor: None,
            current: None,
        }
    }
}

impl Action for UniqueWaitAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let duration = self.current.take().unwrap_or(self.duration);
        let executor = world
            .spawn()
            .insert_bundle(WaitBundle {
                wait: Wait(duration),
                agent: Agent(entity),
            })
            .id();
        self.executor = Some(executor);
    }

    fn on_stop(&mut self, _entity: Entity, world: &mut World, reason: StopReason) {
        let executor = self.executor.unwrap();

        let bundle = world.entity_mut(executor).remove_bundle::<WaitBundle>();
        if let StopReason::Paused = reason {
            self.current = Some(bundle.unwrap().wait.0);
        }

        world.despawn(executor);
    }
}

#[derive(Bundle)]
struct WaitBundle {
    wait: Wait,
    agent: Agent,
}

#[derive(Component)]
struct Wait(f32);

#[derive(Component)]
struct Agent(Entity);

fn wait(
    mut wait_q: Query<(&mut Wait, &Agent)>,
    mut action_finished_q: Query<&mut ActionFinished>,
    time: Res<Time>,
) {
    for (mut wait, agent) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();
        if wait.0 <= 0.0 {
            action_finished_q.get_mut(agent.0).unwrap().confirm();
        }
    }
}
