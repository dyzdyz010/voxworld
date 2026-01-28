//! 材质系统

use bevy::prelude::*;

/// 区块材质资源 - 存储不透明和透明材质的句柄
#[derive(Resource)]
pub struct ChunkMaterials {
    /// 不透明材质（用于大多数方块）
    pub opaque: Handle<StandardMaterial>,
    /// 透明材质（用于水、冰、树叶等）
    pub transparent: Handle<StandardMaterial>,
}

/// 初始化材质系统
/// 创建不透明和透明两种材质
pub fn setup_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
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
