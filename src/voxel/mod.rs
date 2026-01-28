//! 体素世界模块
//!
//! 这个模块包含了体素游戏的核心系统，包括：
//!
//! - **constants**: 常量定义（区块大小、渲染距离等）
//! - **voxel_kind**: 体素类型定义（方块种类、属性、颜色）
//! - **biome**: 生物群系（平原、森林、沙漠等）
//! - **seed**: 世界种子与噪声生成器
//! - **chunk**: 区块数据结构（区块坐标、体素存储、世界管理）
//! - **terrain**: 地形生成器（程序化地形、洞穴、矿石、树木）
//! - **mesh**: 网格构建（顶点去重、面剔除、占位符）
//! - **loading**: 异步加载类型（任务队列、缓冲区）
//! - **systems**: ECS系统函数（区块加载、卸载、渲染）
//! - **materials**: 材质系统（不透明/透明材质）
//! - **components**: 体素相关组件
//! - **plugin**: Bevy插件

pub mod biome;
pub mod chunk;
pub mod components;
pub mod constants;
pub mod loading;
pub mod materials;
pub mod mesh;
pub mod mesh_gen;
pub mod plugin;
pub mod seed;
pub mod systems;
pub mod terrain;
pub mod voxel_kind;

// 重新导出常用类型，方便外部使用
pub use biome::Biome;
pub use chunk::{ChunkData, ChunkMarker, ChunkPos, VoxelWorld};
pub use components::Voxel;
pub use constants::{CHUNK_SIZE, RENDER_DISTANCE, VERTICAL_RENDER_DISTANCE};
pub use loading::{
    ChunkLoadQueue, ChunkReplacementBuffer, CompletedChunk, ComputeMeshTask, MeshBuildInput,
    NeighborEdges, PlaceholderEntities,
};
pub use materials::ChunkMaterials;
pub use mesh::create_placeholder_mesh;
pub use mesh_gen::{build_chunk_mesh_async, generate_chunk_and_mesh_async};
pub use plugin::VoxelPlugin;
pub use seed::WorldSeed;
pub use terrain::TerrainGenerator;
pub use voxel_kind::{VoxelDef, VoxelKind, VoxelProperties};

// ============================================================================
// 辅助函数
// ============================================================================

use bevy::prelude::*;

/// 将整数向量转换为浮点向量（辅助函数）
pub fn ivec3_to_vec3(pos: IVec3) -> Vec3 {
    Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32)
}
