//! 体素（方块）类型定义

use bevy::prelude::*;

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
        !matches!(
            self,
            VoxelKind::Air | VoxelKind::Flower | VoxelKind::TallGrass | VoxelKind::DeadBush
        )
    }
}
