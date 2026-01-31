//! 热力学领域 API
//!
//! 提供对温度场的读写操作接口

use super::state::ThermalState;
use crate::voxel::change::BlockChange;
use crate::voxel::chunk::ChunkData;
use crate::voxel::flags::VoxelFlags;

/// 温度阈值常量
pub const TEMP_HOT_THRESHOLD: f32 = 100.0; // 高温阈值 100°C
pub const TEMP_COLD_THRESHOLD: f32 = 0.0; // 低温阈值 0°C
pub const TEMP_EPSILON: f32 = 0.1; // 温度变化忽略阈值
pub const GRADIENT_THRESHOLD: f32 = 5.0; // 温度梯度阈值（判断是否保持活跃）

/// 热力学领域 API
pub struct ThermalApi;

impl ThermalApi {
    /// 获取方块温度（考虑默认值和覆盖值）
    pub fn get_temp(chunk: &ChunkData, idx: usize) -> f32 {
        // 优先检查覆盖值
        if let Some(thermal) = &chunk.thermal_state {
            if let Some(&temp) = thermal.temp_overrides.get(&idx) {
                return temp;
            }
        }
        // 回退到默认值
        chunk.voxels[idx].def().props.temperature
    }

    /// 设置方块温度
    ///
    /// 会自动：
    /// 1. 激活到活跃集合
    /// 2. 更新温度相关标志位（HOT/COLD）
    /// 3. 记录变更日志
    pub fn set_temp(chunk: &mut ChunkData, idx: usize, temp: f32) {
        let default_temp = chunk.voxels[idx].def().props.temperature;

        // 延迟分配稀疏状态
        let thermal = chunk.thermal_state.get_or_insert_with(ThermalState::default);

        // 如果接近默认值，移除覆盖
        if (temp - default_temp).abs() < TEMP_EPSILON {
            thermal.temp_overrides.remove(&idx);
        } else {
            thermal.temp_overrides.insert(idx, temp);
            chunk.active_thermal.insert(idx);
        }

        // 更新温度相关标志位
        Self::update_temp_flags(chunk, idx, temp);

        // 记录变更
        chunk.changes.push(BlockChange::SetTemp { idx, temp });
        chunk.dirty_blocks.push(idx);
    }

    /// 添加热量（用于燃烧、传导等）
    ///
    /// 根据方块的热容计算温度变化: ΔT = Q / C
    pub fn add_heat(chunk: &mut ChunkData, idx: usize, heat: f32) {
        let current_temp = Self::get_temp(chunk, idx);
        let heat_capacity = chunk.voxels[idx].def().props.heat_capacity;

        // 防止除零
        if heat_capacity <= 0.0 {
            return;
        }

        let delta_temp = heat / heat_capacity;
        Self::set_temp(chunk, idx, current_temp + delta_temp);
    }

    /// 更新温度相关标志位
    fn update_temp_flags(chunk: &mut ChunkData, idx: usize, temp: f32) {
        // 高温标志
        if temp > TEMP_HOT_THRESHOLD {
            if !chunk.flags[idx].contains(VoxelFlags::HOT) {
                chunk.flags[idx].insert(VoxelFlags::HOT);
                chunk.changes.push(BlockChange::SetFlag {
                    idx,
                    flag: VoxelFlags::HOT,
                    set: true,
                });
            }
        } else if chunk.flags[idx].contains(VoxelFlags::HOT) {
            chunk.flags[idx].remove(VoxelFlags::HOT);
            chunk.changes.push(BlockChange::SetFlag {
                idx,
                flag: VoxelFlags::HOT,
                set: false,
            });
        }

        // 低温标志
        if temp < TEMP_COLD_THRESHOLD {
            if !chunk.flags[idx].contains(VoxelFlags::COLD) {
                chunk.flags[idx].insert(VoxelFlags::COLD);
                chunk.changes.push(BlockChange::SetFlag {
                    idx,
                    flag: VoxelFlags::COLD,
                    set: true,
                });
            }
        } else if chunk.flags[idx].contains(VoxelFlags::COLD) {
            chunk.flags[idx].remove(VoxelFlags::COLD);
            chunk.changes.push(BlockChange::SetFlag {
                idx,
                flag: VoxelFlags::COLD,
                set: false,
            });
        }
    }

    /// 判断是否应保持活跃
    ///
    /// 活跃条件：
    /// 1. 温度偏离默认值
    /// 2. 附近有热源（燃烧方块）
    /// 3. 存在温度梯度
    pub fn should_stay_active(chunk: &ChunkData, idx: usize) -> bool {
        let temp = Self::get_temp(chunk, idx);
        let default_temp = chunk.voxels[idx].def().props.temperature;

        // 条件 1：温度偏离默认值
        if (temp - default_temp).abs() > 1.0 {
            return true;
        }

        // 条件 2：正在燃烧
        if chunk.flags[idx].contains(VoxelFlags::BURNING) {
            return true;
        }

        // 条件 3：周围有温度梯度
        for neighbor_idx in get_neighbor_indices(idx) {
            if neighbor_idx >= chunk.voxels.len() {
                continue;
            }

            // 邻居在燃烧
            if chunk.flags[neighbor_idx].contains(VoxelFlags::BURNING) {
                return true;
            }

            // 温度梯度
            let neighbor_temp = Self::get_temp(chunk, neighbor_idx);
            if (neighbor_temp - temp).abs() > GRADIENT_THRESHOLD {
                return true;
            }
        }

        false
    }

    /// 激活一个方块到温度活跃集合
    ///
    /// 同时激活其邻居（因为扩散需要）
    pub fn activate(chunk: &mut ChunkData, idx: usize) {
        chunk.active_thermal.insert(idx);

        // 也激活邻居
        for neighbor_idx in get_neighbor_indices(idx) {
            if neighbor_idx < chunk.voxels.len() {
                chunk.active_thermal.insert(neighbor_idx);
            }
        }
    }

    /// 尝试从活跃集合中移除
    ///
    /// 仅当方块不再需要活跃时移除
    pub fn try_deactivate(chunk: &mut ChunkData, idx: usize) {
        if !Self::should_stay_active(chunk, idx) {
            chunk.active_thermal.remove(&idx);
        }
    }
}

/// 获取 3D 坐标的 6 个邻居索引
///
/// 使用 CHUNK_SIZE = 16 的 Y-Z-X 线性化顺序
pub fn get_neighbor_indices(idx: usize) -> [usize; 6] {
    const SIZE: usize = 16;
    const LAYER: usize = SIZE * SIZE; // 256

    // 注意：这里简化处理，不检查边界
    // 实际应用中，调用方需要检查返回的索引是否有效
    [
        idx.wrapping_sub(1),       // -X
        idx.wrapping_add(1),       // +X
        idx.wrapping_sub(SIZE),    // -Z
        idx.wrapping_add(SIZE),    // +Z
        idx.wrapping_sub(LAYER),   // -Y
        idx.wrapping_add(LAYER),   // +Y
    ]
}

/// 从一维索引提取 3D 坐标
pub fn idx_to_xyz(idx: usize) -> (i32, i32, i32) {
    const SIZE: i32 = 16;
    let idx = idx as i32;
    let y = idx / (SIZE * SIZE);
    let remainder = idx % (SIZE * SIZE);
    let z = remainder / SIZE;
    let x = remainder % SIZE;
    (x, y, z)
}

/// 从 3D 坐标计算一维索引
pub fn xyz_to_idx(x: i32, y: i32, z: i32) -> usize {
    const SIZE: i32 = 16;
    ((y * SIZE * SIZE) + (z * SIZE) + x) as usize
}

/// 检查坐标是否在 chunk 边界内
pub fn is_in_bounds(x: i32, y: i32, z: i32) -> bool {
    const SIZE: i32 = 16;
    x >= 0 && x < SIZE && y >= 0 && y < SIZE && z >= 0 && z < SIZE
}

/// 获取有效的邻居索引列表（过滤边界）
pub fn get_valid_neighbor_indices(idx: usize) -> Vec<usize> {
    let (x, y, z) = idx_to_xyz(idx);
    let mut neighbors = Vec::with_capacity(6);

    // -X
    if x > 0 {
        neighbors.push(xyz_to_idx(x - 1, y, z));
    }
    // +X
    if x < 15 {
        neighbors.push(xyz_to_idx(x + 1, y, z));
    }
    // -Z
    if z > 0 {
        neighbors.push(xyz_to_idx(x, y, z - 1));
    }
    // +Z
    if z < 15 {
        neighbors.push(xyz_to_idx(x, y, z + 1));
    }
    // -Y
    if y > 0 {
        neighbors.push(xyz_to_idx(x, y - 1, z));
    }
    // +Y
    if y < 15 {
        neighbors.push(xyz_to_idx(x, y + 1, z));
    }

    neighbors
}
