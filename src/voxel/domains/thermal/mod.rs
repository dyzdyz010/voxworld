//! 热力学领域模块
//!
//! 提供温度场模拟功能：
//! - 热扩散（相邻方块间的热传导）
//! - 环境热交换（边界与环境的热交换）
//! - 热源（燃烧等持续产热）
//!
//! ## 物理模型
//!
//! 热传导遵循傅里叶定律：Q = -k * A * (dT/dx)
//!
//! 简化后：heat_flow = k_avg * ΔT * dt
//!
//! 其中：
//! - k_avg: 两个相邻方块导热系数的平均值
//! - ΔT: 温度差
//! - dt: 时间步长
//!
//! ## 测试
//!
//! 使用以下快捷键测试热力学系统：
//! - F5: 在世界原点创建热源
//! - F6: 显示温度信息
//! - F7: 清除温度状态

pub mod api;
pub mod state;
pub mod systems;
pub mod test;

pub use api::ThermalApi;
pub use state::ThermalState;
pub use systems::ThermalPlugin;
pub use test::ThermalTestPlugin;
