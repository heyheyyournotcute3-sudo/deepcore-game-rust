//! Chunk and block coordinate types.
//!
//! Chunks are 32×32×32. World coordinates are block coordinates: `x` and `z` span
//! `0..1024`, `y` spans `0..512` with `y = 0` at the bedrock floor and `y = 511` at the
//! top of the sky. "Depth" is the number of blocks below the surface plane, so a larger
//! `y` means shallower.

use serde::{Deserialize, Serialize};

/// Edge length of a chunk on every axis.
pub const CHUNK_SIZE: usize = 32;
/// Number of blocks in a chunk (`32³ = 32768`).
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
/// Log2 of [`CHUNK_SIZE`], used for fast shifts.
pub const CHUNK_SHIFT: u32 = 5;
/// Mask for the local coordinate inside a chunk.
pub const CHUNK_MASK: i32 = (CHUNK_SIZE as i32) - 1;

/// Width/depth of the world in blocks.
pub const WORLD_SIZE_XZ: i32 = 1024;
/// Height of the world in blocks.
pub const WORLD_SIZE_Y: i32 = 512;
/// Y coordinate of the surface plane that the camp sits on.
pub const SURFACE_Y: i32 = WORLD_SIZE_Y - 32;

/// A chunk coordinate in chunk space.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct ChunkPos {
    /// Chunk x index.
    pub x: i32,
    /// Chunk y index (vertical).
    pub y: i32,
    /// Chunk z index.
    pub z: i32,
}

impl ChunkPos {
    /// Creates a chunk position.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Chunk position containing a block position.
    #[must_use]
    pub const fn from_block(pos: BlockPos) -> Self {
        Self { x: pos.x >> CHUNK_SHIFT, y: pos.y >> CHUNK_SHIFT, z: pos.z >> CHUNK_SHIFT }
    }

    /// Number of chunks along x and z.
    #[must_use]
    pub const fn chunks_per_axis_xz() -> i32 {
        WORLD_SIZE_XZ / CHUNK_SIZE as i32
    }

    /// Number of chunks along y.
    #[must_use]
    pub const fn chunks_per_axis_y() -> i32 {
        WORLD_SIZE_Y / CHUNK_SIZE as i32
    }

    /// True when this chunk lies inside the world bounds.
    #[must_use]
    pub const fn in_bounds(self) -> bool {
        self.x >= 0
            && self.z >= 0
            && self.y >= 0
            && self.x < Self::chunks_per_axis_xz()
            && self.z < Self::chunks_per_axis_xz()
            && self.y < Self::chunks_per_axis_y()
    }

    /// Squared distance to another chunk, in chunk units.
    #[must_use]
    pub fn distance_squared(self, other: Self) -> i32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    /// Iterates the six neighbouring chunk positions.
    #[must_use]
    pub fn neighbours(self) -> [Self; 6] {
        [
            Self::new(self.x + 1, self.y, self.z),
            Self::new(self.x - 1, self.y, self.z),
            Self::new(self.x, self.y + 1, self.z),
            Self::new(self.x, self.y - 1, self.z),
            Self::new(self.x, self.y, self.z + 1),
            Self::new(self.x, self.y, self.z - 1),
        ]
    }
}

/// A block coordinate in world space.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct BlockPos {
    /// World x.
    pub x: i32,
    /// World y (0 = bedrock floor).
    pub y: i32,
    /// World z.
    pub z: i32,
}

impl BlockPos {
    /// Creates a block position.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Truncates a floating point world position to the block containing it.
    #[must_use]
    pub fn from_world(v: bevy_math::Vec3) -> Self {
        Self { x: v.x.floor() as i32, y: v.y.floor() as i32, z: v.z.floor() as i32 }
    }

    /// Centre of the block in world space.
    #[must_use]
    pub fn center(self) -> bevy_math::Vec3 {
        bevy_math::Vec3::new(self.x as f32 + 0.5, self.y as f32 + 0.5, self.z as f32 + 0.5)
    }

    /// True when the block lies inside the world bounds.
    #[must_use]
    pub const fn in_bounds(self) -> bool {
        self.x >= 0
            && self.y >= 0
            && self.z >= 0
            && self.x < WORLD_SIZE_XZ
            && self.y < WORLD_SIZE_Y
            && self.z < WORLD_SIZE_XZ
    }

    /// Offset by an integer vector.
    #[must_use]
    pub const fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }

    /// Offset by a face normal.
    #[must_use]
    pub const fn offset_dir(self, dir: Dir) -> Self {
        self.offset(dir.dx(), dir.dy(), dir.dz())
    }

    /// Distance to another block.
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Local coordinates inside the containing chunk, each in `0..32`.
    #[must_use]
    pub const fn local(self) -> (usize, usize, usize) {
        (
            (self.x & CHUNK_MASK) as usize,
            (self.y & CHUNK_MASK) as usize,
            (self.z & CHUNK_MASK) as usize,
        )
    }

    /// Flat index inside the containing chunk, in `0..CHUNK_VOLUME`.
    #[must_use]
    pub const fn chunk_index(self) -> usize {
        let (lx, ly, lz) = self.local();
        index_of(lx, ly, lz)
    }
}

/// Flat index for local chunk coordinates. Layout is x-major, then z, then y.
#[must_use]
pub const fn index_of(x: usize, y: usize, z: usize) -> usize {
    (y * CHUNK_SIZE + z) * CHUNK_SIZE + x
}

/// Inverse of [`index_of`].
#[must_use]
pub const fn coords_of(index: usize) -> (usize, usize, usize) {
    let x = index % CHUNK_SIZE;
    let y = index / (CHUNK_SIZE * CHUNK_SIZE);
    let z = (index / CHUNK_SIZE) % CHUNK_SIZE;
    (x, y, z)
}

/// The six axis-aligned face directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dir {
    /// +X
    PosX,
    /// -X
    NegX,
    /// +Y (up)
    PosY,
    /// -Y (down)
    NegY,
    /// +Z
    PosZ,
    /// -Z
    NegZ,
}

impl Dir {
    /// All six directions.
    pub const ALL: [Self; 6] =
        [Self::PosX, Self::NegX, Self::PosY, Self::NegY, Self::PosZ, Self::NegZ];

    /// X component of the face normal.
    #[must_use]
    pub const fn dx(self) -> i32 {
        match self {
            Self::PosX => 1,
            Self::NegX => -1,
            Self::PosY | Self::NegY | Self::PosZ | Self::NegZ => 0,
        }
    }

    /// Y component of the face normal.
    #[must_use]
    pub const fn dy(self) -> i32 {
        match self {
            Self::PosY => 1,
            Self::NegY => -1,
            Self::PosX | Self::NegX | Self::PosZ | Self::NegZ => 0,
        }
    }

    /// Z component of the face normal.
    #[must_use]
    pub const fn dz(self) -> i32 {
        match self {
            Self::PosZ => 1,
            Self::NegZ => -1,
            Self::PosX | Self::NegX | Self::PosY | Self::NegY => 0,
        }
    }

    /// The opposite face.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::PosX => Self::NegX,
            Self::NegX => Self::PosX,
            Self::PosY => Self::NegY,
            Self::NegY => Self::PosY,
            Self::PosZ => Self::NegZ,
            Self::NegZ => Self::PosZ,
        }
    }

    /// Face normal as a float vector.
    #[must_use]
    pub fn normal(self) -> bevy_math::Vec3 {
        bevy_math::Vec3::new(self.dx() as f32, self.dy() as f32, self.dz() as f32)
    }

    /// Direction whose axis matches `axis` (0 = x, 1 = y, 2 = z) and sign matches `positive`.
    #[must_use]
    pub const fn from_axis(axis: usize, positive: bool) -> Self {
        match (axis, positive) {
            (0, true) => Self::PosX,
            (0, false) => Self::NegX,
            (1, true) => Self::PosY,
            (1, false) => Self::NegY,
            (2, true) => Self::PosZ,
            _ => Self::NegZ,
        }
    }
}

/// Offset then offset back must return to the start, for every direction.
#[cfg(test)]
fn offset_round_trip(dir: Dir) -> bool {
    let p = BlockPos::new(10, 10, 10);
    p.offset_dir(dir).offset_dir(dir.opposite()) == p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_index_round_trips() {
        for i in 0..CHUNK_VOLUME {
            let (x, y, z) = coords_of(i);
            assert!(x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE);
            assert_eq!(index_of(x, y, z), i, "index round trip failed for {i}");
        }
    }

    #[test]
    fn block_to_chunk_and_local_are_consistent() {
        let pos = BlockPos::new(37, 70, -3);
        let chunk = ChunkPos::from_block(pos);
        assert_eq!(chunk, ChunkPos::new(1, 2, -1));
        // Local coordinates use a mask, so negative values wrap correctly.
        assert_eq!(pos.local(), (5, 6, 29));
        let idx = pos.chunk_index();
        assert!(idx < CHUNK_VOLUME);
        assert_eq!(coords_of(idx), pos.local());
    }

    #[test]
    fn boundaries_of_chunks() {
        assert_eq!(ChunkPos::from_block(BlockPos::new(31, 31, 31)), ChunkPos::new(0, 0, 0));
        assert_eq!(ChunkPos::from_block(BlockPos::new(32, 32, 32)), ChunkPos::new(1, 1, 1));
        assert_eq!(BlockPos::new(63, 0, 0).local().0, 31);
        assert_eq!(BlockPos::new(64, 0, 0).local().0, 0);
    }

    #[test]
    fn world_bounds() {
        assert!(BlockPos::new(0, 0, 0).in_bounds());
        assert!(BlockPos::new(WORLD_SIZE_XZ - 1, WORLD_SIZE_Y - 1, WORLD_SIZE_XZ - 1).in_bounds());
        assert!(!BlockPos::new(-1, 0, 0).in_bounds());
        assert!(!BlockPos::new(0, WORLD_SIZE_Y, 0).in_bounds());
        assert!(ChunkPos::new(0, 0, 0).in_bounds());
        assert!(!ChunkPos::new(ChunkPos::chunks_per_axis_xz(), 0, 0).in_bounds());
    }

    #[test]
    fn direction_math() {
        for d in Dir::ALL {
            assert_eq!(d.opposite().opposite(), d);
            assert!(offset_round_trip(d));
        }
        assert_eq!(Dir::PosY.opposite(), Dir::NegY);
        assert_eq!(Dir::from_axis(2, false), Dir::NegZ);
    }
}
