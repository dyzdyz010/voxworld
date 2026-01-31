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
    // === 热学属性 ===
    /// 默认温度（摄氏度）
    pub temperature: f32,
    /// 热容（J/K），值越大温度变化越慢
    pub heat_capacity: f32,
    /// 导热系数（W/m·K），值越大热量传递越快
    pub thermal_conductivity: f32,
    /// 环境热交换系数（0.0-1.0），值越大与环境交换越快
    pub env_exchange_coef: f32,

    // === 湿度属性 ===
    /// 默认湿度（0.0-1.0）
    pub humidity: f32,
    /// 吸湿能力（0.0-1.0）
    pub moisture_capacity: f32,
    /// 蒸发速率（每秒）
    pub evaporation_rate: f32,

    // === 燃烧属性 ===
    /// 是否可燃
    pub is_flammable: bool,
    /// 着火点（°C）
    pub ignition_temp: f32,
    /// 总燃烧能量（焦耳）
    pub burn_energy: f32,
    /// 燃烧速率（1/秒）
    pub burn_rate: f32,
    /// 热释放率（W），燃烧时每秒释放的热量
    pub heat_release: f32,

    // === 相变属性 ===
    /// 熔点（°C），None 表示不会熔化
    pub melting_point: Option<f32>,
    /// 冰点（°C），None 表示不会冻结
    pub freezing_point: Option<f32>,
    /// 沸点（°C），None 表示不会沸腾
    pub boiling_point: Option<f32>,
    /// 液态形式
    pub liquid_form: Option<VoxelKind>,
    /// 固态形式
    pub solid_form: Option<VoxelKind>,

    // === 结构属性 ===
    /// 硬度（0.0-1.0，值越大越难破坏）
    pub hardness: f32,
    /// 延展性（0.0-1.0，值越大越容易变形）
    pub ductility: f32,
    /// 结构完整性（0.0-1.0）
    pub integrity: f32,
    /// 抗腐蚀性（0.0-1.0）
    pub corrosion_resistance: f32,

    // === 生长属性 ===
    /// 是否可生长
    pub is_growable: bool,
    /// 生长速率
    pub growth_rate: f32,
    /// 最大生长阶段
    pub max_growth_stage: u8,
}

impl Default for VoxelProperties {
    fn default() -> Self {
        Self {
            // 热学
            temperature: 20.0,
            heat_capacity: 1000.0,
            thermal_conductivity: 0.5,
            env_exchange_coef: 0.0,
            // 湿度
            humidity: 0.5,
            moisture_capacity: 0.5,
            evaporation_rate: 0.0,
            // 燃烧
            is_flammable: false,
            ignition_temp: 1000.0,
            burn_energy: 0.0,
            burn_rate: 0.0,
            heat_release: 0.0,
            // 相变
            melting_point: None,
            freezing_point: None,
            boiling_point: None,
            liquid_form: None,
            solid_form: None,
            // 结构
            hardness: 0.5,
            ductility: 0.5,
            integrity: 1.0,
            corrosion_resistance: 0.5,
            // 生长
            is_growable: false,
            growth_rate: 0.0,
            max_growth_stage: 0,
        }
    }
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
                    heat_capacity: 1.0,
                    thermal_conductivity: 0.026, // 空气导热系数很低
                    env_exchange_coef: 0.0,
                    humidity: 0.5,
                    hardness: 0.0,
                    ductility: 0.0,
                    ..Default::default()
                },
            },
            VoxelKind::Grass => VoxelDef {
                name: "草方块",
                color: Color::srgb(0.28, 0.62, 0.25),
                props: VoxelProperties {
                    temperature: 18.0,
                    heat_capacity: 800.0,
                    thermal_conductivity: 0.25,
                    env_exchange_coef: 0.1,
                    humidity: 0.6,
                    moisture_capacity: 0.4,
                    is_flammable: true,
                    ignition_temp: 400.0,
                    burn_energy: 30.0,
                    burn_rate: 0.3,
                    heat_release: 50.0,
                    hardness: 0.2,
                    ductility: 0.35,
                    ..Default::default()
                },
            },
            VoxelKind::Dirt => VoxelDef {
                name: "泥土",
                color: Color::srgb(0.42, 0.30, 0.18),
                props: VoxelProperties {
                    temperature: 16.0,
                    heat_capacity: 1500.0,
                    thermal_conductivity: 0.8,
                    env_exchange_coef: 0.05,
                    humidity: 0.4,
                    moisture_capacity: 0.6,
                    hardness: 0.35,
                    ductility: 0.2,
                    ..Default::default()
                },
            },
            VoxelKind::Stone => VoxelDef {
                name: "石头",
                color: Color::srgb(0.55, 0.55, 0.58),
                props: VoxelProperties {
                    temperature: 12.0,
                    heat_capacity: 2000.0,
                    thermal_conductivity: 2.5, // 石头导热好
                    env_exchange_coef: 0.02,
                    humidity: 0.1,
                    melting_point: Some(1200.0), // 石头熔点
                    hardness: 0.9,
                    ductility: 0.05,
                    integrity: 1.0,
                    corrosion_resistance: 0.9,
                    ..Default::default()
                },
            },
            VoxelKind::Sand => VoxelDef {
                name: "沙子",
                color: Color::srgb(0.86, 0.82, 0.58),
                props: VoxelProperties {
                    temperature: 28.0,
                    heat_capacity: 830.0,
                    thermal_conductivity: 0.25,
                    env_exchange_coef: 0.15,
                    humidity: 0.05,
                    evaporation_rate: 0.2,
                    melting_point: Some(1700.0), // 沙子熔点（变玻璃）
                    hardness: 0.25,
                    ductility: 0.45,
                    integrity: 0.3, // 沙子结构不稳
                    ..Default::default()
                },
            },
            VoxelKind::Gravel => VoxelDef {
                name: "砂砾",
                color: Color::srgb(0.52, 0.50, 0.48),
                props: VoxelProperties {
                    temperature: 14.0,
                    heat_capacity: 1200.0,
                    thermal_conductivity: 1.5,
                    env_exchange_coef: 0.05,
                    humidity: 0.15,
                    hardness: 0.4,
                    ductility: 0.3,
                    integrity: 0.4,
                    ..Default::default()
                },
            },
            VoxelKind::Clay => VoxelDef {
                name: "黏土",
                color: Color::srgb(0.62, 0.64, 0.68),
                props: VoxelProperties {
                    temperature: 15.0,
                    heat_capacity: 900.0,
                    thermal_conductivity: 1.0,
                    env_exchange_coef: 0.03,
                    humidity: 0.7,
                    moisture_capacity: 0.8,
                    hardness: 0.3,
                    ductility: 0.5,
                    ..Default::default()
                },
            },
            VoxelKind::Snow => VoxelDef {
                name: "雪块",
                color: Color::srgb(0.95, 0.97, 1.0),
                props: VoxelProperties {
                    temperature: -5.0,
                    heat_capacity: 2090.0, // 冰的热容
                    thermal_conductivity: 0.1, // 雪导热差
                    env_exchange_coef: 0.2,
                    humidity: 0.8,
                    melting_point: Some(0.0),
                    liquid_form: Some(VoxelKind::Water),
                    hardness: 0.1,
                    ductility: 0.2,
                    ..Default::default()
                },
            },
            VoxelKind::Ice => VoxelDef {
                name: "冰块",
                color: Color::srgba(0.68, 0.85, 0.95, 0.85),
                props: VoxelProperties {
                    temperature: -10.0,
                    heat_capacity: 2090.0,
                    thermal_conductivity: 2.22, // 冰导热系数
                    env_exchange_coef: 0.1,
                    humidity: 0.9,
                    melting_point: Some(0.0),
                    liquid_form: Some(VoxelKind::Water),
                    hardness: 0.3,
                    ductility: 0.1,
                    ..Default::default()
                },
            },
            VoxelKind::Water => VoxelDef {
                name: "水",
                color: Color::srgba(0.20, 0.45, 0.78, 0.7),
                props: VoxelProperties {
                    temperature: 14.0,
                    heat_capacity: 4186.0, // 水的比热容
                    thermal_conductivity: 0.6,
                    env_exchange_coef: 0.3,
                    humidity: 1.0,
                    evaporation_rate: 0.1,
                    freezing_point: Some(0.0),
                    boiling_point: Some(100.0),
                    solid_form: Some(VoxelKind::Ice),
                    hardness: 0.0,
                    ductility: 1.0,
                    ..Default::default()
                },
            },
            VoxelKind::OakLog => VoxelDef {
                name: "橡木原木",
                color: Color::srgb(0.40, 0.30, 0.18),
                props: VoxelProperties {
                    temperature: 20.0,
                    heat_capacity: 1700.0,
                    thermal_conductivity: 0.12, // 木材导热差
                    env_exchange_coef: 0.05,
                    humidity: 0.3,
                    moisture_capacity: 0.5,
                    is_flammable: true,
                    ignition_temp: 300.0, // 木材着火点
                    burn_energy: 100.0,
                    burn_rate: 0.2,
                    heat_release: 200.0,
                    hardness: 0.5,
                    ductility: 0.6,
                    corrosion_resistance: 0.3,
                    ..Default::default()
                },
            },
            VoxelKind::OakLeaves => VoxelDef {
                name: "橡树树叶",
                color: Color::srgba(0.22, 0.52, 0.20, 0.9),
                props: VoxelProperties {
                    temperature: 22.0,
                    heat_capacity: 500.0,
                    thermal_conductivity: 0.05,
                    env_exchange_coef: 0.3,
                    humidity: 0.5,
                    is_flammable: true,
                    ignition_temp: 250.0, // 树叶更易燃
                    burn_energy: 20.0,
                    burn_rate: 0.5, // 树叶烧得快
                    heat_release: 100.0,
                    hardness: 0.05,
                    ductility: 0.1,
                    ..Default::default()
                },
            },
            VoxelKind::BirchLog => VoxelDef {
                name: "白桦原木",
                color: Color::srgb(0.85, 0.82, 0.75),
                props: VoxelProperties {
                    temperature: 18.0,
                    heat_capacity: 1600.0,
                    thermal_conductivity: 0.14,
                    env_exchange_coef: 0.05,
                    humidity: 0.35,
                    is_flammable: true,
                    ignition_temp: 280.0,
                    burn_energy: 90.0,
                    burn_rate: 0.22,
                    heat_release: 180.0,
                    hardness: 0.45,
                    ductility: 0.55,
                    ..Default::default()
                },
            },
            VoxelKind::BirchLeaves => VoxelDef {
                name: "白桦树叶",
                color: Color::srgba(0.45, 0.62, 0.35, 0.9),
                props: VoxelProperties {
                    temperature: 20.0,
                    heat_capacity: 500.0,
                    thermal_conductivity: 0.05,
                    env_exchange_coef: 0.3,
                    humidity: 0.45,
                    is_flammable: true,
                    ignition_temp: 240.0,
                    burn_energy: 18.0,
                    burn_rate: 0.55,
                    heat_release: 90.0,
                    hardness: 0.05,
                    ductility: 0.1,
                    ..Default::default()
                },
            },
            VoxelKind::SpruceLog => VoxelDef {
                name: "云杉原木",
                color: Color::srgb(0.30, 0.22, 0.12),
                props: VoxelProperties {
                    temperature: 8.0,
                    heat_capacity: 1800.0,
                    thermal_conductivity: 0.11,
                    env_exchange_coef: 0.04,
                    humidity: 0.4,
                    is_flammable: true,
                    ignition_temp: 320.0,
                    burn_energy: 110.0,
                    burn_rate: 0.18,
                    heat_release: 220.0,
                    hardness: 0.55,
                    ductility: 0.5,
                    ..Default::default()
                },
            },
            VoxelKind::SpruceLeaves => VoxelDef {
                name: "云杉树叶",
                color: Color::srgba(0.15, 0.35, 0.22, 0.9),
                props: VoxelProperties {
                    temperature: 6.0,
                    heat_capacity: 550.0,
                    thermal_conductivity: 0.06,
                    env_exchange_coef: 0.25,
                    humidity: 0.5,
                    is_flammable: true,
                    ignition_temp: 260.0,
                    burn_energy: 25.0,
                    burn_rate: 0.45,
                    heat_release: 110.0,
                    hardness: 0.05,
                    ductility: 0.1,
                    ..Default::default()
                },
            },
            VoxelKind::Cactus => VoxelDef {
                name: "仙人掌",
                color: Color::srgb(0.25, 0.55, 0.20),
                props: VoxelProperties {
                    temperature: 35.0,
                    heat_capacity: 3500.0, // 仙人掌含水量高
                    thermal_conductivity: 0.5,
                    env_exchange_coef: 0.1,
                    humidity: 0.1,
                    moisture_capacity: 0.9, // 仙人掌储水
                    is_flammable: false, // 太湿，不易燃
                    is_growable: true,
                    growth_rate: 0.01,
                    max_growth_stage: 3,
                    hardness: 0.2,
                    ductility: 0.3,
                    ..Default::default()
                },
            },
            VoxelKind::CoalOre => VoxelDef {
                name: "煤矿石",
                color: Color::srgb(0.25, 0.25, 0.28),
                props: VoxelProperties {
                    temperature: 12.0,
                    heat_capacity: 1300.0,
                    thermal_conductivity: 0.2, // 煤导热差
                    env_exchange_coef: 0.02,
                    humidity: 0.1,
                    is_flammable: true,
                    ignition_temp: 450.0, // 煤着火点高
                    burn_energy: 500.0, // 煤能量高
                    burn_rate: 0.05, // 煤烧得慢
                    heat_release: 300.0,
                    hardness: 0.85,
                    ductility: 0.05,
                    ..Default::default()
                },
            },
            VoxelKind::IronOre => VoxelDef {
                name: "铁矿石",
                color: Color::srgb(0.58, 0.52, 0.48),
                props: VoxelProperties {
                    temperature: 12.0,
                    heat_capacity: 450.0, // 铁热容低
                    thermal_conductivity: 80.0, // 铁导热极好
                    env_exchange_coef: 0.02,
                    humidity: 0.1,
                    melting_point: Some(1538.0), // 铁熔点
                    hardness: 0.9,
                    ductility: 0.05,
                    corrosion_resistance: 0.3, // 铁容易生锈
                    ..Default::default()
                },
            },
            VoxelKind::GoldOre => VoxelDef {
                name: "金矿石",
                color: Color::srgb(0.72, 0.65, 0.35),
                props: VoxelProperties {
                    temperature: 12.0,
                    heat_capacity: 129.0, // 金热容很低
                    thermal_conductivity: 317.0, // 金导热极好
                    env_exchange_coef: 0.02,
                    humidity: 0.1,
                    melting_point: Some(1064.0),
                    hardness: 0.85,
                    ductility: 0.15,
                    corrosion_resistance: 1.0, // 金不腐蚀
                    ..Default::default()
                },
            },
            VoxelKind::DiamondOre => VoxelDef {
                name: "钻石矿石",
                color: Color::srgb(0.45, 0.72, 0.78),
                props: VoxelProperties {
                    temperature: 12.0,
                    heat_capacity: 509.0,
                    thermal_conductivity: 2200.0, // 钻石导热极好
                    env_exchange_coef: 0.01,
                    humidity: 0.1,
                    hardness: 0.98,
                    ductility: 0.02,
                    corrosion_resistance: 1.0,
                    ..Default::default()
                },
            },
            VoxelKind::Flower => VoxelDef {
                name: "花",
                color: Color::srgb(0.85, 0.35, 0.40),
                props: VoxelProperties {
                    temperature: 22.0,
                    heat_capacity: 300.0,
                    thermal_conductivity: 0.1,
                    env_exchange_coef: 0.5,
                    humidity: 0.6,
                    is_flammable: true,
                    ignition_temp: 200.0,
                    burn_energy: 5.0,
                    burn_rate: 1.0,
                    heat_release: 20.0,
                    is_growable: true,
                    growth_rate: 0.05,
                    max_growth_stage: 2,
                    hardness: 0.01,
                    ductility: 0.05,
                    ..Default::default()
                },
            },
            VoxelKind::TallGrass => VoxelDef {
                name: "高草丛",
                color: Color::srgb(0.35, 0.58, 0.28),
                props: VoxelProperties {
                    temperature: 20.0,
                    heat_capacity: 200.0,
                    thermal_conductivity: 0.08,
                    env_exchange_coef: 0.6,
                    humidity: 0.5,
                    is_flammable: true,
                    ignition_temp: 180.0,
                    burn_energy: 8.0,
                    burn_rate: 0.8,
                    heat_release: 30.0,
                    is_growable: true,
                    growth_rate: 0.1,
                    max_growth_stage: 2,
                    hardness: 0.01,
                    ductility: 0.05,
                    ..Default::default()
                },
            },
            VoxelKind::DeadBush => VoxelDef {
                name: "枯死的灌木",
                color: Color::srgb(0.55, 0.45, 0.28),
                props: VoxelProperties {
                    temperature: 32.0,
                    heat_capacity: 150.0,
                    thermal_conductivity: 0.05,
                    env_exchange_coef: 0.4,
                    humidity: 0.05,
                    is_flammable: true,
                    ignition_temp: 150.0, // 干燥，极易燃
                    burn_energy: 10.0,
                    burn_rate: 1.0,
                    heat_release: 40.0,
                    hardness: 0.01,
                    ductility: 0.02,
                    ..Default::default()
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
