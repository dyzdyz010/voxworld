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

fn raycast_voxels(
    world: Res<VoxelWorld>,
    camera_q: Query<&GlobalTransform, With<PlayerCamera>>,
    mut highlight: ResMut<HighlightState>,
) {
    let Ok(camera_transform) = camera_q.single() else {
        return;
    };
    let origin = camera_transform.translation();
    let dir = camera_transform.forward().as_vec3();
    let max_dist = 10.0;

    let mut best_hit: Option<VoxelHit> = None;
    for (pos, kind) in world.map.iter() {
        let center = ivec3_to_vec3(*pos);
        let min = center - Vec3::splat(0.5);
        let max = center + Vec3::splat(0.5);
        if let Some(t) = ray_aabb(origin, dir, min, max) {
            if t >= 0.0 && t <= max_dist {
                let replace = match best_hit {
                    Some(hit) => t < hit.distance,
                    None => true,
                };
                if replace {
                    best_hit = Some(VoxelHit {
                        pos: *pos,
                        kind: *kind,
                        distance: t,
                    });
                }
            }
        }
    }

    highlight.current = best_hit;
}

fn draw_highlight_gizmo(mut gizmos: Gizmos, highlight: Res<HighlightState>) {
    if let Some(hit) = highlight.current {
        let center = ivec3_to_vec3(hit.pos);
        let transform = Transform::from_translation(center).with_scale(Vec3::splat(1.02));
        gizmos.cube(transform, Color::srgb(1.0, 0.95, 0.2));
    }
}

fn ray_aabb(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let inv_dir = Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
    let t1 = (min - origin) * inv_dir;
    let t2 = (max - origin) * inv_dir;

    let t_min = Vec3::new(t1.x.min(t2.x), t1.y.min(t2.y), t1.z.min(t2.z));
    let t_max = Vec3::new(t1.x.max(t2.x), t1.y.max(t2.y), t1.z.max(t2.z));

    let t_near = t_min.x.max(t_min.y).max(t_min.z);
    let t_far = t_max.x.min(t_max.y).min(t_max.z);

    if t_far >= t_near {
        Some(t_near)
    } else {
        None
    }
}
