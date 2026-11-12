//! Client-side block access: a local mirror of the world built from the server's chunks (or from
//! the generator in local preview), used for meshing, collision and targeting.

use std::collections::HashMap;

use bevy::prelude::*;

use deepcore_shared::voxel::{BlockId, BlockPos, CHUNK_SIZE, Chunk, ChunkPos};
use deepcore_shared::worldgen::WorldGen;

/// Blocks known to the client.
///
/// The API is complete ahead of use: chunk streaming (M2) will insert and remove chunks as the
/// server sends them, and the HUD reads loaded positions for the debug overlay.
#[derive(Resource, Default)]
pub struct ClientWorld {
    chunks: HashMap<ChunkPos, Chunk>,
}

#[allow(dead_code)]
impl ClientWorld {
    /// Inserts or replaces a chunk.
    pub fn insert(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }

    /// Removes a chunk.
    pub fn remove(&mut self, pos: ChunkPos) -> Option<Chunk> {
        self.chunks.remove(&pos)
    }

    /// Number of loaded chunks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// True when no chunk is loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// True when the chunk containing `pos` is loaded.
    #[must_use]
    pub fn is_loaded(&self, pos: BlockPos) -> bool {
        self.chunks.contains_key(&ChunkPos::from_block(pos))
    }

    /// Reads a block. Unloaded chunks read as air so the player never collides with invisible
    /// terrain while streaming in.
    #[must_use]
    pub fn block_at(&self, pos: BlockPos) -> BlockId {
        if !pos.in_bounds() {
            return BlockId::AIR;
        }
        let chunk_pos = ChunkPos::from_block(pos);
        let Some(chunk) = self.chunks.get(&chunk_pos) else {
            return BlockId::AIR;
        };
        let (x, y, z) = pos.local();
        chunk.get(x, y, z)
    }

    /// Writes a block locally (prediction and server echo).
    pub fn set_block(&mut self, pos: BlockPos, block: BlockId) -> bool {
        let chunk_pos = ChunkPos::from_block(pos);
        let Some(chunk) = self.chunks.get_mut(&chunk_pos) else {
            return false;
        };
        let (x, y, z) = pos.local();
        chunk.set(x, y, z, block);
        true
    }

    /// The chunk containing `pos`.
    #[must_use]
    pub fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    /// All loaded chunk positions.
    pub fn positions(&self) -> Vec<ChunkPos> {
        self.chunks.keys().copied().collect()
    }

    /// Generates a chunk with the world generator, for local preview play.
    pub fn generate(&mut self, generator: &WorldGen, pos: ChunkPos) -> bool {
        if self.chunks.contains_key(&pos) || !pos.in_bounds() {
            return false;
        }
        self.chunks.insert(pos, generator.generate(pos));
        true
    }

    /// Generates every chunk within `radius` of `centre`, plus one layer above and below so the
    /// player can dig into the floor without falling out of the world.
    pub fn generate_area(&mut self, generator: &WorldGen, centre: ChunkPos, radius: i32) -> usize {
        let mut count = 0;
        for dy in -1..=1 {
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    if self.generate(
                        generator,
                        ChunkPos::new(centre.x + dx, centre.y + dy, centre.z + dz),
                    ) {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}

/// World-space position of the corner of a chunk.
#[must_use]
pub fn chunk_origin(pos: ChunkPos) -> Vec3 {
    Vec3::new(
        (pos.x * CHUNK_SIZE as i32) as f32,
        (pos.y * CHUNK_SIZE as i32) as f32,
        (pos.z * CHUNK_SIZE as i32) as f32,
    )
}
