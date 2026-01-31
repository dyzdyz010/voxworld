/// 方块变更记录
///
/// 用于 diff 日志，支持网络同步和存档
use super::flags::VoxelFlags;
use super::voxel_kind::VoxelKind;

/// 单个方块的变更操作
#[derive(Clone, Debug, PartialEq)]
pub enum BlockChange {
    /// 方块类型变化
    SetVoxel {
        idx: usize,
        old: VoxelKind,
        new: VoxelKind,
    },

    /// 标志位变化
    SetFlag {
        idx: usize,
        flag: VoxelFlags,
        set: bool,
    },

    /// 变体/阶段变化
    SetVariant { idx: usize, old: u8, new: u8 },

    /// 温度变化
    SetTemp { idx: usize, temp: f32 },

    /// 湿度变化
    SetMoisture { idx: usize, moisture: f32 },
}

impl BlockChange {
    /// 获取影响的方块索引
    pub fn idx(&self) -> usize {
        match self {
            BlockChange::SetVoxel { idx, .. } => *idx,
            BlockChange::SetFlag { idx, .. } => *idx,
            BlockChange::SetVariant { idx, .. } => *idx,
            BlockChange::SetTemp { idx, .. } => *idx,
            BlockChange::SetMoisture { idx, .. } => *idx,
        }
    }

    /// 判断是否需要重建网格
    pub fn needs_remesh(&self) -> bool {
        matches!(
            self,
            BlockChange::SetVoxel { .. } | BlockChange::SetVariant { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_idx() {
        let change = BlockChange::SetVoxel {
            idx: 42,
            old: VoxelKind::Air,
            new: VoxelKind::Stone,
        };
        assert_eq!(change.idx(), 42);
    }

    #[test]
    fn test_needs_remesh() {
        let change1 = BlockChange::SetVoxel {
            idx: 0,
            old: VoxelKind::Air,
            new: VoxelKind::Stone,
        };
        assert!(change1.needs_remesh());

        let change2 = BlockChange::SetTemp { idx: 0, temp: 100.0 };
        assert!(!change2.needs_remesh());
    }
}
