//! 温度场状态定义
//!
//! 使用稀疏存储，只存储偏离默认温度的方块

use std::collections::HashMap;

/// 温度场状态（稀疏存储）
///
/// 只存储偏离默认温度（VoxelProperties.temperature）的方块
#[derive(Debug, Default, Clone)]
pub struct ThermalState {
    /// 温度覆盖值 (idx -> 摄氏度)
    ///
    /// 默认值从 VoxelKind.def().props.temperature 获取
    pub temp_overrides: HashMap<usize, f32>,

    /// 热能变化缓冲（用于扩散计算的中间值）
    ///
    /// 存储每个 tick 需要施加的热量 delta
    pub heat_buffer: HashMap<usize, f32>,
}

impl ThermalState {
    /// 创建新的温度场状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查是否为空（没有温度覆盖）
    pub fn is_empty(&self) -> bool {
        self.temp_overrides.is_empty()
    }

    /// 获取活跃方块数量
    pub fn active_count(&self) -> usize {
        self.temp_overrides.len()
    }

    /// 清除所有温度覆盖
    pub fn clear(&mut self) {
        self.temp_overrides.clear();
        self.heat_buffer.clear();
    }
}
