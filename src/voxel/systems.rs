//! 体素世界的系统函数

use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures_lite::future;

use crate::voxel::chunk::{ChunkData, ChunkMarker, ChunkPos, VoxelWorld};
use crate::voxel::constants::{CHUNK_HEIGHT, CHUNK_SIZE, RENDER_DISTANCE};
use crate::voxel::loading::{
    ChunkLoadQueue, ChunkReplacementBuffer, CompletedChunk, ComputeMeshTask, PlaceholderEntities,
};
use crate::voxel::materials::ChunkMaterials;
use crate::voxel::mesh::create_placeholder_mesh;
use crate::voxel::mesh_gen::generate_chunk_and_mesh_async;
use crate::voxel::seed::WorldSeed;

// ============================================================================
// 视锥剔除
// ============================================================================

/// 检查区块是否在摄像机视野内
/// 使用简化的视锥剔除算法，基于摄像机朝向和视角范围
///
/// # 参数
/// * `chunk_pos` - 区块位置
/// * `camera_transform` - 摄像机的Transform（包含位置和旋转）
///
/// # 返回值
/// 如果区块在视野内返回 true
fn is_chunk_in_frustum(chunk_pos: &ChunkPos, camera_transform: &Transform) -> bool {
    let origin = chunk_pos.world_origin();

    // 计算区块中心点（世界坐标）
    let chunk_center = Vec3::new(
        origin.x as f32 + CHUNK_SIZE as f32 / 2.0,
        CHUNK_HEIGHT as f32 / 2.0,
        origin.z as f32 + CHUNK_SIZE as f32 / 2.0,
    );

    // 计算从摄像机到区块中心的向量
    let to_chunk = chunk_center - camera_transform.translation;
    let distance = to_chunk.length();

    // 如果距离太近，总是加载（避免摄像机内部的区块被剔除）
    if distance < CHUNK_SIZE as f32 * 2.0 {
        return true;
    }

    // 获取摄像机的前向向量
    let camera_forward = camera_transform.forward();

    // 计算摄像机前向向量与到区块向量的夹角
    let direction_normalized = to_chunk.normalize();
    let dot_product = camera_forward.dot(direction_normalized);

    // 视野角度约110度（FOV），cos(110°/2) ≈ cos(55°) ≈ 0.57
    // 使用更宽的角度（130度）以包含边缘区块，避免误剔除
    // cos(130°/2) = cos(65°) ≈ 0.42
    const FOV_COS_THRESHOLD: f32 = 0.35;

    // 如果夹角在视野范围内，返回 true
    dot_product > FOV_COS_THRESHOLD
}

// ============================================================================
// 区块加载系统
// ============================================================================

/// 更新区块加载系统
/// 根据摄像机位置决定哪些区块需要加载或卸载
/// 按距离排序：距离近的优先加载
/// 优化: 添加视锥剔除，只加载视野内的区块
pub fn update_chunk_loading(
    camera_query: Query<&Transform, With<Camera3d>>,
    world: Res<VoxelWorld>,
    mut queue: ResMut<ChunkLoadQueue>,
    pending_query: Query<&ComputeMeshTask>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_pos = camera_transform.translation;
    let center_chunk = ChunkPos::from_world_pos(camera_pos.x as i32, camera_pos.z as i32);

    // 收集正在处理中的区块
    let pending_chunks: Vec<ChunkPos> = pending_query.iter().map(|t| t.chunk_pos).collect();

    // 收集需要加载的区块
    let mut chunks_to_add = Vec::new();
    for dx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            let chunk_pos = ChunkPos::new(center_chunk.x + dx, center_chunk.z + dz);

            // 优化: 视锥剔除 - 跳过视野外的区块
            if !is_chunk_in_frustum(&chunk_pos, camera_transform) {
                continue;
            }

            if !world.loaded_chunks.contains_key(&chunk_pos)
                && !world.chunks.contains_key(&chunk_pos)
                && !queue.to_load.contains(&chunk_pos)
                && !pending_chunks.contains(&chunk_pos)
            {
                chunks_to_add.push(chunk_pos);
            }
        }
    }

    // 按距离排序（距离近的优先）
    chunks_to_add.sort_by(|a, b| {
        let dist_a = (a.x - center_chunk.x).pow(2) + (a.z - center_chunk.z).pow(2);
        let dist_b = (b.x - center_chunk.x).pow(2) + (b.z - center_chunk.z).pow(2);
        dist_a.cmp(&dist_b)
    });

    // 如果有新区块加入，将它们添加到待批量创建占位符列表
    if !chunks_to_add.is_empty() {
        queue
            .pending_placeholders
            .extend(chunks_to_add.iter().copied());
    }

    queue.to_load.extend(chunks_to_add);

    // 重新排序整个队列（玩家移动后需要重新排序）
    queue.to_load.sort_by(|a, b| {
        let dist_a = (a.x - center_chunk.x).pow(2) + (a.z - center_chunk.z).pow(2);
        let dist_b = (b.x - center_chunk.x).pow(2) + (b.z - center_chunk.z).pow(2);
        dist_a.cmp(&dist_b)
    });

    // 查找需要卸载的区块（超出渲染距离+1）
    for &chunk_pos in world.loaded_chunks.keys() {
        let dx = (chunk_pos.x - center_chunk.x).abs();
        let dz = (chunk_pos.z - center_chunk.z).abs();
        if dx > RENDER_DISTANCE + 1 || dz > RENDER_DISTANCE + 1 {
            if !queue.to_unload.contains(&chunk_pos) {
                queue.to_unload.push(chunk_pos);
            }
        }
    }
}

/// 批量创建占位符实体（一次性显示整个加载范围的蓝色网格线）
pub fn spawn_batch_placeholders(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<ChunkMaterials>,
    mut queue: ResMut<ChunkLoadQueue>,
    mut placeholders: ResMut<PlaceholderEntities>,
) {
    if queue.pending_placeholders.is_empty() {
        return;
    }

    // 批量创建所有待创建的占位符
    let chunks_to_create: Vec<_> = queue.pending_placeholders.drain(..).collect();

    // 创建共享的占位符网格（所有区块使用同一个网格）
    let placeholder_mesh = create_placeholder_mesh();
    let placeholder_handle = meshes.add(placeholder_mesh);

    for chunk_pos in chunks_to_create {
        let origin = chunk_pos.world_origin();

        let placeholder_entity = commands
            .spawn((
                Mesh3d(placeholder_handle.clone()),
                MeshMaterial3d(materials.transparent.clone()),
                Transform::from_translation(Vec3::new(origin.x as f32, 0.0, origin.z as f32)),
                ChunkMarker { pos: chunk_pos },
            ))
            .id();

        // 保存占位符实体，供后续任务使用
        placeholders.map.insert(chunk_pos, placeholder_entity);
    }
}

/// 派发异步网格生成任务（使用已创建的占位符）
pub fn spawn_mesh_tasks(
    mut commands: Commands,
    mut queue: ResMut<ChunkLoadQueue>,
    placeholders: ResMut<PlaceholderEntities>,
    seed: Res<WorldSeed>,
) {
    // 限制并发任务数
    let available_slots = queue.max_concurrent_tasks.saturating_sub(queue.active_tasks);
    if available_slots == 0 {
        return;
    }

    let count = queue.to_load.len().min(available_slots);
    let chunks_to_process: Vec<_> = queue.to_load.drain(..count).collect();

    let task_pool = AsyncComputeTaskPool::get();
    let seed_value = seed.seed;

    for chunk_pos in chunks_to_process {
        // 从占位符映射中获取已创建的占位符实体
        let placeholder_entity = match placeholders.map.get(&chunk_pos) {
            Some(&entity) => entity,
            None => {
                // 如果没有占位符（不应该发生），跳过这个区块
                continue;
            }
        };

        // 派发异步任务（包含区块生成和网格构建）
        let task =
            task_pool.spawn(async move { generate_chunk_and_mesh_async(chunk_pos, seed_value) });

        // 创建任务跟踪实体
        commands.spawn(ComputeMeshTask {
            task,
            chunk_pos,
            placeholder_entity,
        });

        queue.active_tasks += 1;
    }
}

/// 处理完成的网格生成任务（收集到缓冲区，等待批量替换）
pub fn handle_completed_mesh_tasks(
    mut commands: Commands,
    mut queue: ResMut<ChunkLoadQueue>,
    mut buffer: ResMut<ChunkReplacementBuffer>,
    mut pending_query: Query<(Entity, &mut ComputeMeshTask)>,
) {
    for (entity, mut task) in pending_query.iter_mut() {
        // 非阻塞地检查任务是否完成
        if let Some((voxels, mesh)) = future::block_on(future::poll_once(&mut task.task)) {
            let chunk_pos = task.chunk_pos;
            let placeholder_entity = task.placeholder_entity;

            // 移除任务跟踪实体
            commands.entity(entity).despawn();
            queue.active_tasks = queue.active_tasks.saturating_sub(1);

            // 收集到缓冲区，等待批量替换
            buffer.completed.push(CompletedChunk {
                chunk_pos,
                voxels,
                mesh,
                placeholder_entity,
            });
        }
    }
}

/// 批量替换占位符为真实区块
/// 按时间间隔或达到批量大小时触发，减少闪烁
pub fn apply_chunk_replacements(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<ChunkMaterials>,
    mut world: ResMut<VoxelWorld>,
    mut buffer: ResMut<ChunkReplacementBuffer>,
    mut placeholders: ResMut<PlaceholderEntities>,
) {
    if buffer.completed.is_empty() {
        return;
    }

    // 更新定时器
    buffer.timer += time.delta_secs();

    // 检查是否应该批量替换
    let should_replace =
        buffer.completed.len() >= buffer.min_batch_size || buffer.timer >= buffer.interval;

    if !should_replace {
        return;
    }

    // 重置定时器
    buffer.timer = 0.0;

    // 批量替换所有完成的区块
    for completed in buffer.completed.drain(..) {
        // 存储区块数据
        world.chunks.insert(
            completed.chunk_pos,
            ChunkData {
                voxels: completed.voxels,
                is_dirty: false,
            },
        );

        // 移除蓝色占位符实体
        commands.entity(completed.placeholder_entity).despawn();
        placeholders.map.remove(&completed.chunk_pos);

        // 创建真实区块渲染实体（替换占位符）
        let mesh_handle = meshes.add(completed.mesh);
        let origin = completed.chunk_pos.world_origin();

        let chunk_entity = commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(materials.opaque.clone()),
                Transform::from_translation(Vec3::new(origin.x as f32, 0.0, origin.z as f32)),
                ChunkMarker {
                    pos: completed.chunk_pos,
                },
            ))
            .id();

        world
            .loaded_chunks
            .insert(completed.chunk_pos, chunk_entity);
    }
}

/// 处理区块卸载（包括占位符和任务取消）
pub fn process_chunk_unload(
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    mut queue: ResMut<ChunkLoadQueue>,
    mut buffer: ResMut<ChunkReplacementBuffer>,
    mut placeholders: ResMut<PlaceholderEntities>,
    pending_query: Query<(Entity, &ComputeMeshTask)>,
) {
    // 先收集要卸载的区块和要取消的任务数
    let chunks_to_unload: Vec<_> = queue.to_unload.drain(..).collect();
    let mut tasks_to_cancel = 0;

    for chunk_pos in chunks_to_unload {
        // 卸载已渲染的区块
        if let Some(entity) = world.loaded_chunks.remove(&chunk_pos) {
            commands.entity(entity).despawn();
        }

        // 取消该区块的待处理任务并删除占位符
        for (entity, task) in pending_query.iter() {
            if task.chunk_pos == chunk_pos {
                // 删除任务跟踪实体
                commands.entity(entity).despawn();
                // 删除蓝色占位符实体
                commands.entity(task.placeholder_entity).despawn();
                tasks_to_cancel += 1;
            }
        }

        // 删除独立的占位符（如果存在）
        if let Some(entity) = placeholders.map.remove(&chunk_pos) {
            commands.entity(entity).despawn();
        }

        // 从替换缓冲区中移除该区块（如果存在）
        buffer.completed.retain(|c| c.chunk_pos != chunk_pos);

        // 从待创建占位符列表中移除（如果存在）
        queue.pending_placeholders.retain(|&pos| pos != chunk_pos);

        world.chunks.remove(&chunk_pos);
    }

    // 更新活跃任务计数
    queue.active_tasks = queue.active_tasks.saturating_sub(tasks_to_cancel);
}
