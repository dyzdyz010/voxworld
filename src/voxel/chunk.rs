//! 区块数据结构

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::voxel::change::BlockChange;
use crate::voxel::constants::CHUNK_SIZE;
use crate::voxel::domains::thermal::ThermalState;
use crate::voxel::flags::VoxelFlags;
use crate::voxel::voxel_kind::VoxelKind;

/// 区块坐标 - 用于标识世界中区块的位置
/// 注意：这是区块坐标，不是体素（方块）坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    /// 创建新的区块坐标
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// 从世界坐标（体素坐标）转换为区块坐标
    /// 使用欧几里德除法确保负坐标也能正确转换
    pub fn from_world_pos(world_x: i32, world_y: i32, world_z: i32) -> Self {
        Self {
            x: world_x.div_euclid(CHUNK_SIZE),
            y: world_y.div_euclid(CHUNK_SIZE),
            z: world_z.div_euclid(CHUNK_SIZE),
        }
    }

    /// 获取区块在世界坐标系中的起始位置（3D原点）
    pub fn world_origin(&self) -> IVec3 {
        IVec3::new(
            self.x * CHUNK_SIZE,
            self.y * CHUNK_SIZE,
            self.z * CHUNK_SIZE,
        )
    }

    /// 计算到另一个区块的曼哈顿距离（用于加载优先级排序）
    pub fn manhattan_distance_to(&self, other: &ChunkPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }

    /// 计算到另一个区块的欧几里德距离平方（用于渲染距离判断）
    pub fn distance_squared_to(&self, other: &ChunkPos) -> i32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }
}

/// 区块标记组件 - 用于标识游戏实体对应的区块位置
#[derive(Component)]
pub struct ChunkMarker {
    pub pos: ChunkPos,
}

/// 区块数据 - 存储区块内所有体素的类型数据和状态
pub struct ChunkData {
    // === 基础数据 ===
    /// 体素数组，大小为 CHUNK_SIZE³ = 4096
    /// 使用一维数组存储三维数据，通过 index() 函数计算索引
    pub voxels: Vec<VoxelKind>,
    /// 脏标记 - 标识区块是否被修改，需要重新生成网格
    pub is_dirty: bool,

    // === 通用状态（轻量，全量存储）===
    /// 每个方块的状态标志位
    pub flags: Vec<VoxelFlags>,
    /// 每个方块的变体/阶段（水位、生长阶段等，0-255）
    pub variant: Vec<u8>,

    // === 专用状态（稀疏，按需分配）===
    /// 温度场状态（稀疏存储）
    pub thermal_state: Option<ThermalState>,
    // TODO: 后续添加
    // pub moisture_state: Option<MoistureState>,
    // pub combustion_state: Option<CombustionState>,
    // pub phase_state: Option<PhaseState>,

    // === 活跃集合（驱动计算）===
    /// 活跃的温度变化方块索引
    pub active_thermal: HashSet<usize>,
    /// 正在燃烧的方块索引
    pub active_burning: HashSet<usize>,
    /// 正在冻结的方块索引
    pub active_freezing: HashSet<usize>,
    /// 正在融化的方块索引
    pub active_melting: HashSet<usize>,

    // === 渲染与同步 ===
    /// 变化的方块索引列表（用于增量更新）
    pub dirty_blocks: Vec<usize>,
    /// 需要重建网格
    pub needs_remesh: bool,
    /// 变更日志（用于网络同步/存档）
    pub changes: Vec<BlockChange>,
}

impl ChunkData {
    /// 区块体素总数
    pub const VOXEL_COUNT: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;

    /// 创建一个空的区块数据，所有体素初始化为空气
    pub fn new() -> Self {
        Self {
            voxels: vec![VoxelKind::Air; Self::VOXEL_COUNT],
            is_dirty: true,
            flags: vec![VoxelFlags::NONE; Self::VOXEL_COUNT],
            variant: vec![0; Self::VOXEL_COUNT],
            thermal_state: None,
            active_thermal: HashSet::new(),
            active_burning: HashSet::new(),
            active_freezing: HashSet::new(),
            active_melting: HashSet::new(),
            dirty_blocks: Vec::new(),
            needs_remesh: false,
            changes: Vec::new(),
        }
    }

    /// 清空变更日志
    pub fn clear_changes(&mut self) {
        self.dirty_blocks.clear();
        self.changes.clear();
        self.needs_remesh = false;
    }

    /// 获取活跃方块总数（所有领域）
    pub fn active_count(&self) -> usize {
        self.active_thermal.len()
            + self.active_burning.len()
            + self.active_freezing.len()
            + self.active_melting.len()
    }

    /// 将三维坐标转换为一维数组索引
    /// 使用Y-Z-X顺序进行线性化，便于按层遍历
    #[inline]
    pub fn index(x: i32, y: i32, z: i32) -> usize {
        ((y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x) as usize
    }

    /// 获取指定位置的体素类型
    /// 如果坐标超出边界，返回空气
    pub fn get(&self, x: i32, y: i32, z: i32) -> VoxelKind {
        if x < 0 || x >= CHUNK_SIZE || y < 0 || y >= CHUNK_SIZE || z < 0 || z >= CHUNK_SIZE {
            return VoxelKind::Air;
        }
        self.voxels[Self::index(x, y, z)]
    }

    /// 设置指定位置的体素类型
    /// 如果坐标超出边界，不执行操作
    /// 设置后会标记区块为脏，需要重新生成网格
    pub fn set(&mut self, x: i32, y: i32, z: i32, kind: VoxelKind) {
        if x < 0 || x >= CHUNK_SIZE || y < 0 || y >= CHUNK_SIZE || z < 0 || z >= CHUNK_SIZE {
            return;
        }
        self.voxels[Self::index(x, y, z)] = kind;
        self.is_dirty = true;
    }

    /// 检查区块是否完全为空气
    /// 用于优化：空气区块不需要生成网格
    pub fn is_empty(&self) -> bool {
        self.voxels.iter().all(|&kind| kind == VoxelKind::Air)
    }

    /// 检查区块是否完全不透明（所有体素都是实心方块）
    /// 用于优化：完全被包围的不透明区块不需要生成网格
    pub fn is_fully_opaque(&self) -> bool {
        self.voxels.iter().all(|&kind| !kind.is_transparent())
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ChunkData {
    fn clone(&self) -> Self {
        Self {
            voxels: self.voxels.clone(),
            is_dirty: self.is_dirty,
            flags: self.flags.clone(),
            variant: self.variant.clone(),
            thermal_state: self.thermal_state.clone(),
            active_thermal: self.active_thermal.clone(),
            active_burning: self.active_burning.clone(),
            active_freezing: self.active_freezing.clone(),
            active_melting: self.active_melting.clone(),
            dirty_blocks: self.dirty_blocks.clone(),
            needs_remesh: self.needs_remesh,
            changes: self.changes.clone(),
        }
    }
}

/// 体素世界 - 管理整个世界的所有区块
#[derive(Resource, Default)]
pub struct VoxelWorld {
    /// 存储所有已生成的区块数据
    pub chunks: HashMap<ChunkPos, ChunkData>,
    /// 存储已加载区块对应的实体ID，用于场景管理
    pub loaded_chunks: HashMap<ChunkPos, Entity>,
}

impl VoxelWorld {
    /// 获取世界中指定位置的体素类型
    /// 自动将世界坐标转换为区块坐标和局部坐标
    pub fn get_voxel(&self, world_pos: IVec3) -> VoxelKind {
        let chunk_pos = ChunkPos::from_world_pos(world_pos.x, world_pos.y, world_pos.z);
        let local_x = world_pos.x.rem_euclid(CHUNK_SIZE);
        let local_y = world_pos.y.rem_euclid(CHUNK_SIZE);
        let local_z = world_pos.z.rem_euclid(CHUNK_SIZE);

        self.chunks
            .get(&chunk_pos)
            .map(|chunk| chunk.get(local_x, local_y, local_z))
            .unwrap_or(VoxelKind::Air)
    }

    /// 设置世界中指定位置的体素类型
    /// 如果对应的区块不存在，不执行操作
    pub fn set_voxel(&mut self, world_pos: IVec3, kind: VoxelKind) {
        let chunk_pos = ChunkPos::from_world_pos(world_pos.x, world_pos.y, world_pos.z);
        let local_x = world_pos.x.rem_euclid(CHUNK_SIZE);
        let local_y = world_pos.y.rem_euclid(CHUNK_SIZE);
        let local_z = world_pos.z.rem_euclid(CHUNK_SIZE);

        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set(local_x, local_y, local_z, kind);
        }
    }
}
