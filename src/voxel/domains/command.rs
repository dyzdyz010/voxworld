/// 领域命令系统
///
/// 定义了跨域交互的唯一接口：DomainCommand
/// 所有领域系统只能通过产出命令来修改状态

use bevy::prelude::*;

use super::thermal::ThermalApi;
use crate::voxel::change::BlockChange;
use crate::voxel::chunk::ChunkPos;
use crate::voxel::flags::VoxelFlags;
use crate::voxel::voxel_kind::VoxelKind;

/// 统一的领域命令
///
/// 这是领域间交互的唯一接口，所有状态修改都必须通过命令提交
#[derive(Clone, Debug)]
pub enum DomainCommand {
    // === 方块操作 ===
    /// 设置方块类型
    SetBlock { idx: usize, new_voxel: VoxelKind },

    // === 标志位操作 ===
    /// 添加标志位
    AddFlag { idx: usize, flag: VoxelFlags },

    /// 移除标志位
    RemoveFlag { idx: usize, flag: VoxelFlags },

    // === 变体操作 ===
    /// 设置变体值
    SetVariant { idx: usize, variant: u8 },

    /// 变体值 +1
    IncrementVariant { idx: usize },

    /// 变体值 -1
    DecrementVariant { idx: usize },

    // === 热力学操作 ===
    /// 设置温度
    SetTemp { idx: usize, temp: f32 },

    /// 添加热量
    AddHeat { idx: usize, heat: f32 },

    // === 湿度操作 ===
    /// 设置湿度
    SetMoisture { idx: usize, moisture: f32 },

    /// 添加湿度
    AddMoisture { idx: usize, delta: f32 },

    // === 燃烧操作 ===
    /// 点燃方块
    Ignite { idx: usize, power: f32 },

    /// 熄灭方块
    Extinguish { idx: usize },

    // === 相变操作（占位，后续实现）===
    // StartPhaseTransition { idx: usize, phase: PhaseTransition },
    // CompletePhaseTransition { idx: usize },

    // === 结构操作（占位，后续实现）===
    // Damage { idx: usize, amount: f32 },
    // Collapse { idx: usize },
}

/// 命令队列组件
///
/// 全局单例，所有领域系统向这个队列提交命令
#[derive(Component, Default)]
pub struct CommandQueue {
    pub commands: Vec<DomainCommand>,
}

/// 带 chunk 位置的命令
#[derive(Clone, Debug)]
pub struct ChunkCommand {
    pub chunk_pos: ChunkPos,
    pub command: DomainCommand,
}

impl CommandQueue {
    /// 添加带 chunk 位置的命令
    pub fn push(&mut self, chunk_pos: ChunkPos, command: DomainCommand) {
        self.commands.push(command);
        // 注意：简化实现，假设所有命令都针对同一个 chunk
        // 实际实现需要存储 ChunkCommand
    }
}

/// 统一提交系统
///
/// 在 SimulationSet::Commit 阶段执行，处理所有命令
pub fn commit_system(
    mut voxel_world: ResMut<crate::voxel::VoxelWorld>,
    mut command_queues: Query<&mut CommandQueue>,
) {
    // 获取命令队列（如果存在）
    let Some(mut queue) = command_queues.iter_mut().next() else {
        return;
    };

    let commands: Vec<DomainCommand> = std::mem::take(&mut queue.commands);

    if commands.is_empty() {
        return;
    }

    // 解析冲突并执行命令
    let resolved = resolve_conflicts(commands);

    // 遍历所有 chunk 执行命令
    for chunk in voxel_world.chunks.values_mut() {
        for cmd in &resolved {
            execute_command(chunk, cmd);
        }
    }
}

/// 解析命令冲突
///
/// 优先级：SetBlock > 其他
fn resolve_conflicts(commands: Vec<DomainCommand>) -> Vec<DomainCommand> {
    use std::collections::HashMap;

    // 按 idx 分组
    let mut per_idx: HashMap<usize, Vec<DomainCommand>> = HashMap::new();
    for cmd in commands {
        let idx = cmd.idx();
        per_idx.entry(idx).or_default().push(cmd);
    }

    let mut resolved = Vec::new();
    for (_idx, cmds) in per_idx {
        // SetBlock 优先级最高，只保留第一个
        if let Some(set_block) = cmds
            .iter()
            .find(|c| matches!(c, DomainCommand::SetBlock { .. }))
        {
            resolved.push(set_block.clone());
            continue;
        }

        // 其他命令按顺序执行
        resolved.extend(cmds);
    }

    resolved
}

/// 在 chunk 上执行单条命令
fn execute_command(chunk: &mut crate::voxel::ChunkData, cmd: &DomainCommand) {
    match cmd {
        DomainCommand::SetBlock { idx, new_voxel } => {
            if *idx < chunk.voxels.len() {
                let old = chunk.voxels[*idx];
                chunk.voxels[*idx] = *new_voxel;
                chunk.changes.push(BlockChange::SetVoxel {
                    idx: *idx,
                    old,
                    new: *new_voxel,
                });
                chunk.dirty_blocks.push(*idx);
                chunk.needs_remesh = true;
                chunk.is_dirty = true;
            }
        }

        DomainCommand::AddFlag { idx, flag } => {
            if *idx < chunk.flags.len() {
                chunk.flags[*idx].insert(*flag);
                chunk.changes.push(BlockChange::SetFlag {
                    idx: *idx,
                    flag: *flag,
                    set: true,
                });
                chunk.dirty_blocks.push(*idx);
            }
        }

        DomainCommand::RemoveFlag { idx, flag } => {
            if *idx < chunk.flags.len() {
                chunk.flags[*idx].remove(*flag);
                chunk.changes.push(BlockChange::SetFlag {
                    idx: *idx,
                    flag: *flag,
                    set: false,
                });
                chunk.dirty_blocks.push(*idx);
            }
        }

        DomainCommand::SetVariant { idx, variant } => {
            if *idx < chunk.variant.len() {
                let old = chunk.variant[*idx];
                chunk.variant[*idx] = *variant;
                chunk.changes.push(BlockChange::SetVariant {
                    idx: *idx,
                    old,
                    new: *variant,
                });
                chunk.dirty_blocks.push(*idx);
            }
        }

        DomainCommand::IncrementVariant { idx } => {
            if *idx < chunk.variant.len() {
                let old = chunk.variant[*idx];
                chunk.variant[*idx] = old.saturating_add(1);
                chunk.changes.push(BlockChange::SetVariant {
                    idx: *idx,
                    old,
                    new: chunk.variant[*idx],
                });
                chunk.dirty_blocks.push(*idx);
            }
        }

        DomainCommand::DecrementVariant { idx } => {
            if *idx < chunk.variant.len() {
                let old = chunk.variant[*idx];
                chunk.variant[*idx] = old.saturating_sub(1);
                chunk.changes.push(BlockChange::SetVariant {
                    idx: *idx,
                    old,
                    new: chunk.variant[*idx],
                });
                chunk.dirty_blocks.push(*idx);
            }
        }

        DomainCommand::SetTemp { idx, temp } => {
            if *idx < chunk.voxels.len() {
                ThermalApi::set_temp(chunk, *idx, *temp);
            }
        }

        DomainCommand::AddHeat { idx, heat } => {
            if *idx < chunk.voxels.len() {
                ThermalApi::add_heat(chunk, *idx, *heat);
            }
        }

        DomainCommand::SetMoisture { idx, moisture } => {
            // TODO: 实现湿度 API
            if *idx < chunk.voxels.len() {
                chunk.changes.push(BlockChange::SetMoisture {
                    idx: *idx,
                    moisture: *moisture,
                });
            }
        }

        DomainCommand::AddMoisture { idx, delta: _ } => {
            // TODO: 实现湿度 API
            if *idx < chunk.voxels.len() {
                // 暂时不处理
            }
        }

        DomainCommand::Ignite { idx, power: _ } => {
            if *idx < chunk.flags.len() {
                chunk.flags[*idx].insert(VoxelFlags::BURNING);
                chunk.active_burning.insert(*idx);
                chunk.changes.push(BlockChange::SetFlag {
                    idx: *idx,
                    flag: VoxelFlags::BURNING,
                    set: true,
                });
                chunk.dirty_blocks.push(*idx);
            }
        }

        DomainCommand::Extinguish { idx } => {
            if *idx < chunk.flags.len() {
                chunk.flags[*idx].remove(VoxelFlags::BURNING);
                chunk.active_burning.remove(idx);
                chunk.changes.push(BlockChange::SetFlag {
                    idx: *idx,
                    flag: VoxelFlags::BURNING,
                    set: false,
                });
                chunk.dirty_blocks.push(*idx);
            }
        }
    }
}

impl DomainCommand {
    /// 获取命令影响的方块索引
    pub fn idx(&self) -> usize {
        match self {
            DomainCommand::SetBlock { idx, .. } => *idx,
            DomainCommand::AddFlag { idx, .. } => *idx,
            DomainCommand::RemoveFlag { idx, .. } => *idx,
            DomainCommand::SetVariant { idx, .. } => *idx,
            DomainCommand::IncrementVariant { idx } => *idx,
            DomainCommand::DecrementVariant { idx } => *idx,
            DomainCommand::SetTemp { idx, .. } => *idx,
            DomainCommand::AddHeat { idx, .. } => *idx,
            DomainCommand::SetMoisture { idx, .. } => *idx,
            DomainCommand::AddMoisture { idx, .. } => *idx,
            DomainCommand::Ignite { idx, .. } => *idx,
            DomainCommand::Extinguish { idx } => *idx,
        }
    }
}
