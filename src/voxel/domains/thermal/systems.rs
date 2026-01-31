//! 热扩散系统
//!
//! 在 FieldUpdate 阶段执行，对活跃的温度方块进行扩散计算

use bevy::prelude::*;

use super::api::{get_valid_neighbor_indices, ThermalApi};
use crate::voxel::chunk::VoxelWorld;
use crate::voxel::domains::SimulationSet;

/// 环境温度（摄氏度）
const ENV_TEMPERATURE: f32 = 20.0;

/// 热扩散系统
///
/// 执行热传导物理模拟：
/// - 相邻方块之间根据导热系数传递热量
/// - 边界方块与环境进行热交换
/// - 温度稳定的方块从活跃集合移除
pub fn thermal_diffusion_system(mut voxel_world: ResMut<VoxelWorld>, time: Res<Time>) {
    let dt = time.delta_secs();

    // 避免在时间暂停时计算
    if dt <= 0.0 {
        return;
    }

    // 遍历所有 chunk
    for chunk in voxel_world.chunks.values_mut() {
        // 只处理有热力学状态的 chunk
        if chunk.active_thermal.is_empty() {
            continue;
        }

        // 复制活跃索引（避免借用冲突）
        let active_indices: Vec<usize> = chunk.active_thermal.iter().copied().collect();

        // 第一遍：计算热量变化（写入 heat_buffer）
        // 确保 thermal_state 存在
        let thermal = chunk.thermal_state.get_or_insert_with(Default::default);

        for &idx in &active_indices {
            let current_temp = if let Some(&t) = thermal.temp_overrides.get(&idx) {
                t
            } else {
                chunk.voxels[idx].def().props.temperature
            };

            let props = chunk.voxels[idx].def().props;

            let mut heat_delta = 0.0;

            // 对有效的邻居进行热传导计算
            for neighbor_idx in get_valid_neighbor_indices(idx) {
                let neighbor_temp = if let Some(&t) = thermal.temp_overrides.get(&neighbor_idx) {
                    t
                } else {
                    chunk.voxels[neighbor_idx].def().props.temperature
                };

                let neighbor_props = chunk.voxels[neighbor_idx].def().props;

                // 热传导公式：Q = k * A * ΔT * dt
                // 这里 A = 1（单位面积），简化计算
                let k_avg =
                    (props.thermal_conductivity + neighbor_props.thermal_conductivity) / 2.0;
                let delta_t = neighbor_temp - current_temp;
                let heat_flow = k_avg * delta_t * dt;

                heat_delta += heat_flow;
            }

            // 环境热交换（边界条件）
            // 只有暴露在空气中的方块才与环境交换
            if props.env_exchange_coef > 0.0 {
                heat_delta += props.env_exchange_coef * (ENV_TEMPERATURE - current_temp) * dt;
            }

            // 存入缓冲区
            if heat_delta.abs() > 0.001 {
                thermal.heat_buffer.insert(idx, heat_delta);
            }
        }

        // 第二遍：应用热量变化
        // 需要再次获取 thermal_state（由于借用规则）
        let heat_changes: Vec<(usize, f32)> = {
            if let Some(thermal) = &chunk.thermal_state {
                thermal
                    .heat_buffer
                    .iter()
                    .map(|(&idx, &heat)| (idx, heat))
                    .collect()
            } else {
                vec![]
            }
        };

        for (idx, heat) in heat_changes {
            ThermalApi::add_heat(chunk, idx, heat);
        }

        // 清空热量缓冲
        if let Some(thermal) = &mut chunk.thermal_state {
            thermal.heat_buffer.clear();
        }

        // 第三遍：清理不再活跃的方块
        let mut to_remove = Vec::new();
        for &idx in &active_indices {
            if !ThermalApi::should_stay_active(chunk, idx) {
                to_remove.push(idx);
            }
        }
        for idx in to_remove {
            chunk.active_thermal.remove(&idx);
        }
    }
}

/// 热源系统
///
/// 处理持续产热的方块（如燃烧中的方块）
pub fn heat_source_system(mut voxel_world: ResMut<VoxelWorld>, time: Res<Time>) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for chunk in voxel_world.chunks.values_mut() {
        // 复制燃烧索引
        let burning_indices: Vec<usize> = chunk.active_burning.iter().copied().collect();

        for idx in burning_indices {
            let props = chunk.voxels[idx].def().props;

            // 燃烧释放热量
            if props.is_flammable && props.heat_release > 0.0 {
                let heat = props.heat_release * dt;

                // 热量分配：自身 50%，周围 50% 均分
                ThermalApi::add_heat(chunk, idx, heat * 0.5);

                let neighbors = get_valid_neighbor_indices(idx);
                let heat_per_neighbor = (heat * 0.5) / neighbors.len() as f32;

                for neighbor_idx in neighbors {
                    ThermalApi::add_heat(chunk, neighbor_idx, heat_per_neighbor);
                }
            }
        }
    }
}

/// 热力学插件
pub struct ThermalPlugin;

impl Plugin for ThermalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (thermal_diffusion_system, heat_source_system)
                .chain()
                .in_set(SimulationSet::FieldUpdate),
        );
    }
}
