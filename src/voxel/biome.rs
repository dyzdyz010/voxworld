//! 生物群系定义

use crate::voxel::voxel_kind::VoxelKind;

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
    FloatingIslands, // 浮空岛生物群系
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
            Biome::FloatingIslands => VoxelKind::Grass, // 浮空岛顶部是草地
        }
    }

    /// 获取该生物群系的次表层方块类型（表层下方的方块）
    pub fn subsurface_block(self) -> VoxelKind {
        match self {
            Biome::Plains | Biome::Forest | Biome::BirchForest | Biome::Taiga => VoxelKind::Dirt,
            Biome::Desert | Biome::Beach => VoxelKind::Sand,
            Biome::Snowy => VoxelKind::Dirt,
            Biome::Ocean => VoxelKind::Clay,
            Biome::FloatingIslands => VoxelKind::Dirt, // 浮空岛次表层是泥土
        }
    }
}
