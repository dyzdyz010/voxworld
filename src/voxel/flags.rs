/// 方块状态标志位
///
/// 使用 bitflags 实现高效的状态存储，每个 Chunk 中的每个方块都有一个 VoxelFlags
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VoxelFlags: u16 {
        const NONE = 0;

        // === 燃烧相关 ===
        /// 正在燃烧
        const BURNING = 1 << 0;
        /// 已焦化
        const CHARRED = 1 << 1;
        /// 闷烧中（无明火但仍在燃烧）
        const SMOLDERING = 1 << 2;

        // === 温度相关 ===
        /// 高温（>100°C）
        const HOT = 1 << 3;
        /// 低温（<0°C）
        const COLD = 1 << 4;

        // === 湿度相关 ===
        /// 潮湿（湿度 > 0.5）
        const WET = 1 << 5;
        /// 浸透（湿度 > 0.9）
        const SOAKED = 1 << 6;
        /// 冻结
        const FROZEN = 1 << 7;

        // === 状态相关 ===
        /// 融化中
        const MELTING = 1 << 8;
        /// 蒸发中
        const EVAPORATING = 1 << 9;
        /// 凝结中
        const CONDENSING = 1 << 10;

        // === 结构相关 ===
        /// 损坏
        const DAMAGED = 1 << 11;
        /// 不稳定（将塌落）
        const UNSTABLE = 1 << 12;
        /// 腐蚀
        const CORRODED = 1 << 13;

        // === 生物相关 ===
        /// 生长中
        const GROWING = 1 << 14;
        /// 枯萎中
        const WITHERING = 1 << 15;
    }
}

impl Default for VoxelFlags {
    fn default() -> Self {
        Self::NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags_basic() {
        let mut flags = VoxelFlags::NONE;
        assert!(!flags.contains(VoxelFlags::BURNING));

        flags.insert(VoxelFlags::BURNING);
        assert!(flags.contains(VoxelFlags::BURNING));

        flags.remove(VoxelFlags::BURNING);
        assert!(!flags.contains(VoxelFlags::BURNING));
    }

    #[test]
    fn test_flags_combination() {
        let flags = VoxelFlags::WET | VoxelFlags::COLD;
        assert!(flags.contains(VoxelFlags::WET));
        assert!(flags.contains(VoxelFlags::COLD));
        assert!(!flags.contains(VoxelFlags::BURNING));
    }
}
