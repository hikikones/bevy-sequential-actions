use bevy::prelude::*;
use bevy_sequential_actions::ActionsBundle;

use super::assets::*;

pub(super) struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(load_agent.in_base_set(CoreSet::PreUpdate));
    }
}

#[derive(Component)]
pub struct Agent;

#[derive(Default)]
pub struct AgentConfig {
    pub position: Vec3,
    pub rotation: Quat,
}

pub trait SpawnAgentExt {
    fn spawn_agent(&mut self, config: AgentConfig) -> Entity;
}

impl SpawnAgentExt for Commands<'_, '_> {
    fn spawn_agent(&mut self, config: AgentConfig) -> Entity {
        self.spawn((
            SpatialBundle::from_transform(Transform {
                translation: config.position,
                rotation: config.rotation,
                ..Default::default()
            }),
            ActionsBundle::new(),
            Agent,
        ))
        .id()
    }
}

fn load_agent(
    agent_added_q: Query<Entity, Added<Agent>>,
    assets: Res<MyAssets>,
    mut commands: Commands,
) {
    for agent in agent_added_q.iter() {
        commands.entity(agent).with_children(|child| {
            // Capsule
            child.spawn(PbrBundle {
                mesh: assets.mesh(MeshName::Capsule),
                material: assets.material(MaterialName::White),
                transform: Transform {
                    translation: Vec3::Y,
                    ..Default::default()
                },
                ..Default::default()
            });

            // Eyes
            let eye_left = Vec3::new(-0.2, 1.6, -0.4);
            let eye_right = Vec3::new(-eye_left.x, eye_left.y, eye_left.z);
            let eye_scale = Vec3::splat(0.15);

            child.spawn(PbrBundle {
                mesh: assets.mesh(MeshName::Icosphere),
                material: assets.material(MaterialName::Black),
                transform: Transform {
                    translation: eye_left,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
            child.spawn(PbrBundle {
                mesh: assets.mesh(MeshName::Icosphere),
                material: assets.material(MaterialName::Black),
                transform: Transform {
                    translation: eye_right,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
        });
    }
}
