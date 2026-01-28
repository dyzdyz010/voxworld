//! 体素相关组件

use bevy::prelude::*;

use crate::voxel::voxel_kind::VoxelKind;

/// 体素组件 - 用于射线检测和交互
#[derive(Component, Debug, Clone, Copy)]
pub struct Voxel {
    /// 体素类型
    pub kind: VoxelKind,
    /// 体素的世界坐标
    pub pos: IVec3,
}
