//! 体素世界常量定义

/// 区块在X和Z方向上的大小（单位：体素）
pub const CHUNK_SIZE: i32 = 16;

/// 区块在Y方向上的高度（单位：体素）
pub const CHUNK_HEIGHT: i32 = 64;

/// 渲染距离（单位：区块数）- 控制玩家周围加载多少区块
pub const RENDER_DISTANCE: i32 = 16;
