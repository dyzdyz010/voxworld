//! 体素世界常量定义

/// 区块大小（单位：体素）- 所有三个维度统一为16×16×16立方体
pub const CHUNK_SIZE: i32 = 16;

/// 水平渲染距离（单位：区块数）- 控制玩家周围X-Z平面加载多少区块
pub const RENDER_DISTANCE: i32 = 8;

/// 垂直渲染距离（单位：区块数）- 控制玩家周围Y轴加载多少层区块
/// 建议设置为RENDER_DISTANCE的1/4到1/2，以减少内存占用
pub const VERTICAL_RENDER_DISTANCE: i32 = 4;
