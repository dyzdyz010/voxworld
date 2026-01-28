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
    // 优化：使用unlit以减少光照计算开销
    let opaque = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        unlit: false, // 保持光照以获得更好的视觉效果
        cull_mode: Some(bevy::render::render_resource::Face::Back), // 背面剔除
        ..default()
    });

    // 透明材质：低粗糙度，支持透明混合
    let transparent = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.3,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None, // 透明物体不剔除
        ..default()
    });

    commands.insert_resource(ChunkMaterials { opaque, transparent });
}
