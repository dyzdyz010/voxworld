//! 体素系统插件

use bevy::prelude::*;

use crate::voxel::chunk::VoxelWorld;
use crate::voxel::domains::DomainPlugin;
use crate::voxel::loading::{ChunkLoadQueue, ChunkReplacementBuffer, PlaceholderEntities};
use crate::voxel::materials::setup_materials;
use crate::voxel::seed::WorldSeed;
use crate::voxel::systems::{
    apply_chunk_replacements, cleanup_orphan_placeholders, handle_completed_mesh_tasks,
    process_chunk_unload, spawn_batch_placeholders, spawn_mesh_tasks, update_chunk_loading,
};

/// 体素系统插件 - 负责注册体素相关的资源和系统
pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>()
            .init_resource::<WorldSeed>()
            .init_resource::<ChunkLoadQueue>()
            .init_resource::<ChunkReplacementBuffer>()
            .init_resource::<PlaceholderEntities>()
            .add_systems(Startup, setup_materials)
            .add_systems(
                Update,
                (
                    update_chunk_loading,
                    spawn_batch_placeholders,
                    spawn_mesh_tasks,
                    handle_completed_mesh_tasks,
                    apply_chunk_replacements,
                    process_chunk_unload,
                    cleanup_orphan_placeholders,
                )
                    .chain(),
            )
            // 注册领域系统（温度、湿度、燃烧等物理模拟）
            .add_plugins(DomainPlugin);
    }
}
