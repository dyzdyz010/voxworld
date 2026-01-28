//! 异步加载系统的数据类型

use bevy::prelude::*;
use bevy::tasks::Task;
use std::collections::HashMap;
use std::sync::Arc;

use crate::voxel::chunk::{ChunkData, ChunkPos, VoxelWorld};
use crate::voxel::constants::CHUNK_SIZE;
use crate::voxel::voxel_kind::VoxelKind;

// ============================================================================
// 相邻区块边界数据
// ============================================================================

/// 相邻区块边界数据 - 用于跨区块面剔除
#[derive(Clone, Default)]
pub struct NeighborEdges {
    /// +X方向相邻区块的X=0面
    pub pos_x: Option<Vec<VoxelKind>>,
    /// -X方向相邻区块的X=CHUNK_SIZE-1面
    pub neg_x: Option<Vec<VoxelKind>>,
    /// +Z方向相邻区块的Z=0面
    pub pos_z: Option<Vec<VoxelKind>>,
    /// -Z方向相邻区块的Z=CHUNK_SIZE-1面
    pub neg_z: Option<Vec<VoxelKind>>,
}

impl NeighborEdges {
    /// 从相邻区块提取边界数据
    pub fn from_world(world: &VoxelWorld, center: ChunkPos) -> Self {
        Self {
            pos_x: world
                .chunks
                .get(&ChunkPos::new(center.x + 1, center.z))
                .map(|c| Self::extract_x_face(c, 0)),
            neg_x: world
                .chunks
                .get(&ChunkPos::new(center.x - 1, center.z))
                .map(|c| Self::extract_x_face(c, CHUNK_SIZE - 1)),
            pos_z: world
                .chunks
                .get(&ChunkPos::new(center.x, center.z + 1))
                .map(|c| Self::extract_z_face(c, 0)),
            neg_z: world
                .chunks
                .get(&ChunkPos::new(center.x, center.z - 1))
                .map(|c| Self::extract_z_face(c, CHUNK_SIZE - 1)),
        }
    }

    fn extract_x_face(chunk: &ChunkData, x: i32) -> Vec<VoxelKind> {
        use crate::voxel::constants::CHUNK_HEIGHT;
        let mut face = Vec::with_capacity((CHUNK_HEIGHT * CHUNK_SIZE) as usize);
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                face.push(chunk.get(x, y, z));
            }
        }
        face
    }

    fn extract_z_face(chunk: &ChunkData, z: i32) -> Vec<VoxelKind> {
        use crate::voxel::constants::CHUNK_HEIGHT;
        let mut face = Vec::with_capacity((CHUNK_HEIGHT * CHUNK_SIZE) as usize);
        for y in 0..CHUNK_HEIGHT {
            for x in 0..CHUNK_SIZE {
                face.push(chunk.get(x, y, z));
            }
        }
        face
    }

    /// 获取指定位置的相邻体素
    pub fn get_neighbor(&self, local_pos: IVec3, dir: IVec3) -> Option<VoxelKind> {
        match (dir.x, dir.y, dir.z) {
            (1, 0, 0) if local_pos.x == CHUNK_SIZE - 1 => self
                .pos_x
                .as_ref()
                .map(|f| f[(local_pos.y * CHUNK_SIZE + local_pos.z) as usize]),
            (-1, 0, 0) if local_pos.x == 0 => self
                .neg_x
                .as_ref()
                .map(|f| f[(local_pos.y * CHUNK_SIZE + local_pos.z) as usize]),
            (0, 0, 1) if local_pos.z == CHUNK_SIZE - 1 => self
                .pos_z
                .as_ref()
                .map(|f| f[(local_pos.y * CHUNK_SIZE + local_pos.x) as usize]),
            (0, 0, -1) if local_pos.z == 0 => self
                .neg_z
                .as_ref()
                .map(|f| f[(local_pos.y * CHUNK_SIZE + local_pos.x) as usize]),
            _ => None,
        }
    }
}

// ============================================================================
// 异步任务类型
// ============================================================================

/// 异步网格生成任务输入
#[derive(Clone)]
pub struct MeshBuildInput {
    /// 区块位置
    pub chunk_pos: ChunkPos,
    /// 体素数据的副本
    pub voxels: Arc<Vec<VoxelKind>>,
    /// 相邻区块的边界体素数据
    pub neighbor_edges: NeighborEdges,
}

/// 正在进行的网格生成任务
#[derive(Component)]
pub struct ComputeMeshTask {
    /// 异步任务句柄（包含区块生成和网格构建）
    pub task: Task<(Vec<VoxelKind>, Mesh)>,
    /// 区块位置
    pub chunk_pos: ChunkPos,
    /// 占位符实体ID（生成完成后需要替换）
    pub placeholder_entity: Entity,
}

// ============================================================================
// 加载队列和缓冲区
// ============================================================================

/// 区块加载队列 - 管理需要加载和卸载的区块
#[derive(Resource)]
pub struct ChunkLoadQueue {
    /// 待加载的区块列表
    pub to_load: Vec<ChunkPos>,
    /// 待卸载的区块列表
    pub to_unload: Vec<ChunkPos>,
    /// 当前活跃的异步任务数量
    pub active_tasks: usize,
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 待批量创建占位符的区块（新加入队列的区块）
    pub pending_placeholders: Vec<ChunkPos>,
}

impl Default for ChunkLoadQueue {
    fn default() -> Self {
        Self {
            to_load: Vec::new(),
            to_unload: Vec::new(),
            active_tasks: 0,
            max_concurrent_tasks: 16, // 优化: 从64降低到16，减少线程竞争和CPU压力
            pending_placeholders: Vec::new(),
        }
    }
}

/// 占位符实体映射 - 保存每个区块位置对应的占位符实体
#[derive(Resource, Default)]
pub struct PlaceholderEntities {
    pub map: HashMap<ChunkPos, Entity>,
}

/// 完成的区块数据（等待批量替换）
pub struct CompletedChunk {
    pub chunk_pos: ChunkPos,
    pub voxels: Vec<VoxelKind>,
    pub mesh: Mesh,
    pub placeholder_entity: Entity,
}

/// 批量替换缓冲区 - 收集完成的区块，批量替换占位符
#[derive(Resource)]
pub struct ChunkReplacementBuffer {
    /// 完成的区块数据
    pub completed: Vec<CompletedChunk>,
    /// 批量替换定时器（秒）
    pub timer: f32,
    /// 批量替换间隔（秒）
    pub interval: f32,
    /// 最小批量大小
    pub min_batch_size: usize,
}

impl Default for ChunkReplacementBuffer {
    fn default() -> Self {
        Self {
            completed: Vec::new(),
            timer: 0.0,
            interval: 0.3, // 每0.3秒批量替换一次
            min_batch_size: 3, // 至少3个区块一起替换
        }
    }
}
