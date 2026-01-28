//! 世界种子与噪声生成器

use bevy::prelude::*;
use noise::Perlin;

/// 世界种子 - 存储世界生成的随机种子和各种噪声生成器
/// 使用相同的种子可以生成相同的世界
#[derive(Resource)]
pub struct WorldSeed {
    /// 主种子值
    pub seed: u32,
    /// 地形高度噪声生成器
    pub terrain_noise: Perlin,
    /// 生物群系温度噪声生成器
    pub biome_temp_noise: Perlin,
    /// 生物群系湿度噪声生成器
    pub biome_humid_noise: Perlin,
    /// 洞穴生成噪声生成器
    pub cave_noise: Perlin,
    /// 细节噪声生成器（用于矿石、树木等）
    pub detail_noise: Perlin,
}

impl WorldSeed {
    /// 从数字种子创建世界种子
    /// 为不同的噪声生成器分配不同的种子偏移，确保各种噪声独立
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

    /// 从字符串创建世界种子
    /// 通过简单的哈希算法将字符串转换为数字种子
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
