//! 网格构建系统 - 顶点处理、去重和网格构建器

use bevy::mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::voxel::constants::CHUNK_SIZE;

// ============================================================================
// 顶点去重
// ============================================================================

/// 顶点唯一标识键 - 用于HashMap去重
/// 考虑位置、法线和颜色
#[derive(Clone, Copy)]
struct VertexKey {
    /// 位置 - 使用定点数避免浮点精度问题（乘以1000转整数）
    pos: [i32; 3],
    /// 法线方向索引 - 6个方向：0=+X, 1=-X, 2=+Y, 3=-Y, 4=+Z, 5=-Z
    normal_index: u8,
    /// 颜色 - 压缩为32位RGBA
    color_packed: u32,
}

impl VertexKey {
    /// 从浮点数据创建顶点键
    fn new(pos: [f32; 3], normal: [f32; 3], color: [f32; 4]) -> Self {
        // 位置转定点数
        let pos_fixed = [
            (pos[0] * 1000.0) as i32,
            (pos[1] * 1000.0) as i32,
            (pos[2] * 1000.0) as i32,
        ];

        // 法线编码为索引
        let normal_index = match (normal[0] as i32, normal[1] as i32, normal[2] as i32) {
            (1, 0, 0) => 0,
            (-1, 0, 0) => 1,
            (0, 1, 0) => 2,
            (0, -1, 0) => 3,
            (0, 0, 1) => 4,
            (0, 0, -1) => 5,
            _ => 0,
        };

        // 颜色压缩为RGBA8888
        let color_packed = ((color[0] * 255.0) as u32) << 24
            | ((color[1] * 255.0) as u32) << 16
            | ((color[2] * 255.0) as u32) << 8
            | ((color[3] * 255.0) as u32);

        Self {
            pos: pos_fixed,
            normal_index,
            color_packed,
        }
    }
}

impl PartialEq for VertexKey {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
            && self.normal_index == other.normal_index
            && self.color_packed == other.color_packed
    }
}

impl Eq for VertexKey {}

impl Hash for VertexKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].hash(state);
        self.pos[1].hash(state);
        self.pos[2].hash(state);
        self.normal_index.hash(state);
        self.color_packed.hash(state);
    }
}

// ============================================================================
// 线程本地缓冲区
// ============================================================================

/// 网格构建缓冲区 - 避免每次分配新Vec
pub struct MeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
    /// 顶点去重HashMap
    vertex_map: HashMap<VertexKey, u32>,
}

impl MeshBuffers {
    fn new() -> Self {
        // 预分配合理的初始容量
        Self {
            positions: Vec::with_capacity(20000),
            normals: Vec::with_capacity(20000),
            colors: Vec::with_capacity(20000),
            indices: Vec::with_capacity(30000),
            vertex_map: HashMap::with_capacity(20000),
        }
    }

    /// 清空缓冲区但保留容量
    fn clear(&mut self) {
        self.positions.clear();
        self.normals.clear();
        self.colors.clear();
        self.indices.clear();
        self.vertex_map.clear();
    }
}

thread_local! {
    /// 每个线程独立的网格构建缓冲区
    pub static MESH_BUFFERS: RefCell<MeshBuffers> = RefCell::new(MeshBuffers::new());
}

// ============================================================================
// 区块网格构建器
// ============================================================================

/// 优化后的区块网格构建器 - 使用线程本地缓冲区和顶点去重
pub struct ChunkMeshBuilder<'a> {
    buffers: &'a mut MeshBuffers,
}

impl<'a> ChunkMeshBuilder<'a> {
    /// 使用线程本地缓冲区创建构建器
    pub fn with_buffers(buffers: &'a mut MeshBuffers) -> Self {
        buffers.clear();
        Self { buffers }
    }

    /// 添加面片并进行顶点去重
    pub fn add_face_deduplicated(
        &mut self,
        vertices: [[f32; 3]; 4],
        normal: [f32; 3],
        color: [f32; 4],
    ) {
        let mut face_indices = [0u32; 4];

        for (i, &pos) in vertices.iter().enumerate() {
            let key = VertexKey::new(pos, normal, color);

            // 查找或插入顶点
            let index = match self.buffers.vertex_map.get(&key) {
                Some(&existing_index) => existing_index,
                None => {
                    let new_index = self.buffers.positions.len() as u32;
                    self.buffers.positions.push(pos);
                    self.buffers.normals.push(normal);
                    self.buffers.colors.push(color);
                    self.buffers.vertex_map.insert(key, new_index);
                    new_index
                }
            };

            face_indices[i] = index;
        }

        // 添加两个三角形的索引
        self.buffers.indices.extend_from_slice(&[
            face_indices[0],
            face_indices[2],
            face_indices[1],
            face_indices[0],
            face_indices[3],
            face_indices[2],
        ]);
    }

    /// 构建最终网格（从缓冲区克隆数据）
    pub fn build(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.buffers.positions.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.buffers.normals.clone());
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::Float32x4(self.buffers.colors.clone()),
        );
        mesh.insert_indices(Indices::U32(self.buffers.indices.clone()));
        mesh
    }

    /// 构建空网格（用于空气区块优化）
    pub fn build_empty_mesh() -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::Float32x4(Vec::new()),
        );
        mesh.insert_indices(Indices::U32(Vec::new()));
        mesh
    }
}

// ============================================================================
// 占位符网格
// ============================================================================

/// 创建蓝色线框占位符网格
/// 只绘制区块的立方体边框（12条边）
pub fn create_placeholder_mesh() -> Mesh {
    let size = CHUNK_SIZE as f32;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    // 半透明蓝色
    let blue = [0.3, 0.6, 1.0, 0.3];
    let normal = [0.0, 1.0, 0.0]; // 线框法线随意

    // 8个角的顶点
    let corners = [
        [0.0, 0.0, 0.0],    // 0: 左下前
        [size, 0.0, 0.0],   // 1: 右下前
        [size, 0.0, size],  // 2: 右下后
        [0.0, 0.0, size],   // 3: 左下后
        [0.0, size, 0.0],   // 4: 左上前
        [size, size, 0.0],  // 5: 右上前
        [size, size, size], // 6: 右上后
        [0.0, size, size],  // 7: 左上后
    ];

    // 12条边（每条边连接两个顶点）
    let edges = [
        // 底部4条边
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        // 顶部4条边
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        // 垂直4条边
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    // 为每条边添加两个顶点
    for (start, end) in edges {
        let idx = positions.len() as u32;
        positions.push(corners[start]);
        positions.push(corners[end]);
        normals.push(normal);
        normals.push(normal);
        colors.push(blue);
        colors.push(blue);
        indices.extend_from_slice(&[idx, idx + 1]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

// ============================================================================
// 面片顶点计算
// ============================================================================

/// 根据方向获取面片的4个顶点坐标
/// 顶点顺序确保逆时针环绕（用于正确的面剔除）
pub fn get_face_vertices(x: f32, y: f32, z: f32, dir: IVec3) -> [[f32; 3]; 4] {
    match (dir.x, dir.y, dir.z) {
        // 右面 (+X)
        (1, 0, 0) => [
            [x + 1.0, y, z],
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z],
        ],
        // 左面 (-X)
        (-1, 0, 0) => [
            [x, y, z + 1.0],
            [x, y, z],
            [x, y + 1.0, z],
            [x, y + 1.0, z + 1.0],
        ],
        // 上面 (+Y)
        (0, 1, 0) => [
            [x, y + 1.0, z],
            [x + 1.0, y + 1.0, z],
            [x + 1.0, y + 1.0, z + 1.0],
            [x, y + 1.0, z + 1.0],
        ],
        // 下面 (-Y)
        (0, -1, 0) => [
            [x, y, z + 1.0],
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y, z],
            [x, y, z],
        ],
        // 前面 (+Z)
        (0, 0, 1) => [
            [x + 1.0, y, z + 1.0],
            [x, y, z + 1.0],
            [x, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
        ],
        // 后面 (-Z)
        (0, 0, -1) => [
            [x, y, z],
            [x + 1.0, y, z],
            [x + 1.0, y + 1.0, z],
            [x, y + 1.0, z],
        ],
        _ => [[0.0; 3]; 4],
    }
}
