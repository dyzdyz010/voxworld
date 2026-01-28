use bevy::prelude::*;

use crate::player::PlayerCamera;
use crate::voxel::{ivec3_to_vec3, VoxelKind, VoxelWorld};

#[derive(Debug, Clone, Copy)]
pub struct VoxelHit {
    pub pos: IVec3,
    pub kind: VoxelKind,
    pub distance: f32,
}

#[derive(Resource, Default)]
pub struct HighlightState {
    pub current: Option<VoxelHit>,
}

pub struct RaycastPlugin;

impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HighlightState>()
            .add_systems(Update, (raycast_voxels, draw_highlight_gizmo));
    }
}

/// DDA (Digital Differential Analyzer) voxel raycast algorithm
/// Much more efficient than testing every voxel
fn raycast_voxels(
    world: Res<VoxelWorld>,
    camera_q: Query<&GlobalTransform, With<PlayerCamera>>,
    mut highlight: ResMut<HighlightState>,
) {
    let Ok(camera_transform) = camera_q.single() else {
        highlight.current = None;
        return;
    };

    let origin = camera_transform.translation();
    let dir = camera_transform.forward().as_vec3();
    let max_dist = 8.0;

    highlight.current = dda_raycast(&world, origin, dir, max_dist);
}

/// Fast voxel traversal using DDA algorithm
fn dda_raycast(world: &VoxelWorld, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<VoxelHit> {
    // Current voxel position
    let mut pos = IVec3::new(
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );

    // Direction to step in each axis
    let step = IVec3::new(
        if dir.x >= 0.0 { 1 } else { -1 },
        if dir.y >= 0.0 { 1 } else { -1 },
        if dir.z >= 0.0 { 1 } else { -1 },
    );

    // Distance along ray to cross one voxel in each axis
    let delta = Vec3::new(
        if dir.x.abs() < 1e-10 { f32::MAX } else { (1.0 / dir.x).abs() },
        if dir.y.abs() < 1e-10 { f32::MAX } else { (1.0 / dir.y).abs() },
        if dir.z.abs() < 1e-10 { f32::MAX } else { (1.0 / dir.z).abs() },
    );

    // Distance to next voxel boundary in each axis
    let mut t_max = Vec3::new(
        if dir.x >= 0.0 {
            ((pos.x + 1) as f32 - origin.x) * delta.x
        } else {
            (origin.x - pos.x as f32) * delta.x
        },
        if dir.y >= 0.0 {
            ((pos.y + 1) as f32 - origin.y) * delta.y
        } else {
            (origin.y - pos.y as f32) * delta.y
        },
        if dir.z >= 0.0 {
            ((pos.z + 1) as f32 - origin.z) * delta.z
        } else {
            (origin.z - pos.z as f32) * delta.z
        },
    );

    let mut distance = 0.0;

    while distance < max_dist {
        // Check current voxel
        let kind = world.get_voxel(pos);
        if kind != VoxelKind::Air && kind.is_solid() {
            return Some(VoxelHit {
                pos,
                kind,
                distance,
            });
        }

        // Move to next voxel (step along the axis with smallest t_max)
        if t_max.x < t_max.y && t_max.x < t_max.z {
            distance = t_max.x;
            t_max.x += delta.x;
            pos.x += step.x;
        } else if t_max.y < t_max.z {
            distance = t_max.y;
            t_max.y += delta.y;
            pos.y += step.y;
        } else {
            distance = t_max.z;
            t_max.z += delta.z;
            pos.z += step.z;
        }
    }

    None
}

fn draw_highlight_gizmo(mut gizmos: Gizmos, highlight: Res<HighlightState>) {
    if let Some(hit) = highlight.current {
        let center = ivec3_to_vec3(hit.pos) + Vec3::splat(0.5);
        let transform = Transform::from_translation(center).with_scale(Vec3::splat(1.02));
        gizmos.cube(transform, Color::srgb(1.0, 0.95, 0.2));
    }
}
