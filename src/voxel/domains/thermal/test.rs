//! 热力学测试系统
//!
//! 提供交互式热力学测试功能：
//! - F5: 在玩家位置附近创建热源
//! - F6: 显示当前位置的温度信息
//! - F7: 清除所有温度覆盖

use bevy::prelude::*;

use super::api::ThermalApi;
use crate::voxel::chunk::{ChunkData, ChunkPos, VoxelWorld};
use crate::voxel::constants::CHUNK_SIZE;

/// 热力学测试插件
pub struct ThermalTestPlugin;

impl Plugin for ThermalTestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                create_heat_source_system,
                show_temperature_info_system,
                clear_thermal_state_system,
            ),
        );
    }
}

/// 创建热源系统
///
/// 按 F5 在世界原点附近的 chunk 中创建一个热源
fn create_heat_source_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut voxel_world: ResMut<VoxelWorld>,
) {
    if keyboard.just_pressed(KeyCode::F5) {
        // 获取原点 chunk
        let chunk_pos = ChunkPos::new(0, 0, 0);

        if let Some(chunk) = voxel_world.chunks.get_mut(&chunk_pos) {
            // 在 chunk 中心创建热源
            let center_idx = ChunkData::index(8, 8, 8);

            // 设置高温（500°C）
            ThermalApi::set_temp(chunk, center_idx, 500.0);

            // 激活周围方块
            ThermalApi::activate(chunk, center_idx);

            info!(
                "Created heat source at chunk {:?}, idx {}, temp = 500°C",
                chunk_pos, center_idx
            );
            info!(
                "Active thermal blocks: {}",
                chunk.active_thermal.len()
            );
        } else {
            warn!("Chunk at origin not loaded, cannot create heat source");
        }
    }
}

/// 显示温度信息系统
///
/// 按 F6 显示原点 chunk 中心区域的温度信息
fn show_temperature_info_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    voxel_world: Res<VoxelWorld>,
) {
    if keyboard.just_pressed(KeyCode::F6) {
        let chunk_pos = ChunkPos::new(0, 0, 0);

        if let Some(chunk) = voxel_world.chunks.get(&chunk_pos) {
            info!("=== Temperature Info (Chunk {:?}) ===", chunk_pos);
            info!("Active thermal: {}", chunk.active_thermal.len());

            // 显示中心 3x3x3 区域的温度
            for y in 7..=9 {
                let mut line = format!("Y={}: ", y);
                for z in 7..=9 {
                    for x in 7..=9 {
                        let idx = ChunkData::index(x, y, z);
                        let temp = ThermalApi::get_temp(chunk, idx);
                        line.push_str(&format!("{:.1} ", temp));
                    }
                    line.push_str("| ");
                }
                info!("{}", line);
            }

            // 显示活跃方块的温度统计
            if !chunk.active_thermal.is_empty() {
                let temps: Vec<f32> = chunk
                    .active_thermal
                    .iter()
                    .map(|&idx| ThermalApi::get_temp(chunk, idx))
                    .collect();

                let max_temp = temps.iter().cloned().fold(f32::MIN, f32::max);
                let min_temp = temps.iter().cloned().fold(f32::MAX, f32::min);
                let avg_temp = temps.iter().sum::<f32>() / temps.len() as f32;

                info!(
                    "Temperature stats: min={:.1}°C, max={:.1}°C, avg={:.1}°C",
                    min_temp, max_temp, avg_temp
                );
            }
        } else {
            warn!("Chunk at origin not loaded");
        }
    }
}

/// 清除温度状态系统
///
/// 按 F7 清除所有 chunk 的温度覆盖
fn clear_thermal_state_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut voxel_world: ResMut<VoxelWorld>,
) {
    if keyboard.just_pressed(KeyCode::F7) {
        let mut cleared_count = 0;

        for chunk in voxel_world.chunks.values_mut() {
            if let Some(thermal) = &mut chunk.thermal_state {
                cleared_count += thermal.temp_overrides.len();
                thermal.clear();
            }
            chunk.active_thermal.clear();
        }

        info!("Cleared {} temperature overrides", cleared_count);
    }
}

/// 创建温度可视化颜色
///
/// 将温度映射到颜色：冷（蓝色）-> 常温（绿色）-> 热（红色）
pub fn temp_to_color(temp: f32) -> Color {
    // 温度范围：-50°C 到 500°C
    let normalized = ((temp + 50.0) / 550.0).clamp(0.0, 1.0);

    if normalized < 0.5 {
        // 冷到常温：蓝色 -> 绿色
        let t = normalized * 2.0;
        Color::srgb(0.0, t, 1.0 - t)
    } else {
        // 常温到热：绿色 -> 红色
        let t = (normalized - 0.5) * 2.0;
        Color::srgb(t, 1.0 - t, 0.0)
    }
}

/// 生成多个测试热源
///
/// 在 chunk 中创建多个不同温度的热源用于测试扩散效果
pub fn create_test_heat_pattern(chunk: &mut ChunkData) {
    // 热源 1：中心高温
    let center_idx = ChunkData::index(8, 8, 8);
    ThermalApi::set_temp(chunk, center_idx, 500.0);
    ThermalApi::activate(chunk, center_idx);

    // 热源 2：角落中温
    let corner_idx = ChunkData::index(2, 8, 2);
    ThermalApi::set_temp(chunk, corner_idx, 200.0);
    ThermalApi::activate(chunk, corner_idx);

    // 冷源：另一个角落低温
    let cold_idx = ChunkData::index(14, 8, 14);
    ThermalApi::set_temp(chunk, cold_idx, -30.0);
    ThermalApi::activate(chunk, cold_idx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::voxel_kind::VoxelKind;

    #[test]
    fn test_thermal_api_basic() {
        let mut chunk = ChunkData::new();

        // 默认温度应该是 Air 的温度
        let default_temp = ThermalApi::get_temp(&chunk, 0);
        assert!((default_temp - 20.0).abs() < 0.1);

        // 设置温度
        ThermalApi::set_temp(&mut chunk, 0, 100.0);
        let new_temp = ThermalApi::get_temp(&chunk, 0);
        assert!((new_temp - 100.0).abs() < 0.1);

        // 应该被添加到活跃集合
        assert!(chunk.active_thermal.contains(&0));
    }

    #[test]
    fn test_thermal_api_heat_capacity() {
        let mut chunk = ChunkData::new();

        // 将一个方块设为石头（热容 2000）
        chunk.voxels[0] = VoxelKind::Stone;

        let initial_temp = ThermalApi::get_temp(&chunk, 0);

        // 添加热量
        ThermalApi::add_heat(&mut chunk, 0, 2000.0); // 添加 2000J

        let new_temp = ThermalApi::get_temp(&chunk, 0);

        // ΔT = Q / C = 2000 / 2000 = 1°C
        assert!((new_temp - initial_temp - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_temp_to_color() {
        // 冷色
        let cold_color = temp_to_color(-50.0);
        assert!(cold_color.to_srgba().blue > 0.9);

        // 热色
        let hot_color = temp_to_color(500.0);
        assert!(hot_color.to_srgba().red > 0.9);
    }
}
