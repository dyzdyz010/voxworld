use bevy::prelude::*;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use noise::{NoiseFn, Perlin};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

// ============================================================================
// 常量定义
// ============================================================================

/// 区块在X和Z方向上的大小（单位：体素）
pub const CHUNK_SIZE: i32 = 16;
/// 区块在Y方向上的高度（单位：体素）
pub const CHUNK_HEIGHT: i32 = 64;
/// 渲染距离（单位：区块数）- 控制玩家周围加载多少区块
pub const RENDER_DISTANCE: i32 = 16; // chunks

// ============================================================================
// 体素（方块）类型
// ============================================================================

/// 体素种类枚举 - 定义游戏中所有可用的方块类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VoxelKind {
    #[default]
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Gravel,
    Clay,
    Snow,
    Ice,
    Water,
    OakLog,
    OakLeaves,
    BirchLog,
    BirchLeaves,
    SpruceLog,
    SpruceLeaves,
    Cactus,
    CoalOre,
    IronOre,
    GoldOre,
    DiamondOre,
    Flower,
    TallGrass,
    DeadBush,
}

/// 体素的物理属性
#[derive(Debug, Clone, Copy)]
pub struct VoxelProperties {
    /// 温度（摄氏度）
    pub temperature: f32,
    /// 湿度（0.0-1.0）
    pub humidity: f32,
    /// 硬度（0.0-1.0，值越大越难破坏）
    pub hardness: f32,
    /// 延展性（0.0-1.0，值越大越容易变形）
    pub ductility: f32,
}

/// 体素定义 - 包含体素的所有基础信息
#[derive(Debug, Clone, Copy)]
pub struct VoxelDef {
    /// 方块名称
    pub name: &'static str,
    /// 方块颜色
    pub color: Color,
    /// 方块物理属性
    pub props: VoxelProperties,
}

impl VoxelKind {
    /// 获取当前体素种类的完整定义信息
    pub fn def(self) -> VoxelDef {
        match self {
            VoxelKind::Air => VoxelDef {
                name: "空气",
                color: Color::NONE,
                props: VoxelProperties {
                    temperature: 20.0,
                    humidity: 0.5,
                    hardness: 0.0,
                    ductility: 0.0,
                },
            },
            VoxelKind::Grass => VoxelDef {
                name: "草方块",
                color: Color::srgb(0.28, 0.62, 0.25),
                props: VoxelProperties {
                    temperature: 18.0,
                    humidity: 0.6,
                    hardness: 0.2,
                    ductility: 0.35,
                },
            },
            VoxelKind::Dirt => VoxelDef {
                name: "泥土",
                color: Color::srgb(0.42, 0.30, 0.18),
                props: VoxelProperties {
                    temperature: 16.0,
                    humidity: 0.4,
                    hardness: 0.35,
                    ductility: 0.2,
                },
            },
            VoxelKind::Stone => VoxelDef {
                name: "石头",
                color: Color::srgb(0.55, 0.55, 0.58),
                props: VoxelProperties {
                    temperature: 12.0,
                    humidity: 0.1,
                    hardness: 0.9,
                    ductility: 0.05,
                },
            },
            VoxelKind::Sand => VoxelDef {
                name: "沙子",
                color: Color::srgb(0.86, 0.82, 0.58),
                props: VoxelProperties {
                    temperature: 28.0,
                    humidity: 0.05,
                    hardness: 0.25,
                    ductility: 0.45,
                },
            },
            VoxelKind::Gravel => VoxelDef {
                name: "砂砾",
                color: Color::srgb(0.52, 0.50, 0.48),
                props: VoxelProperties {
                    temperature: 14.0,
                    humidity: 0.15,
                    hardness: 0.4,
                    ductility: 0.3,
                },
            },
            VoxelKind::Clay => VoxelDef {
                name: "黏土",
                color: Color::srgb(0.62, 0.64, 0.68),
                props: VoxelProperties {
                    temperature: 15.0,
                    humidity: 0.7,
                    hardness: 0.3,
                    ductility: 0.5,
                },
            },
            VoxelKind::Snow => VoxelDef {
                name: "雪块",
                color: Color::srgb(0.95, 0.97, 1.0),
                props: VoxelProperties {
                    temperature: -5.0,
                    humidity: 0.8,
                    hardness: 0.1,
                    ductility: 0.2,
                },
            },
            VoxelKind::Ice => VoxelDef {
                name: "冰块",
                color: Color::srgba(0.68, 0.85, 0.95, 0.85),
                props: VoxelProperties {
                    temperature: -10.0,
                    humidity: 0.9,
                    hardness: 0.3,
                    ductility: 0.1,
                },
            },
            VoxelKind::Water => VoxelDef {
                name: "水",
                color: Color::srgba(0.20, 0.45, 0.78, 0.7),
                props: VoxelProperties {
                    temperature: 14.0,
                    humidity: 1.0,
                    hardness: 0.0,
                    ductility: 1.0,
                },
            },
            VoxelKind::OakLog => VoxelDef {
                name: "橡木原木",
                color: Color::srgb(0.40, 0.30, 0.18),
                props: VoxelProperties {
                    temperature: 20.0,
                    humidity: 0.3,
                    hardness: 0.5,
                    ductility: 0.6,
                },
            },
            VoxelKind::OakLeaves => VoxelDef {
                name: "橡树树叶",
                color: Color::srgba(0.22, 0.52, 0.20, 0.9),
                props: VoxelProperties {
                    temperature: 22.0,
                    humidity: 0.5,
                    hardness: 0.05,
                    ductility: 0.1,
                },
            },
            VoxelKind::BirchLog => VoxelDef {
                name: "白桦原木",
                color: Color::srgb(0.85, 0.82, 0.75),
                props: VoxelProperties {
                    temperature: 18.0,
                    humidity: 0.35,
                    hardness: 0.45,
                    ductility: 0.55,
                },
            },
            VoxelKind::BirchLeaves => VoxelDef {
                name: "白桦树叶",
                color: Color::srgba(0.45, 0.62, 0.35, 0.9),
                props: VoxelProperties {
                    temperature: 20.0,
                    humidity: 0.45,
                    hardness: 0.05,
                    ductility: 0.1,
                },
            },
            VoxelKind::SpruceLog => VoxelDef {
                name: "云杉原木",
                color: Color::srgb(0.30, 0.22, 0.12),
                props: VoxelProperties {
                    temperature: 8.0,
                    humidity: 0.4,
                    hardness: 0.55,
                    ductility: 0.5,
                },
            },
            VoxelKind::SpruceLeaves => VoxelDef {
                name: "云杉树叶",
                color: Color::srgba(0.15, 0.35, 0.22, 0.9),
                props: VoxelProperties {
                    temperature: 6.0,
                    humidity: 0.5,
                    hardness: 0.05,
                    ductility: 0.1,
                },
            },
            VoxelKind::Cactus => VoxelDef {
                name: "仙人掌",
                color: Color::srgb(0.25, 0.55, 0.20),
                props: VoxelProperties {
                    temperature: 35.0,
                    humidity: 0.1,
                    hardness: 0.2,
                    ductility: 0.3,
                },
            },
            VoxelKind::CoalOre => VoxelDef {
                name: "煤矿石",
                color: Color::srgb(0.25, 0.25, 0.28),
                props: VoxelProperties {
                    temperature: 12.0,
                    humidity: 0.1,
                    hardness: 0.85,
                    ductility: 0.05,
                },
            },
            VoxelKind::IronOre => VoxelDef {
                name: "铁矿石",
                color: Color::srgb(0.58, 0.52, 0.48),
                props: VoxelProperties {
                    temperature: 12.0,
                    humidity: 0.1,
                    hardness: 0.9,
                    ductility: 0.05,
                },
            },
            VoxelKind::GoldOre => VoxelDef {
                name: "金矿石",
                color: Color::srgb(0.72, 0.65, 0.35),
                props: VoxelProperties {
                    temperature: 12.0,
                    humidity: 0.1,
                    hardness: 0.85,
                    ductility: 0.15,
                },
            },
            VoxelKind::DiamondOre => VoxelDef {
                name: "钻石矿石",
                color: Color::srgb(0.45, 0.72, 0.78),
                props: VoxelProperties {
                    temperature: 12.0,
                    humidity: 0.1,
                    hardness: 0.98,
                    ductility: 0.02,
                },
            },
            VoxelKind::Flower => VoxelDef {
                name: "花",
                color: Color::srgb(0.85, 0.35, 0.40),
                props: VoxelProperties {
                    temperature: 22.0,
                    humidity: 0.6,
                    hardness: 0.01,
                    ductility: 0.05,
                },
            },
            VoxelKind::TallGrass => VoxelDef {
                name: "高草丛",
                color: Color::srgb(0.35, 0.58, 0.28),
                props: VoxelProperties {
                    temperature: 20.0,
                    humidity: 0.5,
                    hardness: 0.01,
                    ductility: 0.05,
                },
            },
            VoxelKind::DeadBush => VoxelDef {
                name: "枯死的灌木",
                color: Color::srgb(0.55, 0.45, 0.28),
                props: VoxelProperties {
                    temperature: 32.0,
                    humidity: 0.05,
                    hardness: 0.01,
                    ductility: 0.02,
                },
            },
        }
    }

    /// 判断体素是否透明（用于渲染优化，透明方块需要渲染相邻面）
    pub fn is_transparent(self) -> bool {
        matches!(
            self,
            VoxelKind::Air
                | VoxelKind::Water
                | VoxelKind::Ice
                | VoxelKind::OakLeaves
                | VoxelKind::BirchLeaves
                | VoxelKind::SpruceLeaves
                | VoxelKind::Flower
                | VoxelKind::TallGrass
                | VoxelKind::DeadBush
        )
    }

    /// 判断体素是否为固体（用于碰撞检测）
    pub fn is_solid(self) -> bool {
        !matches!(self, VoxelKind::Air | VoxelKind::Flower | VoxelKind::TallGrass | VoxelKind::DeadBush)
    }
}

// ============================================================================
// 生物群系
// ============================================================================

/// 生物群系类型 - 决定地形的表面方块、植被和环境特征
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Forest,
    BirchForest,
    Desert,
    Snowy,
    Taiga,
    Ocean,
    Beach,
}

impl Biome {
    /// 获取该生物群系的表面方块类型（地表最顶层的方块）
    pub fn surface_block(self) -> VoxelKind {
        match self {
            Biome::Plains | Biome::Forest | Biome::BirchForest => VoxelKind::Grass,
            Biome::Desert => VoxelKind::Sand,
            Biome::Snowy => VoxelKind::Snow,
            Biome::Taiga => VoxelKind::Grass,
            Biome::Ocean => VoxelKind::Gravel,
            Biome::Beach => VoxelKind::Sand,
        }
    }

    /// 获取该生物群系的次表层方块类型（表层下方的方块）
    pub fn subsurface_block(self) -> VoxelKind {
        match self {
            Biome::Plains | Biome::Forest | Biome::BirchForest | Biome::Taiga => VoxelKind::Dirt,
            Biome::Desert | Biome::Beach => VoxelKind::Sand,
            Biome::Snowy => VoxelKind::Dirt,
            Biome::Ocean => VoxelKind::Clay,
        }
    }
}

// ============================================================================
// 世界种子与生成
// ============================================================================

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

// ============================================================================
// 区块系统
// ============================================================================

/// 区块坐标 - 用于标识世界中区块的位置
/// 注意：这是区块坐标，不是体素（方块）坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    /// 创建新的区块坐标
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// 从世界坐标（体素坐标）转换为区块坐标
    /// 使用欧几里德除法确保负坐标也能正确转换
    pub fn from_world_pos(world_x: i32, world_z: i32) -> Self {
        Self {
            x: world_x.div_euclid(CHUNK_SIZE),
            z: world_z.div_euclid(CHUNK_SIZE),
        }
    }

    /// 获取区块在世界坐标系中的起始位置（左下角）
    pub fn world_origin(&self) -> IVec3 {
        IVec3::new(self.x * CHUNK_SIZE, 0, self.z * CHUNK_SIZE)
    }
}

/// 区块标记组件 - 用于标识游戏实体对应的区块位置
#[derive(Component)]
pub struct ChunkMarker {
    pub pos: ChunkPos,
}

/// 区块数据 - 存储区块内所有体素的类型数据
pub struct ChunkData {
    /// 体素数组，大小为 CHUNK_SIZE × CHUNK_HEIGHT × CHUNK_SIZE
    /// 使用一维数组存储三维数据，通过 index() 函数计算索引
    pub voxels: Vec<VoxelKind>,
    /// 脏标记 - 标识区块是否被修改，需要重新生成网格
    pub is_dirty: bool,
}

impl ChunkData {
    /// 创建一个空的区块数据，所有体素初始化为空气
    pub fn new() -> Self {
        Self {
            voxels: vec![VoxelKind::Air; (CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE) as usize],
            is_dirty: true,
        }
    }

    /// 将三维坐标转换为一维数组索引
    /// 使用Y-Z-X顺序进行线性化，便于按层遍历
    #[inline]
    fn index(x: i32, y: i32, z: i32) -> usize {
        ((y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x) as usize
    }

    /// 获取指定位置的体素类型
    /// 如果坐标超出边界，返回空气
    pub fn get(&self, x: i32, y: i32, z: i32) -> VoxelKind {
        if x < 0 || x >= CHUNK_SIZE || y < 0 || y >= CHUNK_HEIGHT || z < 0 || z >= CHUNK_SIZE {
            return VoxelKind::Air;
        }
        self.voxels[Self::index(x, y, z)]
    }

    /// 设置指定位置的体素类型
    /// 如果坐标超出边界，不执行操作
    /// 设置后会标记区块为脏，需要重新生成网格
    pub fn set(&mut self, x: i32, y: i32, z: i32, kind: VoxelKind) {
        if x < 0 || x >= CHUNK_SIZE || y < 0 || y >= CHUNK_HEIGHT || z < 0 || z >= CHUNK_SIZE {
            return;
        }
        self.voxels[Self::index(x, y, z)] = kind;
        self.is_dirty = true;
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
        let chunk_pos = ChunkPos::from_world_pos(world_pos.x, world_pos.z);
        let local_x = world_pos.x.rem_euclid(CHUNK_SIZE);
        let local_z = world_pos.z.rem_euclid(CHUNK_SIZE);

        self.chunks
            .get(&chunk_pos)
            .map(|chunk| chunk.get(local_x, world_pos.y, local_z))
            .unwrap_or(VoxelKind::Air)
    }

    /// 设置世界中指定位置的体素类型
    /// 如果对应的区块不存在，不执行操作
    pub fn set_voxel(&mut self, world_pos: IVec3, kind: VoxelKind) {
        let chunk_pos = ChunkPos::from_world_pos(world_pos.x, world_pos.z);
        let local_x = world_pos.x.rem_euclid(CHUNK_SIZE);
        let local_z = world_pos.z.rem_euclid(CHUNK_SIZE);

        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set(local_x, world_pos.y, local_z, kind);
        }
    }
}

// ============================================================================
// 地形生成器
// ============================================================================

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
                    Biome::Taiga  // 湿润的寒带 → 针叶林
                } else {
                    Biome::Snowy  // 干燥的寒带 → 雪地
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
            Biome::Forest | Biome::Taiga => 0.06,  // 森林和针叶林：6%
            Biome::BirchForest => 0.04,            // 白桦林：4%
            Biome::Plains => 0.003,                // 平原：0.3%
            _ => 0.0,                              // 其他生物群系不生成树木
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
                            VoxelKind::Ice  // 寒冷生物群系的水面结冰
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

// ============================================================================
// 顶点去重
// ============================================================================

/// 顶点唯一标识键 - 用于HashMap去重
/// 考虑位置、法线和颜色
#[derive(Clone, Copy)]
struct VertexKey {
    /// 位置 - 使用定点数避免浮点精度问题（乘以1000转整数）
    pos: [i32; 3],
    /// 法线方向索引 - 6个方向：0=+X, 1=-X, 2=+Y, 3=-Y, 4=+Z, 5=-Z
    normal_index: u8,
    /// 颜色 - 压缩为32位RGBA
    color_packed: u32,
}

impl VertexKey {
    /// 从浮点数据创建顶点键
    fn new(pos: [f32; 3], normal: [f32; 3], color: [f32; 4]) -> Self {
        // 位置转定点数
        let pos_fixed = [
            (pos[0] * 1000.0) as i32,
            (pos[1] * 1000.0) as i32,
            (pos[2] * 1000.0) as i32,
        ];

        // 法线编码为索引
        let normal_index = match (normal[0] as i32, normal[1] as i32, normal[2] as i32) {
            (1, 0, 0) => 0,
            (-1, 0, 0) => 1,
            (0, 1, 0) => 2,
            (0, -1, 0) => 3,
            (0, 0, 1) => 4,
            (0, 0, -1) => 5,
            _ => 0,
        };

        // 颜色压缩为RGBA8888
        let color_packed = ((color[0] * 255.0) as u32) << 24
            | ((color[1] * 255.0) as u32) << 16
            | ((color[2] * 255.0) as u32) << 8
            | ((color[3] * 255.0) as u32);

        Self {
            pos: pos_fixed,
            normal_index,
            color_packed,
        }
    }
}

impl PartialEq for VertexKey {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
            && self.normal_index == other.normal_index
            && self.color_packed == other.color_packed
    }
}

impl Eq for VertexKey {}

impl Hash for VertexKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].hash(state);
        self.pos[1].hash(state);
        self.pos[2].hash(state);
        self.normal_index.hash(state);
        self.color_packed.hash(state);
    }
}

// ============================================================================
// 线程本地缓冲区
// ============================================================================

/// 网格构建缓冲区 - 避免每次分配新Vec
struct MeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
    /// 顶点去重HashMap
    vertex_map: HashMap<VertexKey, u32>,
}

impl MeshBuffers {
    fn new() -> Self {
        // 预分配合理的初始容量
        Self {
            positions: Vec::with_capacity(20000),
            normals: Vec::with_capacity(20000),
            colors: Vec::with_capacity(20000),
            indices: Vec::with_capacity(30000),
            vertex_map: HashMap::with_capacity(20000),
        }
    }

    /// 清空缓冲区但保留容量
    fn clear(&mut self) {
        self.positions.clear();
        self.normals.clear();
        self.colors.clear();
        self.indices.clear();
        self.vertex_map.clear();
    }
}

thread_local! {
    /// 每个线程独立的网格构建缓冲区
    static MESH_BUFFERS: RefCell<MeshBuffers> = RefCell::new(MeshBuffers::new());
}

// ============================================================================
// 贪婪网格化（Greedy Meshing）
// ============================================================================

/// 面片数据 - 用于网格优化算法
#[derive(Clone, Copy, PartialEq, Eq)]
struct FaceData {
    kind: VoxelKind,
    ao: u8, // 环境光遮蔽（Ambient Occlusion）
}

/// 优化后的区块网格构建器 - 使用线程本地缓冲区和顶点去重
struct ChunkMeshBuilder<'a> {
    buffers: &'a mut MeshBuffers,
}

impl<'a> ChunkMeshBuilder<'a> {
    /// 使用线程本地缓冲区创建构建器
    fn with_buffers(buffers: &'a mut MeshBuffers) -> Self {
        buffers.clear();
        Self { buffers }
    }

    /// 添加面片并进行顶点去重
    fn add_face_deduplicated(
        &mut self,
        vertices: [[f32; 3]; 4],
        normal: [f32; 3],
        color: [f32; 4],
    ) {
        let mut face_indices = [0u32; 4];

        for (i, &pos) in vertices.iter().enumerate() {
            let key = VertexKey::new(pos, normal, color);

            // 查找或插入顶点
            let index = match self.buffers.vertex_map.get(&key) {
                Some(&existing_index) => existing_index,
                None => {
                    let new_index = self.buffers.positions.len() as u32;
                    self.buffers.positions.push(pos);
                    self.buffers.normals.push(normal);
                    self.buffers.colors.push(color);
                    self.buffers.vertex_map.insert(key, new_index);
                    new_index
                }
            };

            face_indices[i] = index;
        }

        // 添加两个三角形的索引
        self.buffers.indices.extend_from_slice(&[
            face_indices[0],
            face_indices[2],
            face_indices[1],
            face_indices[0],
            face_indices[3],
            face_indices[2],
        ]);
    }

    /// 构建最终网格（从缓冲区克隆数据）
    fn build(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.buffers.positions.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.buffers.normals.clone());
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::Float32x4(self.buffers.colors.clone()),
        );
        mesh.insert_indices(Indices::U32(self.buffers.indices.clone()));
        mesh
    }
}

/// 创建蓝色线框占位符网格
/// 只绘制区块的立方体边框（12条边）
fn create_placeholder_mesh() -> Mesh {
    let size = CHUNK_SIZE as f32;
    let height = CHUNK_HEIGHT as f32;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    // 半透明蓝色
    let blue = [0.3, 0.6, 1.0, 0.3];
    let normal = [0.0, 1.0, 0.0]; // 线框法线随意

    // 8个角的顶点
    let corners = [
        [0.0, 0.0, 0.0],       // 0: 左下前
        [size, 0.0, 0.0],      // 1: 右下前
        [size, 0.0, size],     // 2: 右下后
        [0.0, 0.0, size],      // 3: 左下后
        [0.0, height, 0.0],    // 4: 左上前
        [size, height, 0.0],   // 5: 右上前
        [size, height, size],  // 6: 右上后
        [0.0, height, size],   // 7: 左上后
    ];

    // 12条边（每条边连接两个顶点）
    let edges = [
        // 底部4条边
        (0, 1), (1, 2), (2, 3), (3, 0),
        // 顶部4条边
        (4, 5), (5, 6), (6, 7), (7, 4),
        // 垂直4条边
        (0, 4), (1, 5), (2, 6), (3, 7),
    ];

    // 为每条边添加两个顶点
    for (start, end) in edges {
        let idx = positions.len() as u32;
        positions.push(corners[start]);
        positions.push(corners[end]);
        normals.push(normal);
        normals.push(normal);
        colors.push(blue);
        colors.push(blue);
        indices.extend_from_slice(&[idx, idx + 1]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// 在工作线程中生成区块数据并构建网格
/// 包含地形生成和网格构建两个阶段
fn generate_chunk_and_mesh_async(chunk_pos: ChunkPos, seed: u32) -> (Vec<VoxelKind>, Mesh) {
    // 阶段1：生成区块地形数据
    let world_seed = WorldSeed::new(seed);
    let generator = TerrainGenerator::new(&world_seed);
    let chunk_data = generator.generate_chunk(chunk_pos);
    let voxels = chunk_data.voxels.clone();

    // 阶段2：构建网格（需要相邻区块数据，但首次生成时使用空边界）
    let input = MeshBuildInput {
        chunk_pos,
        voxels: Arc::new(voxels.clone()),
        neighbor_edges: NeighborEdges::default(),
    };

    let mesh = build_chunk_mesh_async(input);

    (voxels, mesh)
}

/// 在工作线程中构建区块网格
/// 使用线程本地缓冲区和顶点去重优化
fn build_chunk_mesh_async(input: MeshBuildInput) -> Mesh {
    MESH_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let mut builder = ChunkMeshBuilder::with_buffers(&mut buffers);

        // 6个面的方向和法线
        let directions: [(IVec3, [f32; 3]); 6] = [
            (IVec3::X, [1.0, 0.0, 0.0]),
            (IVec3::NEG_X, [-1.0, 0.0, 0.0]),
            (IVec3::Y, [0.0, 1.0, 0.0]),
            (IVec3::NEG_Y, [0.0, -1.0, 0.0]),
            (IVec3::Z, [0.0, 0.0, 1.0]),
            (IVec3::NEG_Z, [0.0, 0.0, -1.0]),
        ];

        // 遍历区块中的所有体素
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let index = ((y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x) as usize;
                    let kind = input.voxels[index];

                    if kind == VoxelKind::Air {
                        continue;
                    }

                    let def = kind.def();
                    let color = def.color.to_srgba();
                    let base_color = [color.red, color.green, color.blue, color.alpha];
                    let local_pos = IVec3::new(x, y, z);

                    // 检查每个面
                    for (dir, normal) in &directions {
                        let neighbor_local = local_pos + *dir;

                        // 判断相邻位置是否在区块内
                        let neighbor = if neighbor_local.x >= 0
                            && neighbor_local.x < CHUNK_SIZE
                            && neighbor_local.y >= 0
                            && neighbor_local.y < CHUNK_HEIGHT
                            && neighbor_local.z >= 0
                            && neighbor_local.z < CHUNK_SIZE
                        {
                            // 区块内部查询
                            let ni = ((neighbor_local.y * CHUNK_SIZE * CHUNK_SIZE)
                                + (neighbor_local.z * CHUNK_SIZE)
                                + neighbor_local.x) as usize;
                            input.voxels[ni]
                        } else if neighbor_local.y < 0 || neighbor_local.y >= CHUNK_HEIGHT {
                            // Y方向超出世界边界
                            VoxelKind::Air
                        } else {
                            // 查询相邻区块边界
                            input
                                .neighbor_edges
                                .get_neighbor(local_pos, *dir)
                                .unwrap_or(VoxelKind::Air)
                        };

                        // 只渲染暴露的面
                        if !neighbor.is_transparent() {
                            continue;
                        }

                        // 水面之间不渲染
                        if kind == VoxelKind::Water && neighbor == VoxelKind::Water {
                            continue;
                        }

                        let vertices = get_face_vertices(x as f32, y as f32, z as f32, *dir);
                        builder.add_face_deduplicated(vertices, *normal, base_color);
                    }
                }
            }
        }

        builder.build()
    })
}

/// 根据方向获取面片的4个顶点坐标
/// 顶点顺序确保逆时针环绕（用于正确的面剔除）
fn get_face_vertices(x: f32, y: f32, z: f32, dir: IVec3) -> [[f32; 3]; 4] {
    match (dir.x, dir.y, dir.z) {
        // 右面 (+X)
        (1, 0, 0) => [
            [x + 1.0, y, z],
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z],
        ],
        // 左面 (-X)
        (-1, 0, 0) => [
            [x, y, z + 1.0],
            [x, y, z],
            [x, y + 1.0, z],
            [x, y + 1.0, z + 1.0],
        ],
        // 上面 (+Y)
        (0, 1, 0) => [
            [x, y + 1.0, z],
            [x + 1.0, y + 1.0, z],
            [x + 1.0, y + 1.0, z + 1.0],
            [x, y + 1.0, z + 1.0],
        ],
        // 下面 (-Y)
        (0, -1, 0) => [
            [x, y, z + 1.0],
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y, z],
            [x, y, z],
        ],
        // 前面 (+Z)
        (0, 0, 1) => [
            [x + 1.0, y, z + 1.0],
            [x, y, z + 1.0],
            [x, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
        ],
        // 后面 (-Z)
        (0, 0, -1) => [
            [x, y, z],
            [x + 1.0, y, z],
            [x + 1.0, y + 1.0, z],
            [x, y + 1.0, z],
        ],
        _ => [[0.0; 3]; 4],
    }
}

// ============================================================================
// 射线检测组件
// ============================================================================

/// 体素组件 - 用于射线检测和交互
#[derive(Component, Debug, Clone, Copy)]
pub struct Voxel {
    /// 体素类型
    pub kind: VoxelKind,
    /// 体素的世界坐标
    pub pos: IVec3,
}

// ============================================================================
// 插件与系统
// ============================================================================

/// 区块材质资源 - 存储不透明和透明材质的句柄
#[derive(Resource)]
pub struct ChunkMaterials {
    /// 不透明材质（用于大多数方块）
    pub opaque: Handle<StandardMaterial>,
    /// 透明材质（用于水、冰、树叶等）
    pub transparent: Handle<StandardMaterial>,
}

// ============================================================================
// 异步网格生成
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
        let mut face = Vec::with_capacity((CHUNK_HEIGHT * CHUNK_SIZE) as usize);
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                face.push(chunk.get(x, y, z));
            }
        }
        face
    }

    fn extract_z_face(chunk: &ChunkData, z: i32) -> Vec<VoxelKind> {
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

impl Default for ChunkLoadQueue {
    fn default() -> Self {
        Self {
            to_load: Vec::new(),
            to_unload: Vec::new(),
            active_tasks: 0,
            max_concurrent_tasks: 64,
            pending_placeholders: Vec::new(),
        }
    }
}

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
                )
                    .chain(),
            );
    }
}

/// 初始化材质系统
/// 创建不透明和透明两种材质
fn setup_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    // 不透明材质：高粗糙度，适合大多数方块
    let opaque = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        ..default()
    });

    // 透明材质：低粗糙度，支持透明混合
    let transparent = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.3,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.insert_resource(ChunkMaterials { opaque, transparent });
}

/// 更新区块加载系统
/// 根据摄像机位置决定哪些区块需要加载或卸载
/// 按距离排序：距离近的优先加载
fn update_chunk_loading(
    camera_query: Query<&Transform, With<Camera3d>>,
    world: Res<VoxelWorld>,
    mut queue: ResMut<ChunkLoadQueue>,
    pending_query: Query<&ComputeMeshTask>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_pos = camera_transform.translation;
    let center_chunk = ChunkPos::from_world_pos(camera_pos.x as i32, camera_pos.z as i32);

    // 收集正在处理中的区块
    let pending_chunks: Vec<ChunkPos> = pending_query.iter().map(|t| t.chunk_pos).collect();

    // 收集需要加载的区块
    let mut chunks_to_add = Vec::new();
    for dx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let chunk_pos = ChunkPos::new(center_chunk.x + dx, center_chunk.z + dz);
            if !world.loaded_chunks.contains_key(&chunk_pos)
                && !world.chunks.contains_key(&chunk_pos)
                && !queue.to_load.contains(&chunk_pos)
                && !pending_chunks.contains(&chunk_pos)
            {
                chunks_to_add.push(chunk_pos);
            }
        }
    }

    // 按距离排序（距离近的优先）
    chunks_to_add.sort_by(|a, b| {
        let dist_a = (a.x - center_chunk.x).pow(2) + (a.z - center_chunk.z).pow(2);
        let dist_b = (b.x - center_chunk.x).pow(2) + (b.z - center_chunk.z).pow(2);
        dist_a.cmp(&dist_b)
    });

    // 如果有新区块加入，将它们添加到待批量创建占位符列表
    if !chunks_to_add.is_empty() {
        queue.pending_placeholders.extend(chunks_to_add.iter().copied());
    }

    queue.to_load.extend(chunks_to_add);

    // 重新排序整个队列（玩家移动后需要重新排序）
    queue.to_load.sort_by(|a, b| {
        let dist_a = (a.x - center_chunk.x).pow(2) + (a.z - center_chunk.z).pow(2);
        let dist_b = (b.x - center_chunk.x).pow(2) + (b.z - center_chunk.z).pow(2);
        dist_a.cmp(&dist_b)
    });

    // 查找需要卸载的区块（超出渲染距离+1）
    for &chunk_pos in world.loaded_chunks.keys() {
        let dx = (chunk_pos.x - center_chunk.x).abs();
        let dz = (chunk_pos.z - center_chunk.z).abs();
        if dx > RENDER_DISTANCE + 1 || dz > RENDER_DISTANCE + 1 {
            if !queue.to_unload.contains(&chunk_pos) {
                queue.to_unload.push(chunk_pos);
            }
        }
    }
}

/// 批量创建占位符实体（一次性显示整个加载范围的蓝色网格线）
fn spawn_batch_placeholders(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<ChunkMaterials>,
    mut queue: ResMut<ChunkLoadQueue>,
    mut placeholders: ResMut<PlaceholderEntities>,
) {
    if queue.pending_placeholders.is_empty() {
        return;
    }

    // 批量创建所有待创建的占位符
    let chunks_to_create: Vec<_> = queue.pending_placeholders.drain(..).collect();

    // 创建共享的占位符网格（所有区块使用同一个网格）
    let placeholder_mesh = create_placeholder_mesh();
    let placeholder_handle = meshes.add(placeholder_mesh);

    for chunk_pos in chunks_to_create {
        let origin = chunk_pos.world_origin();

        let placeholder_entity = commands
            .spawn((
                Mesh3d(placeholder_handle.clone()),
                MeshMaterial3d(materials.transparent.clone()),
                Transform::from_translation(Vec3::new(origin.x as f32, 0.0, origin.z as f32)),
                ChunkMarker { pos: chunk_pos },
            ))
            .id();

        // 保存占位符实体，供后续任务使用
        placeholders.map.insert(chunk_pos, placeholder_entity);
    }
}

/// 派发异步网格生成任务（使用已创建的占位符）
fn spawn_mesh_tasks(
    mut commands: Commands,
    mut queue: ResMut<ChunkLoadQueue>,
    placeholders: ResMut<PlaceholderEntities>,
    seed: Res<WorldSeed>,
) {
    // 限制并发任务数
    let available_slots = queue.max_concurrent_tasks.saturating_sub(queue.active_tasks);
    if available_slots == 0 {
        return;
    }

    let count = queue.to_load.len().min(available_slots);
    let chunks_to_process: Vec<_> = queue.to_load.drain(..count).collect();

    let task_pool = AsyncComputeTaskPool::get();
    let seed_value = seed.seed;

    for chunk_pos in chunks_to_process {
        // 从占位符映射中获取已创建的占位符实体
        let placeholder_entity = match placeholders.map.get(&chunk_pos) {
            Some(&entity) => entity,
            None => {
                // 如果没有占位符（不应该发生），跳过这个区块
                continue;
            }
        };

        // 派发异步任务（包含区块生成和网格构建）
        let task = task_pool.spawn(async move {
            generate_chunk_and_mesh_async(chunk_pos, seed_value)
        });

        // 创建任务跟踪实体
        commands.spawn(ComputeMeshTask {
            task,
            chunk_pos,
            placeholder_entity,
        });

        queue.active_tasks += 1;
    }
}

/// 处理完成的网格生成任务（收集到缓冲区，等待批量替换）
fn handle_completed_mesh_tasks(
    mut commands: Commands,
    mut queue: ResMut<ChunkLoadQueue>,
    mut buffer: ResMut<ChunkReplacementBuffer>,
    mut pending_query: Query<(Entity, &mut ComputeMeshTask)>,
) {
    for (entity, mut task) in pending_query.iter_mut() {
        // 非阻塞地检查任务是否完成
        if let Some((voxels, mesh)) = future::block_on(future::poll_once(&mut task.task)) {
            let chunk_pos = task.chunk_pos;
            let placeholder_entity = task.placeholder_entity;

            // 移除任务跟踪实体
            commands.entity(entity).despawn();
            queue.active_tasks = queue.active_tasks.saturating_sub(1);

            // 收集到缓冲区，等待批量替换
            buffer.completed.push(CompletedChunk {
                chunk_pos,
                voxels,
                mesh,
                placeholder_entity,
            });
        }
    }
}

/// 批量替换占位符为真实区块
/// 按时间间隔或达到批量大小时触发，减少闪烁
fn apply_chunk_replacements(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<ChunkMaterials>,
    mut world: ResMut<VoxelWorld>,
    mut buffer: ResMut<ChunkReplacementBuffer>,
    mut placeholders: ResMut<PlaceholderEntities>,
) {
    if buffer.completed.is_empty() {
        return;
    }

    // 更新定时器
    buffer.timer += time.delta_secs();

    // 检查是否应该批量替换
    let should_replace = buffer.completed.len() >= buffer.min_batch_size
        || buffer.timer >= buffer.interval;

    if !should_replace {
        return;
    }

    // 重置定时器
    buffer.timer = 0.0;

    // 批量替换所有完成的区块
    for completed in buffer.completed.drain(..) {
        // 存储区块数据
        world.chunks.insert(completed.chunk_pos, ChunkData {
            voxels: completed.voxels,
            is_dirty: false,
        });

        // 移除蓝色占位符实体
        commands.entity(completed.placeholder_entity).despawn();
        placeholders.map.remove(&completed.chunk_pos);

        // 创建真实区块渲染实体（替换占位符）
        let mesh_handle = meshes.add(completed.mesh);
        let origin = completed.chunk_pos.world_origin();

        let chunk_entity = commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(materials.opaque.clone()),
                Transform::from_translation(Vec3::new(origin.x as f32, 0.0, origin.z as f32)),
                ChunkMarker { pos: completed.chunk_pos },
            ))
            .id();

        world.loaded_chunks.insert(completed.chunk_pos, chunk_entity);
    }
}

/// 处理区块卸载（包括占位符和任务取消）
fn process_chunk_unload(
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    mut queue: ResMut<ChunkLoadQueue>,
    mut buffer: ResMut<ChunkReplacementBuffer>,
    mut placeholders: ResMut<PlaceholderEntities>,
    pending_query: Query<(Entity, &ComputeMeshTask)>,
) {
    // 先收集要卸载的区块和要取消的任务数
    let chunks_to_unload: Vec<_> = queue.to_unload.drain(..).collect();
    let mut tasks_to_cancel = 0;

    for chunk_pos in chunks_to_unload {
        // 卸载已渲染的区块
        if let Some(entity) = world.loaded_chunks.remove(&chunk_pos) {
            commands.entity(entity).despawn();
        }

        // 取消该区块的待处理任务并删除占位符
        for (entity, task) in pending_query.iter() {
            if task.chunk_pos == chunk_pos {
                // 删除任务跟踪实体
                commands.entity(entity).despawn();
                // 删除蓝色占位符实体
                commands.entity(task.placeholder_entity).despawn();
                tasks_to_cancel += 1;
            }
        }

        // 删除独立的占位符（如果存在）
        if let Some(entity) = placeholders.map.remove(&chunk_pos) {
            commands.entity(entity).despawn();
        }

        // 从替换缓冲区中移除该区块（如果存在）
        buffer.completed.retain(|c| c.chunk_pos != chunk_pos);

        // 从待创建占位符列表中移除（如果存在）
        queue.pending_placeholders.retain(|&pos| pos != chunk_pos);

        world.chunks.remove(&chunk_pos);
    }

    // 更新活跃任务计数
    queue.active_tasks = queue.active_tasks.saturating_sub(tasks_to_cancel);
}

/// 将整数向量转换为浮点向量（辅助函数）
pub fn ivec3_to_vec3(pos: IVec3) -> Vec3 {
    Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32)
}
