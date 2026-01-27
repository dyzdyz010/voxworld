use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoxelKind {
    Grass,
    Dirt,
    Stone,
    Sand,
    Water,
    Wood,
}

#[derive(Debug, Clone, Copy)]
pub struct VoxelProperties {
    pub temperature: f32,
    pub humidity: f32,
    pub hardness: f32,
    pub ductility: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct VoxelDef {
    pub name: &'static str,
    pub color: Color,
    pub props: VoxelProperties,
}

impl VoxelKind {
    pub fn def(self) -> VoxelDef {
        match self {
            VoxelKind::Grass => VoxelDef {
                name: "草地",
                color: Color::srgb(0.28, 0.62, 0.25),
                props: VoxelProperties {
                    temperature: 18.0,
                    humidity: 0.6,
                    hardness: 0.2,
                    ductility: 0.35,
                },
            },
            VoxelKind::Dirt => VoxelDef {
                name: "泥土",
                color: Color::srgb(0.42, 0.30, 0.18),
                props: VoxelProperties {
                    temperature: 16.0,
                    humidity: 0.4,
                    hardness: 0.35,
                    ductility: 0.2,
                },
            },
            VoxelKind::Stone => VoxelDef {
                name: "岩石",
                color: Color::srgb(0.55, 0.55, 0.58),
                props: VoxelProperties {
                    temperature: 12.0,
                    humidity: 0.1,
                    hardness: 0.9,
                    ductility: 0.05,
                },
            },
            VoxelKind::Sand => VoxelDef {
                name: "沙土",
                color: Color::srgb(0.76, 0.70, 0.38),
                props: VoxelProperties {
                    temperature: 24.0,
                    humidity: 0.2,
                    hardness: 0.25,
                    ductility: 0.45,
                },
            },
            VoxelKind::Water => VoxelDef {
                name: "水体",
                color: Color::srgba(0.20, 0.45, 0.78, 0.8),
                props: VoxelProperties {
                    temperature: 14.0,
                    humidity: 0.95,
                    hardness: 0.0,
                    ductility: 1.0,
                },
            },
            VoxelKind::Wood => VoxelDef {
                name: "木材",
                color: Color::srgb(0.55, 0.40, 0.25),
                props: VoxelProperties {
                    temperature: 20.0,
                    humidity: 0.3,
                    hardness: 0.5,
                    ductility: 0.6,
                },
            },
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Voxel {
    pub kind: VoxelKind,
    pub pos: IVec3,
}

#[derive(Resource, Default)]
pub struct VoxelWorld {
    pub map: HashMap<IVec3, VoxelKind>,
}

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>()
            .add_systems(Startup, setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<VoxelWorld>,
) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mut material_map: HashMap<VoxelKind, Handle<StandardMaterial>> = HashMap::new();
    for kind in [
        VoxelKind::Grass,
        VoxelKind::Dirt,
        VoxelKind::Stone,
        VoxelKind::Sand,
        VoxelKind::Water,
        VoxelKind::Wood,
    ] {
        let def = kind.def();
        let mut material = StandardMaterial {
            base_color: def.color,
            perceptual_roughness: 0.9,
            ..default()
        };
        if kind == VoxelKind::Water {
            material.alpha_mode = AlphaMode::Blend;
            material.perceptual_roughness = 0.1;
        }
        material_map.insert(kind, materials.add(material));
    }

    for x in -10..=10 {
        for z in -10..=10 {
            spawn_voxel(
                &mut commands,
                &mut world,
                cube.clone(),
                material_map[&VoxelKind::Grass].clone(),
                IVec3::new(x, 0, z),
                VoxelKind::Grass,
            );
            spawn_voxel(
                &mut commands,
                &mut world,
                cube.clone(),
                material_map[&VoxelKind::Dirt].clone(),
                IVec3::new(x, -1, z),
                VoxelKind::Dirt,
            );
        }
    }

    for x in -2..=2 {
        for z in -2..=2 {
            spawn_voxel(
                &mut commands,
                &mut world,
                cube.clone(),
                material_map[&VoxelKind::Stone].clone(),
                IVec3::new(x, 1, z),
                VoxelKind::Stone,
            );
        }
    }

    for x in 5..=8 {
        for z in -3..=1 {
            spawn_voxel(
                &mut commands,
                &mut world,
                cube.clone(),
                material_map[&VoxelKind::Sand].clone(),
                IVec3::new(x, 0, z),
                VoxelKind::Sand,
            );
        }
    }

    for x in -6..=-3 {
        for z in 4..=7 {
            spawn_voxel(
                &mut commands,
                &mut world,
                cube.clone(),
                material_map[&VoxelKind::Water].clone(),
                IVec3::new(x, 0, z),
                VoxelKind::Water,
            );
        }
    }

    for y in 1..=4 {
        spawn_voxel(
            &mut commands,
            &mut world,
            cube.clone(),
            material_map[&VoxelKind::Wood].clone(),
            IVec3::new(-7, y, -7),
            VoxelKind::Wood,
        );
    }
}

fn spawn_voxel(
    commands: &mut Commands,
    world: &mut VoxelWorld,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    pos: IVec3,
    kind: VoxelKind,
) {
    if world.map.contains_key(&pos) {
        return;
    }
    world.map.insert(pos, kind);
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(ivec3_to_vec3(pos)),
        Voxel { kind, pos },
    ));
}

pub fn ivec3_to_vec3(pos: IVec3) -> Vec3 {
    Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32)
}
