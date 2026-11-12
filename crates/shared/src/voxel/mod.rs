//! Voxel storage: chunk coordinates, palette-compressed chunks and raycasting.

pub mod chunk;
pub mod pos;
pub mod raycast;

pub use chunk::{BlockId, Chunk};
pub use pos::{
    BlockPos, CHUNK_MASK, CHUNK_SHIFT, CHUNK_SIZE, CHUNK_VOLUME, ChunkPos, Dir, SURFACE_Y,
    WORLD_SIZE_XZ, WORLD_SIZE_Y, coords_of, index_of,
};

/// Errors produced by voxel serialisation.
#[derive(Debug, thiserror::Error)]
pub enum VoxelError {
    /// Chunk payload could not be serialised.
    #[error("failed to serialise chunk: {0}")]
    Serialise(#[from] postcard::Error),
    /// Chunk payload could not be compressed or decompressed.
    #[error("failed to compress chunk: {0}")]
    Compress(#[from] std::io::Error),
    /// Chunk payload was structurally invalid.
    #[error("malformed chunk payload")]
    MalformedChunk,
}
