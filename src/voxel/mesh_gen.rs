//! 异步网格生成

use bevy::prelude::*;
use std::sync::Arc;

use crate::voxel::chunk::{ChunkData, ChunkPos};
use crate::voxel::constants::{CHUNK_HEIGHT, CHUNK_SIZE};
use crate::voxel::loading::{MeshBuildInput, NeighborEdges};
use crate::voxel::mesh::{get_face_vertices, ChunkMeshBuilder, MESH_BUFFERS};
use crate::voxel::seed::WorldSeed;
use crate::voxel::terrain::TerrainGenerator;
use crate::voxel::voxel_kind::VoxelKind;

/// 在工作线程中生成区块数据并构建网格
/// 包含地形生成和网格构建两个阶段
pub fn generate_chunk_and_mesh_async(chunk_pos: ChunkPos, seed: u32) -> (Vec<VoxelKind>, Mesh) {
    // 阶段1：生成区块地形数据
    let world_seed = WorldSeed::new(seed);
    let generator = TerrainGenerator::new(&world_seed);
    let chunk_data = generator.generate_chunk(chunk_pos);
    let voxels = chunk_data.voxels.clone();

    // 阶段2：构建网格（需要相邻区块数据，但首次生成时使用空边界）
    let input = MeshBuildInput {
        chunk_pos,
        voxels: Arc::new(voxels.clone()),
        neighbor_edges: NeighborEdges::default(),
    };

    let mesh = build_chunk_mesh_async(input);

    (voxels, mesh)
}

/// 在工作线程中构建区块网格
/// 使用线程本地缓冲区和顶点去重优化
pub fn build_chunk_mesh_async(input: MeshBuildInput) -> Mesh {
    MESH_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let mut builder = ChunkMeshBuilder::with_buffers(&mut buffers);

        // 6个面的方向和法线
        let directions: [(IVec3, [f32; 3]); 6] = [
            (IVec3::X, [1.0, 0.0, 0.0]),
            (IVec3::NEG_X, [-1.0, 0.0, 0.0]),
            (IVec3::Y, [0.0, 1.0, 0.0]),
            (IVec3::NEG_Y, [0.0, -1.0, 0.0]),
            (IVec3::Z, [0.0, 0.0, 1.0]),
            (IVec3::NEG_Z, [0.0, 0.0, -1.0]),
        ];

        // 遍历区块中的所有体素
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let index = ChunkData::index(x, y, z);
                    let kind = input.voxels[index];

                    if kind == VoxelKind::Air {
                        continue;
                    }

                    let def = kind.def();
                    let color = def.color.to_srgba();
                    let base_color = [color.red, color.green, color.blue, color.alpha];
                    let local_pos = IVec3::new(x, y, z);

                    // 检查每个面
                    for (dir, normal) in &directions {
                        let neighbor_local = local_pos + *dir;

                        // 判断相邻位置是否在区块内
                        let neighbor = if neighbor_local.x >= 0
                            && neighbor_local.x < CHUNK_SIZE
                            && neighbor_local.y >= 0
                            && neighbor_local.y < CHUNK_HEIGHT
                            && neighbor_local.z >= 0
                            && neighbor_local.z < CHUNK_SIZE
                        {
                            // 区块内部查询
                            let ni = ChunkData::index(
                                neighbor_local.x,
                                neighbor_local.y,
                                neighbor_local.z,
                            );
                            input.voxels[ni]
                        } else if neighbor_local.y < 0 || neighbor_local.y >= CHUNK_HEIGHT {
                            // Y方向超出世界边界
                            VoxelKind::Air
                        } else {
                            // 查询相邻区块边界
                            input
                                .neighbor_edges
                                .get_neighbor(local_pos, *dir)
                                .unwrap_or(VoxelKind::Air)
                        };

                        // 只渲染暴露的面
                        if !neighbor.is_transparent() {
                            continue;
                        }

                        // 水面之间不渲染
                        if kind == VoxelKind::Water && neighbor == VoxelKind::Water {
                            continue;
                        }

                        let vertices = get_face_vertices(x as f32, y as f32, z as f32, *dir);
                        builder.add_face_deduplicated(vertices, *normal, base_color);
                    }
                }
            }
        }

        builder.build()
    })
}
