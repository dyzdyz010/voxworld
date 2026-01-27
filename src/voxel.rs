use bevy::prelude::*;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues};
use noise::{NoiseFn, Perlin};
use std::collections::HashMap;

// ============================================================================
// Constants
// ============================================================================

pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_HEIGHT: i32 = 64;
pub const RENDER_DISTANCE: i32 = 4; // chunks

// ============================================================================
// Block Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VoxelKind {
    #[default]
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Gravel,
    Clay,
    Snow,
    Ice,
    Water,
    OakLog,
    OakLeaves,
    BirchLog,
    BirchLeaves,
    SpruceLog,
    SpruceLeaves,
    Cactus,
    CoalOre,
    IronOre,
    GoldOre,
    DiamondOre,
    Flower,
    TallGrass,
    DeadBush,
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
            VoxelKind::Air => VoxelDef {
                name: "空气",
                color: Color::NONE,
                props: VoxelProperties {
                    temperature: 20.0,
                    humidity: 0.5,
                    hardness: 0.0,
                    ductility: 0.0,
                },
            },
            VoxelKind::Grass => VoxelDef {
                name: "草方块",
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
                name: "石头",
                color: Color::srgb(0.55, 0.55, 0.58),
                props: VoxelProperties {
                    temperature: 12.0,
                    humidity: 0.1,
                    hardness: 0.9,
                    ductility: 0.05,
                },
            },
            VoxelKind::Sand => VoxelDef {
                name: "沙子",
                color: Color::srgb(0.86, 0.82, 0.58),
                props: VoxelProperties {
                    temperature: 28.0,
                    humidity: 0.05,
                    hardness: 0.25,
                    ductility: 0.45,
                },
            },
            VoxelKind::Gravel => VoxelDef {
                name: "砂砾",
                color: Color::srgb(0.52, 0.50, 0.48),
                props: VoxelProperties {
                    temperature: 14.0,
                    humidity: 0.15,
                    hardness: 0.4,
                    ductility: 0.3,
                },
            },
            VoxelKind::Clay => VoxelDef {
                name: "黏土",
                color: Color::srgb(0.62, 0.64, 0.68),
                props: VoxelProperties {
                    temperature: 15.0,
                    humidity: 0.7,
                    hardness: 0.3,
                    ductility: 0.5,
                },
            },
            VoxelKind::Snow => VoxelDef {
                name: "雪块",
                color: Color::srgb(0.95, 0.97, 1.0),
                props: VoxelProperties {
                    temperature: -5.0,
                    humidity: 0.8,
                    hardness: 0.1,
                    ductility: 0.2,
                },
            },
            VoxelKind::Ice => VoxelDef {
                name: "冰块",
                color: Color::srgba(0.68, 0.85, 0.95, 0.85),
                props: VoxelProperties {
                    temperature: -10.0,
                    humidity: 0.9,
                    hardness: 0.3,
                    ductility: 0.1,
                },
            },
            VoxelKind::Water => VoxelDef {
                name: "水",
                color: Color::srgba(0.20, 0.45, 0.78, 0.7),
                props: VoxelProperties {
                    temperature: 14.0,
                    humidity: 1.0,
                    hardness: 0.0,
                    ductility: 1.0,
                },
            },
            VoxelKind::OakLog => VoxelDef {
                name: "橡木原木",
                color: Color::srgb(0.40, 0.30, 0.18),
                props: VoxelProperties {
                    temperature: 20.0,
                    humidity: 0.3,
                    hardness: 0.5,
                    ductility: 0.6,
                },
            },
            VoxelKind::OakLeaves => VoxelDef {
                name: "橡树树叶",
                color: Color::srgba(0.22, 0.52, 0.20, 0.9),
                props: VoxelProperties {
                    temperature: 22.0,
                    humidity: 0.5,
                    hardness: 0.05,
                    ductility: 0.1,
                },
            },
            VoxelKind::BirchLog => VoxelDef {
                name: "白桦原木",
                color: Color::srgb(0.85, 0.82, 0.75),
                props: VoxelProperties {
                    temperature: 18.0,
                    humidity: 0.35,
                    hardness: 0.45,
                    ductility: 0.55,
                },
            },
            VoxelKind::BirchLeaves => VoxelDef {
                name: "白桦树叶",
                color: Color::srgba(0.45, 0.62, 0.35, 0.9),
                props: VoxelProperties {
                    temperature: 20.0,
                    humidity: 0.45,
                    hardness: 0.05,
                    ductility: 0.1,
                },
            },
            VoxelKind::SpruceLog => VoxelDef {
                name: "云杉原木",
                color: Color::srgb(0.30, 0.22, 0.12),
                props: VoxelProperties {
                    temperature: 8.0,
                    humidity: 0.4,
                    hardness: 0.55,
                    ductility: 0.5,
                },
            },
            VoxelKind::SpruceLeaves => VoxelDef {
                name: "云杉树叶",
                color: Color::srgba(0.15, 0.35, 0.22, 0.9),
                props: VoxelProperties {
                    temperature: 6.0,
                    humidity: 0.5,
                    hardness: 0.05,
                    ductility: 0.1,
                },
            },
            VoxelKind::Cactus => VoxelDef {
                name: "仙人掌",
                color: Color::srgb(0.25, 0.55, 0.20),
                props: VoxelProperties {
                    temperature: 35.0,
                    humidity: 0.1,
                    hardness: 0.2,
                    ductility: 0.3,
                },
            },
            VoxelKind::CoalOre => VoxelDef {
                name: "煤矿石",
                color: Color::srgb(0.25, 0.25, 0.28),
                props: VoxelProperties {
                    temperature: 12.0,
                    humidity: 0.1,
                    hardness: 0.85,
                    ductility: 0.05,
                },
            },
            VoxelKind::IronOre => VoxelDef {
                name: "铁矿石",
                color: Color::srgb(0.58, 0.52, 0.48),
                props: VoxelProperties {
                    temperature: 12.0,
                    humidity: 0.1,
                    hardness: 0.9,
                    ductility: 0.05,
                },
            },
            VoxelKind::GoldOre => VoxelDef {
                name: "金矿石",
                color: Color::srgb(0.72, 0.65, 0.35),
                props: VoxelProperties {
                    temperature: 12.0,
                    humidity: 0.1,
                    hardness: 0.85,
                    ductility: 0.15,
                },
            },
            VoxelKind::DiamondOre => VoxelDef {
                name: "钻石矿石",
                color: Color::srgb(0.45, 0.72, 0.78),
                props: VoxelProperties {
                    temperature: 12.0,
                    humidity: 0.1,
                    hardness: 0.98,
                    ductility: 0.02,
                },
            },
            VoxelKind::Flower => VoxelDef {
                name: "花",
                color: Color::srgb(0.85, 0.35, 0.40),
                props: VoxelProperties {
                    temperature: 22.0,
                    humidity: 0.6,
                    hardness: 0.01,
                    ductility: 0.05,
                },
            },
            VoxelKind::TallGrass => VoxelDef {
                name: "高草丛",
                color: Color::srgb(0.35, 0.58, 0.28),
                props: VoxelProperties {
                    temperature: 20.0,
                    humidity: 0.5,
                    hardness: 0.01,
                    ductility: 0.05,
                },
            },
            VoxelKind::DeadBush => VoxelDef {
                name: "枯死的灌木",
                color: Color::srgb(0.55, 0.45, 0.28),
                props: VoxelProperties {
                    temperature: 32.0,
                    humidity: 0.05,
                    hardness: 0.01,
                    ductility: 0.02,
                },
            },
        }
    }

    pub fn is_transparent(self) -> bool {
        matches!(
            self,
            VoxelKind::Air
                | VoxelKind::Water
                | VoxelKind::Ice
                | VoxelKind::OakLeaves
                | VoxelKind::BirchLeaves
                | VoxelKind::SpruceLeaves
                | VoxelKind::Flower
                | VoxelKind::TallGrass
                | VoxelKind::DeadBush
        )
    }

    pub fn is_solid(self) -> bool {
        !matches!(self, VoxelKind::Air | VoxelKind::Flower | VoxelKind::TallGrass | VoxelKind::DeadBush)
    }
}

// ============================================================================
// Biomes
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Forest,
    BirchForest,
    Desert,
    Snowy,
    Taiga,
    Ocean,
    Beach,
}

impl Biome {
    pub fn surface_block(self) -> VoxelKind {
        match self {
            Biome::Plains | Biome::Forest | Biome::BirchForest => VoxelKind::Grass,
            Biome::Desert => VoxelKind::Sand,
            Biome::Snowy => VoxelKind::Snow,
            Biome::Taiga => VoxelKind::Grass,
            Biome::Ocean => VoxelKind::Gravel,
            Biome::Beach => VoxelKind::Sand,
        }
    }

    pub fn subsurface_block(self) -> VoxelKind {
        match self {
            Biome::Plains | Biome::Forest | Biome::BirchForest | Biome::Taiga => VoxelKind::Dirt,
            Biome::Desert | Biome::Beach => VoxelKind::Sand,
            Biome::Snowy => VoxelKind::Dirt,
            Biome::Ocean => VoxelKind::Clay,
        }
    }
}

// ============================================================================
// World Seed & Generation
// ============================================================================

#[derive(Resource)]
pub struct WorldSeed {
    pub seed: u32,
    pub terrain_noise: Perlin,
    pub biome_temp_noise: Perlin,
    pub biome_humid_noise: Perlin,
    pub cave_noise: Perlin,
    pub detail_noise: Perlin,
}

impl WorldSeed {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            terrain_noise: Perlin::new(seed),
            biome_temp_noise: Perlin::new(seed.wrapping_add(1000)),
            biome_humid_noise: Perlin::new(seed.wrapping_add(2000)),
            cave_noise: Perlin::new(seed.wrapping_add(3000)),
            detail_noise: Perlin::new(seed.wrapping_add(4000)),
        }
    }

    pub fn from_string(s: &str) -> Self {
        let seed = s
            .bytes()
            .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        Self::new(seed)
    }
}

impl Default for WorldSeed {
    fn default() -> Self {
        Self::new(12345)
    }
}

// ============================================================================
// Chunk System
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn from_world_pos(world_x: i32, world_z: i32) -> Self {
        Self {
            x: world_x.div_euclid(CHUNK_SIZE),
            z: world_z.div_euclid(CHUNK_SIZE),
        }
    }

    pub fn world_origin(&self) -> IVec3 {
        IVec3::new(self.x * CHUNK_SIZE, 0, self.z * CHUNK_SIZE)
    }
}

#[derive(Component)]
pub struct ChunkMarker {
    pub pos: ChunkPos,
}

pub struct ChunkData {
    pub voxels: Vec<VoxelKind>, // CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE
    pub is_dirty: bool,
}

impl ChunkData {
    pub fn new() -> Self {
        Self {
            voxels: vec![VoxelKind::Air; (CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE) as usize],
            is_dirty: true,
        }
    }

    #[inline]
    fn index(x: i32, y: i32, z: i32) -> usize {
        ((y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x) as usize
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> VoxelKind {
        if x < 0 || x >= CHUNK_SIZE || y < 0 || y >= CHUNK_HEIGHT || z < 0 || z >= CHUNK_SIZE {
            return VoxelKind::Air;
        }
        self.voxels[Self::index(x, y, z)]
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, kind: VoxelKind) {
        if x < 0 || x >= CHUNK_SIZE || y < 0 || y >= CHUNK_HEIGHT || z < 0 || z >= CHUNK_SIZE {
            return;
        }
        self.voxels[Self::index(x, y, z)] = kind;
        self.is_dirty = true;
    }
}

#[derive(Resource, Default)]
pub struct VoxelWorld {
    pub chunks: HashMap<ChunkPos, ChunkData>,
    pub loaded_chunks: HashMap<ChunkPos, Entity>,
}

impl VoxelWorld {
    pub fn get_voxel(&self, world_pos: IVec3) -> VoxelKind {
        let chunk_pos = ChunkPos::from_world_pos(world_pos.x, world_pos.z);
        let local_x = world_pos.x.rem_euclid(CHUNK_SIZE);
        let local_z = world_pos.z.rem_euclid(CHUNK_SIZE);

        self.chunks
            .get(&chunk_pos)
            .map(|chunk| chunk.get(local_x, world_pos.y, local_z))
            .unwrap_or(VoxelKind::Air)
    }

    pub fn set_voxel(&mut self, world_pos: IVec3, kind: VoxelKind) {
        let chunk_pos = ChunkPos::from_world_pos(world_pos.x, world_pos.z);
        let local_x = world_pos.x.rem_euclid(CHUNK_SIZE);
        let local_z = world_pos.z.rem_euclid(CHUNK_SIZE);

        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set(local_x, world_pos.y, local_z, kind);
        }
    }
}

// ============================================================================
// Terrain Generator
// ============================================================================

pub struct TerrainGenerator<'a> {
    seed: &'a WorldSeed,
}

impl<'a> TerrainGenerator<'a> {
    pub fn new(seed: &'a WorldSeed) -> Self {
        Self { seed }
    }

    pub fn get_height(&self, x: i32, z: i32) -> i32 {
        let scale = 0.02;
        let fx = x as f64 * scale;
        let fz = z as f64 * scale;

        let mut height = 0.0;
        height += self.seed.terrain_noise.get([fx, fz]) * 12.0;
        height += self.seed.terrain_noise.get([fx * 2.0, fz * 2.0]) * 6.0;
        height += self.seed.detail_noise.get([fx * 4.0, fz * 4.0]) * 3.0;

        let base_height = 32;
        ((base_height as f64 + height) as i32).clamp(1, CHUNK_HEIGHT - 10)
    }

    pub fn get_biome(&self, x: i32, z: i32) -> Biome {
        let scale = 0.008;
        let fx = x as f64 * scale;
        let fz = z as f64 * scale;

        let temp = self.seed.biome_temp_noise.get([fx, fz]);
        let humid = self.seed.biome_humid_noise.get([fx, fz]);
        let height = self.get_height(x, z);

        if height < 28 {
            return Biome::Ocean;
        }
        if height < 32 {
            return Biome::Beach;
        }

        match (temp, humid) {
            (t, _) if t < -0.3 => {
                if humid > 0.2 {
                    Biome::Taiga
                } else {
                    Biome::Snowy
                }
            }
            (t, h) if t > 0.3 && h < -0.2 => Biome::Desert,
            (_, h) if h > 0.3 => Biome::Forest,
            (_, h) if h > 0.0 => Biome::BirchForest,
            _ => Biome::Plains,
        }
    }

    pub fn is_cave(&self, x: i32, y: i32, z: i32) -> bool {
        if y > 40 || y < 5 {
            return false;
        }
        let scale = 0.08;
        let value = self
            .seed
            .cave_noise
            .get([x as f64 * scale, y as f64 * scale, z as f64 * scale]);
        value > 0.55
    }

    pub fn get_ore(&self, x: i32, y: i32, z: i32) -> Option<VoxelKind> {
        let scale = 0.15;
        let noise = self
            .seed
            .detail_noise
            .get([x as f64 * scale, y as f64 * scale, z as f64 * scale]);

        if noise > 0.7 {
            if y < 16 && noise > 0.85 {
                Some(VoxelKind::DiamondOre)
            } else if y < 32 && noise > 0.80 {
                Some(VoxelKind::GoldOre)
            } else if y < 48 {
                Some(VoxelKind::IronOre)
            } else {
                Some(VoxelKind::CoalOre)
            }
        } else {
            None
        }
    }

    pub fn should_place_tree(&self, x: i32, z: i32, biome: Biome) -> bool {
        let tree_chance = match biome {
            Biome::Forest | Biome::Taiga => 0.06,
            Biome::BirchForest => 0.04,
            Biome::Plains => 0.003,
            _ => 0.0,
        };

        if tree_chance == 0.0 {
            return false;
        }

        let scale = 0.5;
        let noise = self.seed.detail_noise.get([x as f64 * scale, z as f64 * scale]);
        noise > (1.0 - tree_chance * 2.0)
    }

    pub fn generate_chunk(&self, chunk_pos: ChunkPos) -> ChunkData {
        let mut chunk = ChunkData::new();
        let origin = chunk_pos.world_origin();
        let water_level = 30;

        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let wx = origin.x + lx;
                let wz = origin.z + lz;
                let height = self.get_height(wx, wz);
                let biome = self.get_biome(wx, wz);

                for y in 0..CHUNK_HEIGHT {
                    if self.is_cave(wx, y, wz) && y < height {
                        continue;
                    }

                    let kind = if y > height && y <= water_level {
                        if biome == Biome::Snowy && y == water_level {
                            VoxelKind::Ice
                        } else {
                            VoxelKind::Water
                        }
                    } else if y == height {
                        biome.surface_block()
                    } else if y > height - 4 && y < height {
                        biome.subsurface_block()
                    } else if y < height {
                        self.get_ore(wx, y, wz).unwrap_or(VoxelKind::Stone)
                    } else {
                        VoxelKind::Air
                    };

                    if kind != VoxelKind::Air {
                        chunk.set(lx, y, lz, kind);
                    }
                }

                // Trees
                if height > water_level && self.should_place_tree(wx, wz, biome) {
                    self.generate_tree(&mut chunk, lx, height + 1, lz, biome);
                }
            }
        }

        chunk
    }

    fn generate_tree(&self, chunk: &mut ChunkData, x: i32, y: i32, z: i32, biome: Biome) {
        let (log, leaves, trunk_h) = match biome {
            Biome::Forest | Biome::Plains => (VoxelKind::OakLog, VoxelKind::OakLeaves, 5),
            Biome::BirchForest => (VoxelKind::BirchLog, VoxelKind::BirchLeaves, 6),
            Biome::Taiga | Biome::Snowy => (VoxelKind::SpruceLog, VoxelKind::SpruceLeaves, 6),
            _ => return,
        };

        // Trunk
        for dy in 0..trunk_h {
            chunk.set(x, y + dy, z, log);
        }

        // Leaves
        let leaf_start = trunk_h - 2;
        for dy in leaf_start..trunk_h + 2 {
            let radius: i32 = if dy >= trunk_h { 1 } else { 2 };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    if dx == 0 && dz == 0 && dy < trunk_h {
                        continue;
                    }
                    if dx.abs() + dz.abs() <= radius + 1 {
                        chunk.set(x + dx, y + dy, z + dz, leaves);
                    }
                }
            }
        }
    }
}

// ============================================================================
// Greedy Meshing
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
struct FaceData {
    kind: VoxelKind,
    ao: u8, // ambient occlusion
}

struct ChunkMeshBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl ChunkMeshBuilder {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn add_face(
        &mut self,
        vertices: [[f32; 3]; 4],
        normal: [f32; 3],
        color: [f32; 4],
    ) {
        let base = self.positions.len() as u32;

        for v in vertices {
            self.positions.push(v);
            self.normals.push(normal);
            self.colors.push(color);
        }

        // Two triangles per face (counter-clockwise winding)
        self.indices.extend_from_slice(&[
            base, base + 2, base + 1,
            base, base + 3, base + 2,
        ]);
    }

    fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, VertexAttributeValues::Float32x4(self.colors));
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

pub fn build_chunk_mesh(chunk: &ChunkData, world: &VoxelWorld, chunk_pos: ChunkPos) -> Mesh {
    let mut builder = ChunkMeshBuilder::new();
    let origin = chunk_pos.world_origin();

    // Face directions: +X, -X, +Y, -Y, +Z, -Z
    let directions = [
        (IVec3::X, [1.0, 0.0, 0.0]),
        (IVec3::NEG_X, [-1.0, 0.0, 0.0]),
        (IVec3::Y, [0.0, 1.0, 0.0]),
        (IVec3::NEG_Y, [0.0, -1.0, 0.0]),
        (IVec3::Z, [0.0, 0.0, 1.0]),
        (IVec3::NEG_Z, [0.0, 0.0, -1.0]),
    ];

    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let kind = chunk.get(x, y, z);
                if kind == VoxelKind::Air {
                    continue;
                }

                let def = kind.def();
                let color = def.color.to_srgba();
                let base_color = [color.red, color.green, color.blue, color.alpha];

                let world_pos = origin + IVec3::new(x, y, z);

                for (dir, normal) in &directions {
                    let neighbor_pos = world_pos + *dir;
                    let neighbor = if neighbor_pos.x >= origin.x
                        && neighbor_pos.x < origin.x + CHUNK_SIZE
                        && neighbor_pos.z >= origin.z
                        && neighbor_pos.z < origin.z + CHUNK_SIZE
                    {
                        chunk.get(
                            neighbor_pos.x - origin.x,
                            neighbor_pos.y,
                            neighbor_pos.z - origin.z,
                        )
                    } else {
                        world.get_voxel(neighbor_pos)
                    };

                    // Only render face if neighbor is transparent (or air)
                    if !neighbor.is_transparent() {
                        continue;
                    }

                    // Skip water faces between water blocks
                    if kind == VoxelKind::Water && neighbor == VoxelKind::Water {
                        continue;
                    }

                    let vertices = get_face_vertices(x as f32, y as f32, z as f32, *dir);
                    builder.add_face(vertices, *normal, base_color);
                }
            }
        }
    }

    builder.build()
}

fn get_face_vertices(x: f32, y: f32, z: f32, dir: IVec3) -> [[f32; 3]; 4] {
    match (dir.x, dir.y, dir.z) {
        (1, 0, 0) => [
            [x + 1.0, y, z],
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z],
        ],
        (-1, 0, 0) => [
            [x, y, z + 1.0],
            [x, y, z],
            [x, y + 1.0, z],
            [x, y + 1.0, z + 1.0],
        ],
        (0, 1, 0) => [
            [x, y + 1.0, z],
            [x + 1.0, y + 1.0, z],
            [x + 1.0, y + 1.0, z + 1.0],
            [x, y + 1.0, z + 1.0],
        ],
        (0, -1, 0) => [
            [x, y, z + 1.0],
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y, z],
            [x, y, z],
        ],
        (0, 0, 1) => [
            [x + 1.0, y, z + 1.0],
            [x, y, z + 1.0],
            [x, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
        ],
        (0, 0, -1) => [
            [x, y, z],
            [x + 1.0, y, z],
            [x + 1.0, y + 1.0, z],
            [x, y + 1.0, z],
        ],
        _ => [[0.0; 3]; 4],
    }
}

// ============================================================================
// Components for Raycast
// ============================================================================

#[derive(Component, Debug, Clone, Copy)]
pub struct Voxel {
    pub kind: VoxelKind,
    pub pos: IVec3,
}

// ============================================================================
// Plugin & Systems
// ============================================================================

#[derive(Resource)]
pub struct ChunkMaterials {
    pub opaque: Handle<StandardMaterial>,
    pub transparent: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct ChunkLoadQueue {
    pub to_load: Vec<ChunkPos>,
    pub to_unload: Vec<ChunkPos>,
}

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>()
            .init_resource::<WorldSeed>()
            .init_resource::<ChunkLoadQueue>()
            .add_systems(Startup, setup_materials)
            .add_systems(Update, (update_chunk_loading, process_chunk_queue).chain());
    }
}

fn setup_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let opaque = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        ..default()
    });

    let transparent = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.3,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.insert_resource(ChunkMaterials { opaque, transparent });
}

fn update_chunk_loading(
    camera_query: Query<&Transform, With<Camera3d>>,
    world: Res<VoxelWorld>,
    mut queue: ResMut<ChunkLoadQueue>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_pos = camera_transform.translation;
    let center_chunk = ChunkPos::from_world_pos(camera_pos.x as i32, camera_pos.z as i32);

    // Find chunks to load
    for dx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let chunk_pos = ChunkPos::new(center_chunk.x + dx, center_chunk.z + dz);
            if !world.loaded_chunks.contains_key(&chunk_pos) && !queue.to_load.contains(&chunk_pos) {
                queue.to_load.push(chunk_pos);
            }
        }
    }

    // Find chunks to unload
    for &chunk_pos in world.loaded_chunks.keys() {
        let dx = (chunk_pos.x - center_chunk.x).abs();
        let dz = (chunk_pos.z - center_chunk.z).abs();
        if dx > RENDER_DISTANCE + 1 || dz > RENDER_DISTANCE + 1 {
            if !queue.to_unload.contains(&chunk_pos) {
                queue.to_unload.push(chunk_pos);
            }
        }
    }
}

fn process_chunk_queue(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<ChunkMaterials>,
    mut world: ResMut<VoxelWorld>,
    mut queue: ResMut<ChunkLoadQueue>,
    seed: Res<WorldSeed>,
) {
    // Unload chunks
    for chunk_pos in queue.to_unload.drain(..) {
        if let Some(entity) = world.loaded_chunks.remove(&chunk_pos) {
            commands.entity(entity).despawn();
        }
        world.chunks.remove(&chunk_pos);
    }

    // Load up to 2 chunks per frame
    let count = queue.to_load.len().min(2);
    let chunks_to_load: Vec<_> = queue.to_load.drain(..count).collect();

    for chunk_pos in chunks_to_load {
        let generator = TerrainGenerator::new(&seed);
        let chunk_data = generator.generate_chunk(chunk_pos);

        // Build mesh
        let mesh = build_chunk_mesh(&chunk_data, &world, chunk_pos);
        let mesh_handle = meshes.add(mesh);

        let origin = chunk_pos.world_origin();
        let entity = commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(materials.opaque.clone()),
                Transform::from_translation(Vec3::new(origin.x as f32, 0.0, origin.z as f32)),
                ChunkMarker { pos: chunk_pos },
            ))
            .id();

        world.chunks.insert(chunk_pos, chunk_data);
        world.loaded_chunks.insert(chunk_pos, entity);
    }
}

pub fn ivec3_to_vec3(pos: IVec3) -> Vec3 {
    Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32)
}
