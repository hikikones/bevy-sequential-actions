use bevy::{prelude::*, render::view::screenshot::ScreenshotManager, window::PrimaryWindow};
use bevy_sequential_actions::*;

fn main() {
    std::fs::remove_dir_all("./screenshots").ok();
    std::fs::create_dir_all("./screenshots").unwrap();

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "gif".into(),
                    resolution: bevy::window::WindowResolution::new(720.0, 480.0),
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            SequentialActionsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                record_screenshots,
                wait,
                lerp_position,
                lerp_rotation,
                bevy::window::close_on_esc,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Capsule guy
    let capsule_guy = commands
        .spawn((PbrBundle {
            mesh: meshes.add(shape::Capsule::default().into()),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ..Default::default()
        },))
        .with_children(|builder| {
            let icosphere = meshes.add(shape::Icosphere::default().try_into().unwrap());
            let black = materials.add(Color::BLACK.into());

            let eye_left = Vec3::new(-0.2, 0.6, -0.4);
            let eye_right = Vec3::new(-eye_left.x, eye_left.y, eye_left.z);
            let eye_scale = Vec3::splat(0.15);

            builder.spawn(PbrBundle {
                mesh: icosphere.clone(),
                material: black.clone(),
                transform: Transform {
                    translation: eye_left,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
            builder.spawn(PbrBundle {
                mesh: icosphere,
                material: black,
                transform: Transform {
                    translation: eye_right,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
        })
        .id();

    // Floor
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Cube::default().into()),
        material: materials.add(Color::DARK_GRAY.into()),
        transform: Transform {
            translation: Vec3::NEG_Y * 1.5,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(10.0, 1.0, 5.0),
        },
        ..Default::default()
    });

    // Camera
    let camera = commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.5, 5.0)),
            ..Default::default()
        })
        .id();

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 25000.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // Executor
    let executor = commands.spawn(ActionsBundle::new()).id();
    commands.actions(executor).add_many(actions![
        WaitAction(1.0),
        ParallelActions(actions![
            LerpPositionAction {
                entity: camera,
                target: Vec3::new(0.0, 10.0, 10.0),
                duration: 1.0,
            },
            LerpRotationAction {
                entity: camera,
                target: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
                duration: 1.0,
            },
        ]),
        ParallelActions(actions![
            LerpPositionAction {
                entity: capsule_guy,
                target: Vec3::X * 4.0,
                duration: 1.0,
            },
            LerpRotationAction {
                entity: capsule_guy,
                target: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
                duration: 0.5,
            },
        ]),
        WaitAction(0.5),
        ParallelActions(actions![
            LerpPositionAction {
                entity: capsule_guy,
                target: Vec3::NEG_X * 4.0,
                duration: 1.5,
            },
            LerpRotationAction {
                entity: capsule_guy,
                target: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                duration: 0.5,
            },
        ]),
        WaitAction(0.5),
        ParallelActions(actions![
            LerpPositionAction {
                entity: capsule_guy,
                target: Vec3::ZERO,
                duration: 1.0,
            },
            LerpRotationAction {
                entity: capsule_guy,
                target: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
                duration: 0.5,
            },
        ]),
        ParallelActions(actions![
            LerpPositionAction {
                entity: camera,
                target: Vec3::new(0.0, 0.5, 5.0),
                duration: 1.0,
            },
            LerpRotationAction {
                entity: camera,
                target: Quat::IDENTITY,
                duration: 1.0,
            },
            LerpRotationAction {
                entity: capsule_guy,
                target: Quat::from_rotation_y(std::f32::consts::PI),
                duration: 1.0,
            },
        ]),
    ]);
}

fn record_screenshots(
    mut screenshot_manager: ResMut<ScreenshotManager>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut frame: Local<u32>,
) {
    screenshot_manager
        .save_screenshot_to_disk(
            primary_window.single(),
            format!("screenshots/{:04}.png", *frame),
        )
        .unwrap();

    *frame += 1;
}

struct WaitAction(f32);

impl Action for WaitAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        world.get::<WaitTimer>(agent).unwrap().0 <= 0.0
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        world.entity_mut(agent).insert(WaitTimer(self.0));
        false
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, _reason: StopReason) {
        world.entity_mut(agent).remove::<WaitTimer>();
    }
}

#[derive(Component)]
struct WaitTimer(f32);

fn wait(mut wait_q: Query<&mut WaitTimer>, time: Res<Time>) {
    for mut wait_timer in &mut wait_q {
        wait_timer.0 -= time.delta_seconds();
    }
}

struct LerpPositionAction {
    entity: Entity,
    target: Vec3,
    duration: f32,
}

impl Action for LerpPositionAction {
    fn is_finished(&self, _agent: Entity, world: &World) -> bool {
        world
            .get::<LerpPositionTimer>(self.entity)
            .unwrap()
            .0
            .finished()
    }

    fn on_start(&mut self, _agent: Entity, world: &mut World) -> bool {
        let start = world.get::<Transform>(self.entity).unwrap().translation;
        world.entity_mut(self.entity).insert((
            LerpPositionTimer(Timer::from_seconds(self.duration, TimerMode::Once)),
            LerpPosition(start, self.target),
        ));
        false
    }

    fn on_stop(&mut self, _agent: Entity, world: &mut World, _reason: StopReason) {
        world.entity_mut(self.entity).remove::<LerpPositionTimer>();
    }
}

#[derive(Component)]
struct LerpPositionTimer(Timer);

#[derive(Component)]
struct LerpPosition(Vec3, Vec3);

fn lerp_position(
    mut lerp_q: Query<(&mut LerpPositionTimer, &mut Transform, &LerpPosition)>,
    time: Res<Time>,
) {
    for (mut timer, mut transform, pos) in &mut lerp_q {
        timer.0.tick(time.delta());

        let t = timer.0.percent();
        let smoothstep = 3.0 * t * t - 2.0 * t * t * t;

        transform.translation = Vec3::lerp(pos.0, pos.1, smoothstep);
    }
}

struct LerpRotationAction {
    entity: Entity,
    target: Quat,
    duration: f32,
}

impl Action for LerpRotationAction {
    fn is_finished(&self, _agent: Entity, world: &World) -> bool {
        world
            .get::<LerpRotationTimer>(self.entity)
            .unwrap()
            .0
            .finished()
    }

    fn on_start(&mut self, _agent: Entity, world: &mut World) -> bool {
        let start = world.get::<Transform>(self.entity).unwrap().rotation;
        world.entity_mut(self.entity).insert((
            LerpRotationTimer(Timer::from_seconds(self.duration, TimerMode::Once)),
            LerpRotation(start, self.target),
        ));
        false
    }

    fn on_stop(&mut self, _agent: Entity, world: &mut World, _reason: StopReason) {
        world.entity_mut(self.entity).remove::<LerpRotationTimer>();
    }
}

#[derive(Component)]
struct LerpRotationTimer(Timer);

#[derive(Component)]
struct LerpRotation(Quat, Quat);

fn lerp_rotation(
    mut lerp_q: Query<(&mut LerpRotationTimer, &mut Transform, &LerpRotation)>,
    time: Res<Time>,
) {
    for (mut timer, mut transform, rot) in &mut lerp_q {
        timer.0.tick(time.delta());

        let t = timer.0.percent();
        let smoothstep = 3.0 * t * t - 2.0 * t * t * t;

        transform.rotation = Quat::slerp(rot.0, rot.1, smoothstep);
    }
}

struct ParallelActions<const N: usize>([BoxedAction; N]);

impl<const N: usize> Action for ParallelActions<N> {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.0.iter().all(|action| action.is_finished(agent, world))
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        self.0
            .iter_mut()
            .for_each(|action| action.on_add(agent, world));
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        std::array::from_fn::<bool, N, _>(|i| self.0[i].on_start(agent, world))
            .into_iter()
            .all(|b| b)
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        self.0
            .iter_mut()
            .for_each(|action| action.on_stop(agent, world, reason));
    }

    fn on_remove(&mut self, agent: Entity, world: &mut World) {
        self.0
            .iter_mut()
            .for_each(|action| action.on_remove(agent, world));
    }
}
