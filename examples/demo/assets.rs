use bevy::{prelude::*, utils::HashMap};

pub(super) struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Meshes::default())
            .insert_resource(Materials::default())
            .add_startup_system_to_stage(StartupStage::PreStartup, load);
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum MeshName {
    Quad,
    Cube,
    Capsule,
    Icosphere,
}

impl MeshName {
    fn mesh(&self) -> Mesh {
        match self {
            MeshName::Quad => Mesh::from(shape::Quad::default()),
            MeshName::Cube => Mesh::from(shape::Cube::default()),
            MeshName::Capsule => Mesh::from(shape::Capsule::default()),
            MeshName::Icosphere => Mesh::from(shape::Icosphere::default()),
        }
    }

    fn iter() -> impl Iterator<Item = Self> {
        [
            MeshName::Quad,
            MeshName::Cube,
            MeshName::Capsule,
            MeshName::Icosphere,
        ]
        .into_iter()
    }
}

#[derive(Default)]
pub struct Meshes(HashMap<MeshName, Handle<Mesh>>);

impl Meshes {
    pub fn get(&self, name: MeshName) -> Handle<Mesh> {
        self.0[&name].clone()
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum MaterialName {
    None,
    White,
    Black,
    Red,
    Silver,
    SeaGreen,
    DarkGray,
    Maroon,
    Gold,
    AliceBlue,
    AquaMarine,
}

impl MaterialName {
    fn color(&self) -> Color {
        match self {
            MaterialName::None => Color::NONE,
            MaterialName::Black => Color::BLACK,
            MaterialName::White => Color::WHITE,
            MaterialName::Red => Color::RED,
            MaterialName::Silver => Color::SILVER,
            MaterialName::SeaGreen => Color::SEA_GREEN,
            MaterialName::DarkGray => Color::DARK_GRAY,
            MaterialName::Maroon => Color::MAROON,
            MaterialName::Gold => Color::GOLD,
            MaterialName::AliceBlue => Color::ALICE_BLUE,
            MaterialName::AquaMarine => Color::AQUAMARINE,
        }
    }

    fn iter() -> impl Iterator<Item = Self> {
        [
            MaterialName::None,
            MaterialName::White,
            MaterialName::Black,
            MaterialName::Red,
            MaterialName::Silver,
            MaterialName::SeaGreen,
            MaterialName::DarkGray,
            MaterialName::Maroon,
            MaterialName::Gold,
            MaterialName::AliceBlue,
            MaterialName::AquaMarine,
        ]
        .into_iter()
    }
}

#[derive(Default)]
pub struct Materials(HashMap<MaterialName, Handle<StandardMaterial>>);

impl Materials {
    pub fn get(&self, name: MaterialName) -> Handle<StandardMaterial> {
        self.0[&name].clone()
    }
}

fn load(
    mut meshes: ResMut<Meshes>,
    mut materials: ResMut<Materials>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
) {
    for mesh in MeshName::iter() {
        meshes.0.insert(mesh, mesh_assets.add(mesh.mesh().into()));
    }

    for mat in MaterialName::iter() {
        materials.0.insert(mat, mat_assets.add(mat.color().into()));
    }
}
