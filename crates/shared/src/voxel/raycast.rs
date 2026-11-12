//! Amanatides & Woo voxel traversal.
//!
//! Used for block targeting (what is under the crosshair), line-of-sight validation on the
//! server, and the Ore Scanner. The traversal is a pure function over a block query closure so
//! it works against the client's mesh cache and the server's chunk store alike.

use bevy_math::Vec3;

use crate::voxel::{BlockId, BlockPos, Dir};

/// Result of a ray cast.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    /// The solid block that was hit.
    pub block: BlockPos,
    /// The face of that block the ray entered through.
    pub face: Dir,
    /// Distance along the ray to the hit, in blocks.
    pub distance: f32,
}

/// Casts a ray through the voxel grid, returning the first block for which `is_solid` is true.
///
/// `origin` and `direction` are in world space; `direction` must be normalised. Returns `None`
/// if nothing solid is hit within `max_distance`.
pub fn raycast<F>(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    mut is_solid: F,
) -> Option<RayHit>
where
    F: FnMut(BlockPos) -> bool,
{
    if max_distance <= 0.0 || direction.length_squared() < 1e-6 {
        return None;
    }
    let dir = direction.normalize();

    let mut cell = BlockPos::from_world(origin);
    let step = (
        if dir.x > 0.0 {
            1
        } else if dir.x < 0.0 {
            -1
        } else {
            0
        },
        if dir.y > 0.0 {
            1
        } else if dir.y < 0.0 {
            -1
        } else {
            0
        },
        if dir.z > 0.0 {
            1
        } else if dir.z < 0.0 {
            -1
        } else {
            0
        },
    );

    let mut t_max = Vec3::ZERO;
    let mut t_delta = Vec3::splat(f32::INFINITY);
    let cell_axes = [cell.x, cell.y, cell.z];
    let origin_axes = [origin.x, origin.y, origin.z];
    for axis in 0..3 {
        let d = dir[axis];
        if d.abs() > 1e-8 {
            t_delta[axis] = (1.0 / d).abs();
            let boundary =
                if d > 0.0 { (cell_axes[axis] + 1) as f32 } else { cell_axes[axis] as f32 };
            t_max[axis] = ((boundary - origin_axes[axis]) / d).max(0.0);
        }
    }

    // Starting inside a solid block counts as an immediate hit on the face the ray came from.
    if is_solid(cell) {
        return Some(RayHit { block: cell, face: dominant_entry_face(dir), distance: 0.0 });
    }

    let mut travelled = 0.0_f32;
    while travelled <= max_distance {
        // Advance along the axis with the smallest t_max.
        let axis = if t_max.x < t_max.y && t_max.x < t_max.z {
            0
        } else if t_max.y < t_max.z {
            1
        } else {
            2
        };
        travelled = t_max[axis];
        if travelled > max_distance {
            break;
        }
        t_max[axis] += t_delta[axis];
        match axis {
            0 => cell.x += step.0,
            1 => cell.y += step.1,
            _ => cell.z += step.2,
        }
        if is_solid(cell) {
            let face = match axis {
                0 => Dir::from_axis(0, step.0 < 0),
                1 => Dir::from_axis(1, step.1 < 0),
                _ => Dir::from_axis(2, step.2 < 0),
            };
            return Some(RayHit { block: cell, face, distance: travelled });
        }
    }
    None
}

/// Best guess at the face a ray entered through when it starts inside a block.
fn dominant_entry_face(dir: Vec3) -> Dir {
    let ax = dir.x.abs();
    let ay = dir.y.abs();
    let az = dir.z.abs();
    if ax >= ay && ax >= az {
        Dir::from_axis(0, dir.x > 0.0)
    } else if ay >= az {
        Dir::from_axis(1, dir.y > 0.0)
    } else {
        Dir::from_axis(2, dir.z > 0.0)
    }
}

/// Casts a ray through a chunk-store-like lookup.
pub fn raycast_world<F>(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    mut block_at: F,
) -> Option<RayHit>
where
    F: FnMut(BlockPos) -> BlockId,
{
    raycast(origin, direction, max_distance, |p| !block_at(p).is_air())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hits_block_straight_ahead() {
        let hit = raycast(Vec3::new(0.5, 0.5, 0.5), Vec3::X, 10.0, |p| p.x == 5).unwrap();
        assert_eq!(hit.block, BlockPos::new(5, 0, 0));
        assert_eq!(hit.face, Dir::NegX, "entered through the -X face");
        assert!((hit.distance - 4.5).abs() < 1e-3);
    }

    #[test]
    fn hits_from_above() {
        let hit = raycast(Vec3::new(0.5, 10.5, 0.5), Vec3::NEG_Y, 20.0, |p| p.y == 3).unwrap();
        assert_eq!(hit.block, BlockPos::new(0, 3, 0));
        assert_eq!(hit.face, Dir::PosY);
    }

    #[test]
    fn misses_when_out_of_range() {
        let hit = raycast(Vec3::new(0.5, 0.5, 0.5), Vec3::X, 3.0, |p| p.x == 50);
        assert!(hit.is_none());
    }

    #[test]
    fn no_hit_in_empty_world() {
        assert!(raycast(Vec3::ZERO, Vec3::Y, 32.0, |_| false).is_none());
    }

    #[test]
    fn starting_inside_solid_hits_immediately() {
        let hit = raycast(Vec3::new(0.5, 0.5, 0.5), Vec3::Y, 4.0, |_| true).unwrap();
        assert_eq!(hit.distance, 0.0);
    }

    #[test]
    fn diagonal_ray_hits_something_reachable() {
        let dir = Vec3::new(1.0, -1.0, 0.0).normalize();
        let hit = raycast(Vec3::new(0.5, 5.5, 0.5), dir, 20.0, |p| p.y == 0).unwrap();
        assert_eq!(hit.block.y, 0);
    }
}
