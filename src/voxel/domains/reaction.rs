/// 反应规则系统
///
/// 定义了条件判定和命令生成的接口

use bevy::prelude::*;

use super::command::DomainCommand;
use crate::voxel::chunk::ChunkData;

/// 反应规则特征
///
/// 每个规则负责：
/// 1. 判断是否触发（evaluate）
/// 2. 产生命令列表（emit_commands）
pub trait ReactionRule: Send + Sync {
    /// 判断规则是否在指定方块上触发
    ///
    /// 只读访问 ChunkData，不修改状态
    fn evaluate(&self, chunk: &ChunkData, idx: usize) -> bool;

    /// 产生命令列表
    ///
    /// 只读访问 ChunkData，返回需要执行的命令
    fn emit_commands(&self, chunk: &ChunkData, idx: usize) -> Vec<DomainCommand>;
}

/// 反应规则注册表
///
/// 全局资源，包含所有启用的反应规则
#[derive(Resource, Default)]
pub struct ReactionRules {
    pub rules: Vec<Box<dyn ReactionRule>>,
}

// ===== 示例规则（占位，实际规则在各领域模块中实现）=====

/// 示例：日志规则（打印所有方块信息）
pub struct LogRule;

impl ReactionRule for LogRule {
    fn evaluate(&self, _chunk: &ChunkData, _idx: usize) -> bool {
        // 这是一个示例规则，总是返回 false
        false
    }

    fn emit_commands(&self, _chunk: &ChunkData, _idx: usize) -> Vec<DomainCommand> {
        vec![]
    }
}

// TODO: 后续添加实际规则
// - ThermalIgnitionRule: 温度点燃规则
// - FreezingRule: 水冻结规则
// - MeltingRule: 冰融化规则
// - GrowthRule: 植物生长规则
// - CorrosionRule: 金属腐蚀规则
