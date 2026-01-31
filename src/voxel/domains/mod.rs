/// 领域模块系统
///
/// 每个领域（Domain）代表一条物理/属性线：
/// - thermal: 温度场
/// - moisture: 湿度场
/// - combustion: 燃烧系统
/// - phase: 相变系统
/// - reaction: 反应规则与命令系统

use bevy::prelude::*;

pub mod command;
pub mod reaction;
pub mod thermal;

// TODO: 后续添加
// pub mod moisture;
// pub mod combustion;
// pub mod phase;

/// 模拟系统执行顺序
///
/// 所有领域系统必须注册到指定的 SystemSet 中，禁止跨阶段写入
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    /// 1. 外部输入（玩家、脚本、网络）
    ///
    /// 在这个阶段，外部事件被转换为 DomainCommand
    ExternalActions,

    /// 2. 连续场更新（扩散、传导）
    ///
    /// 在这个阶段，温度场、湿度场等进行扩散计算
    FieldUpdate,

    /// 3. 离散状态更新（燃烧 tick、生长 tick）
    ///
    /// 在这个阶段，燃烧消耗燃料、植物生长等
    StateUpdate,

    /// 4. 反应规则判定（产生命令）
    ///
    /// 在这个阶段，评估所有反应规则，产出 DomainCommand
    Reactions,

    /// 5. 统一提交（执行命令、写回状态）
    ///
    /// 在这个阶段，统一执行所有命令，处理冲突，写入 diff
    Commit,

    /// 6. 后处理（清理、打包 diff、渲染）
    ///
    /// 在这个阶段，清理临时数据，打包网络同步数据
    Post,
}

/// 领域系统插件
///
/// 配置所有领域相关的系统执行顺序
pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app
            // 配置系统集顺序
            .configure_sets(
                FixedUpdate,
                (
                    SimulationSet::ExternalActions,
                    SimulationSet::FieldUpdate,
                    SimulationSet::StateUpdate,
                    SimulationSet::Reactions,
                    SimulationSet::Commit,
                    SimulationSet::Post,
                )
                    .chain(),
            )
            // 注册反应规则资源
            .init_resource::<reaction::ReactionRules>()
            // 添加命令队列组件
            .add_systems(Startup, spawn_command_queue)
            // 添加提交系统
            .add_systems(FixedUpdate, command::commit_system.in_set(SimulationSet::Commit))
            // 添加后处理系统（清理变更日志）
            .add_systems(FixedUpdate, cleanup_changes_system.in_set(SimulationSet::Post))
            // 添加温度场可视化调试系统
            .add_systems(Update, thermal_debug_system)
            // 注册热力学插件和测试插件
            .add_plugins((thermal::ThermalPlugin, thermal::ThermalTestPlugin));
    }
}

/// 清理变更日志系统
fn cleanup_changes_system(mut voxel_world: ResMut<crate::voxel::VoxelWorld>) {
    for chunk in voxel_world.chunks.values_mut() {
        chunk.clear_changes();
    }
}

/// 温度场调试系统
///
/// 按 F3 显示活跃方块统计信息
fn thermal_debug_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    voxel_world: Res<crate::voxel::VoxelWorld>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        let mut total_active = 0;
        let mut total_thermal = 0;
        let mut chunks_with_thermal = 0;

        for chunk in voxel_world.chunks.values() {
            total_active += chunk.active_count();
            total_thermal += chunk.active_thermal.len();
            if !chunk.active_thermal.is_empty() {
                chunks_with_thermal += 1;
            }
        }

        info!(
            "=== Thermal Debug ===\nChunks: {}\nChunks with thermal: {}\nActive thermal: {}\nTotal active: {}",
            voxel_world.chunks.len(),
            chunks_with_thermal,
            total_thermal,
            total_active
        );
    }
}

/// 在启动时创建全局命令队列
fn spawn_command_queue(mut commands: Commands) {
    commands.spawn(command::CommandQueue::default());
}
