//! 地形生成器

use noise::NoiseFn;

use crate::voxel::biome::Biome;
use crate::voxel::chunk::{ChunkData, ChunkPos};
use crate::voxel::constants::{CHUNK_HEIGHT, CHUNK_SIZE};
use crate::voxel::seed::WorldSeed;
use crate::voxel::voxel_kind::VoxelKind;

/// 地形生成器 - 使用程序化生成算法创建地形
/// 基于柏林噪声（Perlin Noise）生成自然的地形特征
pub struct TerrainGenerator<'a> {
    seed: &'a WorldSeed,
}

impl<'a> TerrainGenerator<'a> {
    /// 创建新的地形生成器
    pub fn new(seed: &'a WorldSeed) -> Self {
        Self { seed }
    }

    /// 计算指定位置的地形高度
    /// 使用多层噪声叠加（分形噪声）生成更自然的地形
    /// - 第一层：大尺度地形特征（山脉、山谷）
    /// - 第二层：中等尺度起伏
    /// - 第三层：小尺度细节
    pub fn get_height(&self, x: i32, z: i32) -> i32 {
        let scale = 0.02;
        let fx = x as f64 * scale;
        let fz = z as f64 * scale;

        let mut height = 0.0;
        // 大尺度地形
        height += self.seed.terrain_noise.get([fx, fz]) * 12.0;
        // 中等尺度起伏
        height += self.seed.terrain_noise.get([fx * 2.0, fz * 2.0]) * 6.0;
        // 小尺度细节
        height += self.seed.detail_noise.get([fx * 4.0, fz * 4.0]) * 3.0;

        let base_height = 32;
        ((base_height as f64 + height) as i32).clamp(1, CHUNK_HEIGHT - 10)
    }

    /// 根据温度、湿度和高度确定生物群系类型
    /// 使用噪声函数生成温度和湿度图，模拟真实的气候分布
    pub fn get_biome(&self, x: i32, z: i32) -> Biome {
        let scale = 0.008;
        let fx = x as f64 * scale;
        let fz = z as f64 * scale;

        // 获取温度和湿度值（范围：-1.0 到 1.0）
        let temp = self.seed.biome_temp_noise.get([fx, fz]);
        let humid = self.seed.biome_humid_noise.get([fx, fz]);
        let height = self.get_height(x, z);

        // 低海拔地区为海洋
        if height < 28 {
            return Biome::Ocean;
        }
        // 海拔稍高的地区为海滩
        if height < 32 {
            return Biome::Beach;
        }

        // 根据温度和湿度确定生物群系
        match (temp, humid) {
            // 低温地区
            (t, _) if t < -0.3 => {
                if humid > 0.2 {
                    Biome::Taiga // 湿润的寒带 → 针叶林
                } else {
                    Biome::Snowy // 干燥的寒带 → 雪地
                }
            }
            // 高温低湿 → 沙漠
            (t, h) if t > 0.3 && h < -0.2 => Biome::Desert,
            // 高湿度 → 森林
            (_, h) if h > 0.3 => Biome::Forest,
            // 中等湿度 → 白桦林
            (_, h) if h > 0.0 => Biome::BirchForest,
            // 默认 → 平原
            _ => Biome::Plains,
        }
    }

    /// 判断指定位置是否应该生成洞穴
    /// 使用3D噪声生成自然的洞穴系统
    /// 洞穴只在Y=5到Y=40的范围内生成
    pub fn is_cave(&self, x: i32, y: i32, z: i32) -> bool {
        if y > 40 || y < 5 {
            return false;
        }
        let scale = 0.08;
        let value = self
            .seed
            .cave_noise
            .get([x as f64 * scale, y as f64 * scale, z as f64 * scale]);
        value > 0.55
    }

    /// 根据位置和深度生成矿石
    /// 越深的地方生成越稀有的矿石
    /// - 钻石矿：Y < 16，最稀有
    /// - 金矿：Y < 32，稀有
    /// - 铁矿：Y < 48，常见
    /// - 煤矿：Y >= 48，最常见
    pub fn get_ore(&self, x: i32, y: i32, z: i32) -> Option<VoxelKind> {
        let scale = 0.15;
        let noise = self
            .seed
            .detail_noise
            .get([x as f64 * scale, y as f64 * scale, z as f64 * scale]);

        if noise > 0.7 {
            if y < 16 && noise > 0.85 {
                Some(VoxelKind::DiamondOre)
            } else if y < 32 && noise > 0.80 {
                Some(VoxelKind::GoldOre)
            } else if y < 48 {
                Some(VoxelKind::IronOre)
            } else {
                Some(VoxelKind::CoalOre)
            }
        } else {
            None
        }
    }

    /// 判断指定位置是否应该放置树木
    /// 不同生物群系有不同的树木生成概率
    pub fn should_place_tree(&self, x: i32, z: i32, biome: Biome) -> bool {
        // 根据生物群系设置树木生成概率
        let tree_chance = match biome {
            Biome::Forest | Biome::Taiga => 0.06, // 森林和针叶林：6%
            Biome::BirchForest => 0.04,           // 白桦林：4%
            Biome::Plains => 0.003,               // 平原：0.3%
            _ => 0.0,                             // 其他生物群系不生成树木
        };

        if tree_chance == 0.0 {
            return false;
        }

        // 使用噪声函数随机决定是否生成树木
        let scale = 0.5;
        let noise = self.seed.detail_noise.get([x as f64 * scale, z as f64 * scale]);
        noise > (1.0 - tree_chance * 2.0)
    }

    /// 生成指定区块的完整地形数据
    /// 生成步骤：
    /// 1. 生成基础地形（高度、生物群系）
    /// 2. 填充地下的方块（石头、矿石）
    /// 3. 生成洞穴系统
    /// 4. 填充水体
    /// 5. 生成地表植被（树木）
    pub fn generate_chunk(&self, chunk_pos: ChunkPos) -> ChunkData {
        let mut chunk = ChunkData::new();
        let origin = chunk_pos.world_origin();
        let water_level = 30;

        // 遍历区块中的每一列
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let wx = origin.x + lx;
                let wz = origin.z + lz;
                let height = self.get_height(wx, wz);
                let biome = self.get_biome(wx, wz);

                // 从下到上填充每一层
                for y in 0..CHUNK_HEIGHT {
                    // 如果是洞穴位置，跳过（保持为空气）
                    if self.is_cave(wx, y, wz) && y < height {
                        continue;
                    }

                    let kind = if y > height && y <= water_level {
                        // 地表以上，水位以下 → 水体
                        if biome == Biome::Snowy && y == water_level {
                            VoxelKind::Ice // 寒冷生物群系的水面结冰
                        } else {
                            VoxelKind::Water
                        }
                    } else if y == height {
                        // 地表层 → 使用生物群系的表层方块
                        biome.surface_block()
                    } else if y > height - 4 && y < height {
                        // 次表层（地表下1-3层）→ 使用生物群系的次表层方块
                        biome.subsurface_block()
                    } else if y < height {
                        // 深层 → 石头或矿石
                        self.get_ore(wx, y, wz).unwrap_or(VoxelKind::Stone)
                    } else {
                        // 地表以上 → 空气
                        VoxelKind::Air
                    };

                    if kind != VoxelKind::Air {
                        chunk.set(lx, y, lz, kind);
                    }
                }

                // 在水位以上的陆地上生成树木
                if height > water_level && self.should_place_tree(wx, wz, biome) {
                    self.generate_tree(&mut chunk, lx, height + 1, lz, biome);
                }
            }
        }

        chunk
    }

    /// 在指定位置生成一棵树
    /// 根据生物群系类型生成不同的树木（橡树、白桦、云杉）
    fn generate_tree(&self, chunk: &mut ChunkData, x: i32, y: i32, z: i32, biome: Biome) {
        // 根据生物群系选择树木类型和高度
        let (log, leaves, trunk_h) = match biome {
            Biome::Forest | Biome::Plains => (VoxelKind::OakLog, VoxelKind::OakLeaves, 5),
            Biome::BirchForest => (VoxelKind::BirchLog, VoxelKind::BirchLeaves, 6),
            Biome::Taiga | Biome::Snowy => (VoxelKind::SpruceLog, VoxelKind::SpruceLeaves, 6),
            _ => return,
        };

        // 生成树干
        for dy in 0..trunk_h {
            chunk.set(x, y + dy, z, log);
        }

        // 生成树叶
        let leaf_start = trunk_h - 2;
        for dy in leaf_start..trunk_h + 2 {
            // 顶部树叶半径较小，底部树叶半径较大
            let radius: i32 = if dy >= trunk_h { 1 } else { 2 };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    // 树干位置不放置树叶
                    if dx == 0 && dz == 0 && dy < trunk_h {
                        continue;
                    }
                    // 使用曼哈顿距离创建菱形树冠
                    if dx.abs() + dz.abs() <= radius + 1 {
                        chunk.set(x + dx, y + dy, z + dz, leaves);
                    }
                }
            }
        }
    }
}
