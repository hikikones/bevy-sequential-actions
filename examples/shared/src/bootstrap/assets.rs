use bevy::{prelude::*, utils::HashMap};

pub(super) struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        #[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
        #[system_set(base)]
        struct LoadAssets;

        app.insert_resource(MyAssets::default()).add_startup_system(
            load_assets
                .in_base_set(LoadAssets)
                .before(StartupSet::PreStartup),
        );
    }
}

#[derive(Default, Resource)]
pub(super) struct MyAssets {
    meshes: HashMap<MeshName, Handle<Mesh>>,
    materials: HashMap<MaterialName, Handle<StandardMaterial>>,
}

impl MyAssets {
    pub fn mesh(&self, name: MeshName) -> Handle<Mesh> {
        self.meshes[&name].clone_weak()
    }

    pub fn material(&self, name: MaterialName) -> Handle<StandardMaterial> {
        self.materials[&name].clone_weak()
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum MeshName {
    Quad,
    Cube,
    Capsule,
    Icosphere,
}

impl MeshName {
    fn mesh(&self) -> Mesh {
        match self {
            MeshName::Quad => shape::Quad::default().into(),
            MeshName::Cube => shape::Cube::default().into(),
            MeshName::Capsule => shape::Capsule::default().into(),
            MeshName::Icosphere => shape::Icosphere::default().try_into().unwrap(),
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

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum MaterialName {
    White,
    Black,
    Red,
    Silver,
    SeaGreen,
    DarkGray,
    Cyan,
    MidnightBlue,
}

impl MaterialName {
    fn color(&self) -> Color {
        match self {
            MaterialName::Black => Color::BLACK,
            MaterialName::White => Color::WHITE,
            MaterialName::Red => Color::RED,
            MaterialName::Silver => Color::SILVER,
            MaterialName::SeaGreen => Color::SEA_GREEN,
            MaterialName::DarkGray => Color::DARK_GRAY,
            MaterialName::Cyan => Color::CYAN,
            MaterialName::MidnightBlue => Color::MIDNIGHT_BLUE,
        }
    }

    fn iter() -> impl Iterator<Item = Self> {
        [
            MaterialName::White,
            MaterialName::Black,
            MaterialName::Red,
            MaterialName::Silver,
            MaterialName::SeaGreen,
            MaterialName::DarkGray,
            MaterialName::Cyan,
            MaterialName::MidnightBlue,
        ]
        .into_iter()
    }
}

fn load_assets(
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
    mut my_assets: ResMut<MyAssets>,
) {
    for mesh in MeshName::iter() {
        my_assets
            .meshes
            .insert(mesh, mesh_assets.add(mesh.mesh().into()));
    }

    for mat in MaterialName::iter() {
        my_assets
            .materials
            .insert(mat, mat_assets.add(mat.color().into()));
    }
}
