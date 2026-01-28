//! 地形生成器

use noise::NoiseFn;

use crate::voxel::biome::Biome;
use crate::voxel::chunk::{ChunkData, ChunkPos};
use crate::voxel::constants::CHUNK_SIZE;
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
        ((base_height as f64 + height) as i32).max(1)
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
        if y > 60 || y < 5 {
            return false;
        }
        let scale = 0.08;
        let value = self
            .seed
            .cave_noise
            .get([x as f64 * scale, y as f64 * scale, z as f64 * scale]);
        value > 0.55
    }

    /// 判断指定位置是否属于浮空岛
    /// 使用3D噪声生成浮空岛地形
    /// 浮空岛在Y=65到Y=100的范围内生成
    pub fn is_floating_island(&self, x: i32, y: i32, z: i32) -> bool {
        // 浮空岛只在高空生成
        if y < 65 || y > 110 {
            return false;
        }

        // 使用3D噪声生成浮空岛
        let scale = 0.015; // 较大的尺度产生较大的浮空岛
        let noise_value = self.seed.terrain_noise.get([
            x as f64 * scale,
            y as f64 * scale * 0.5, // Y轴压缩，让岛更扁平
            z as f64 * scale,
        ]);

        // 添加细节噪声
        let detail_scale = 0.06;
        let detail = self.seed.detail_noise.get([
            x as f64 * detail_scale,
            y as f64 * detail_scale,
            z as f64 * detail_scale,
        ]);

        // 计算到岛中心的距离，让浮空岛有边界
        let island_center_scale = 0.005;
        let island_noise = self.seed.biome_temp_noise.get([
            x as f64 * island_center_scale,
            z as f64 * island_center_scale,
        ]);

        // 只在特定区域生成浮空岛（稀疏分布）
        if island_noise < 0.3 {
            return false;
        }

        // 高度衰减：越接近上下边界，越难生成
        let y_normalized = (y - 65) as f64 / 45.0; // 0.0 到 1.0
        let y_factor = if y_normalized < 0.5 {
            // 下半部分：从底部向上逐渐增强
            y_normalized * 2.0
        } else {
            // 上半部分：从顶部向下逐渐减弱
            (1.0 - y_normalized) * 2.0
        };

        // 综合判断
        let threshold = 0.4 - y_factor * 0.2;
        (noise_value + detail * 0.3) > threshold
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

    /// 生成指定区块的完整地形数据（3D分层版本）
    /// 生成步骤：
    /// 1. 计算chunk的世界Y范围
    /// 2. 遍历chunk内的每个体素
    /// 3. 根据世界坐标决定体素类型
    /// 4. 生成跨chunk结构（树木等）的部分
    pub fn generate_chunk(&self, chunk_pos: ChunkPos) -> ChunkData {
        let mut chunk = ChunkData::new();
        let origin = chunk_pos.world_origin();

        // 计算chunk的世界Y范围
        let chunk_y_min = origin.y;
        let chunk_y_max = origin.y + CHUNK_SIZE - 1;

        const WATER_LEVEL: i32 = 30;
        const BEDROCK_LAYER: i32 = 0;

        // 遍历chunk内的每个体素
        for ly in 0..CHUNK_SIZE {
            let world_y = chunk_y_min + ly;

            for lz in 0..CHUNK_SIZE {
                for lx in 0..CHUNK_SIZE {
                    let world_x = origin.x + lx;
                    let world_z = origin.z + lz;

                    // 获取该列的地形高度和生物群系
                    let height = self.get_height(world_x, world_z);
                    let biome = self.get_biome(world_x, world_z);

                    // 判断当前体素应该是什么类型
                    let kind = if world_y == BEDROCK_LAYER {
                        // Y=0层：基岩（不可破坏）
                        VoxelKind::Stone // 或者添加 VoxelKind::Bedrock
                    } else if world_y < height {
                        // 地表以下：根据深度生成不同材料
                        if self.is_cave(world_x, world_y, world_z) {
                            // 洞穴空间：空气
                            VoxelKind::Air
                        } else if world_y == height - 1 {
                            // 地表层
                            biome.surface_block()
                        } else if world_y > height - 5 {
                            // 次表层（地表下1-4层）
                            biome.subsurface_block()
                        } else {
                            // 深层：石头或矿石
                            self.get_ore(world_x, world_y, world_z)
                                .unwrap_or(VoxelKind::Stone)
                        }
                    } else if world_y >= height && world_y <= WATER_LEVEL {
                        // 地表到水位之间：水体
                        if biome == Biome::Snowy && world_y == WATER_LEVEL {
                            VoxelKind::Ice
                        } else {
                            VoxelKind::Water
                        }
                    } else if world_y > WATER_LEVEL && self.is_floating_island(world_x, world_y, world_z) {
                        // 高空：浮空岛
                        // 使用简单的高度判断来决定岛的材质
                        // 计算浮空岛局部的顶部和底部
                        let above = self.is_floating_island(world_x, world_y + 1, world_z);
                        let below = self.is_floating_island(world_x, world_y - 1, world_z);

                        if !above {
                            // 顶层：草地
                            VoxelKind::Grass
                        } else if !below || world_y < 68 {
                            // 底层或接近底部：石头
                            VoxelKind::Stone
                        } else {
                            // 中间层：泥土
                            VoxelKind::Dirt
                        }
                    } else {
                        // 地表以上：空气（或树木，后续处理）
                        VoxelKind::Air
                    };

                    if kind != VoxelKind::Air {
                        chunk.set(lx, ly, lz, kind);
                    }
                }
            }
        }

        // 生成树木（在地表和浮空岛上生成）
        // 地表树木
        if chunk_y_min <= WATER_LEVEL + 10 && chunk_y_max >= WATER_LEVEL {
            for lz in 0..CHUNK_SIZE {
                for lx in 0..CHUNK_SIZE {
                    let world_x = origin.x + lx;
                    let world_z = origin.z + lz;
                    let height = self.get_height(world_x, world_z);
                    let biome = self.get_biome(world_x, world_z);

                    if height > WATER_LEVEL && self.should_place_tree(world_x, world_z, biome) {
                        let tree_base_y = height + 1;
                        // 只生成位于当前chunk Y范围内的树木部分
                        self.generate_tree_partial(
                            &mut chunk,
                            lx,
                            tree_base_y,
                            lz,
                            biome,
                            chunk_y_min,
                            chunk_y_max,
                        );
                    }
                }
            }
        }

        // 浮空岛树木
        if chunk_y_min <= 100 && chunk_y_max >= 65 {
            for lz in 0..CHUNK_SIZE {
                for lx in 0..CHUNK_SIZE {
                    let world_x = origin.x + lx;
                    let world_z = origin.z + lz;

                    // 在浮空岛上寻找合适的位置生成树木
                    for ly in 0..CHUNK_SIZE {
                        let world_y = chunk_y_min + ly;
                        if world_y < 65 || world_y > 100 {
                            continue;
                        }

                        // 检查当前位置是草地且上方是空气（浮空岛表面）
                        if chunk.get(lx, ly, lz) == VoxelKind::Grass
                            && chunk.get(lx, ly + 1, lz) == VoxelKind::Air
                        {
                            // 使用噪声决定是否生成树木（稀疏分布）
                            let scale = 0.3;
                            let noise = self.seed.detail_noise.get([
                                world_x as f64 * scale,
                                world_z as f64 * scale,
                            ]);
                            if noise > 0.88 {
                                // 浮空岛上生成小树
                                self.generate_tree_partial(
                                    &mut chunk,
                                    lx,
                                    world_y + 1,
                                    lz,
                                    Biome::FloatingIslands,
                                    chunk_y_min,
                                    chunk_y_max,
                                );
                            }
                            break; // 每列只检查一次
                        }
                    }
                }
            }
        }

        chunk
    }

    /// 生成树木的部分（仅生成在当前chunk范围内的部分）
    /// 支持跨chunk的树木生成
    fn generate_tree_partial(
        &self,
        chunk: &mut ChunkData,
        local_x: i32,
        tree_base_world_y: i32,
        local_z: i32,
        biome: Biome,
        chunk_y_min: i32,
        chunk_y_max: i32,
    ) {
        // 根据生物群系选择树木类型和高度
        let (log, leaves, trunk_h) = match biome {
            Biome::Forest | Biome::Plains => (VoxelKind::OakLog, VoxelKind::OakLeaves, 5),
            Biome::BirchForest => (VoxelKind::BirchLog, VoxelKind::BirchLeaves, 6),
            Biome::Taiga | Biome::Snowy => (VoxelKind::SpruceLog, VoxelKind::SpruceLeaves, 6),
            Biome::FloatingIslands => (VoxelKind::OakLog, VoxelKind::OakLeaves, 4), // 浮空岛上的小树
            _ => return,
        };

        // 生成树干
        for dy in 0..trunk_h {
            let world_y = tree_base_world_y + dy;
            if world_y >= chunk_y_min && world_y <= chunk_y_max {
                let local_y = world_y - chunk_y_min;
                chunk.set(local_x, local_y, local_z, log);
            }
        }

        // 生成树叶
        let leaf_start = trunk_h - 2;
        for dy in leaf_start..trunk_h + 2 {
            let radius: i32 = if dy >= trunk_h { 1 } else { 2 };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    // 树干位置不放置树叶
                    if dx == 0 && dz == 0 && dy < trunk_h {
                        continue;
                    }
                    // 使用曼哈顿距离创建菱形树冠
                    if dx.abs() + dz.abs() <= radius + 1 {
                        let world_y = tree_base_world_y + dy;
                        if world_y >= chunk_y_min && world_y <= chunk_y_max {
                            let local_y = world_y - chunk_y_min;
                            let leaf_x = local_x + dx;
                            let leaf_z = local_z + dz;
                            // 边界检查
                            if leaf_x >= 0
                                && leaf_x < CHUNK_SIZE
                                && leaf_z >= 0
                                && leaf_z < CHUNK_SIZE
                            {
                                chunk.set(leaf_x, local_y, leaf_z, leaves);
                            }
                        }
                    }
                }
            }
        }
    }
}
