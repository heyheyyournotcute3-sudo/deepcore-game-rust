//! Server world storage: generated chunks, edits, and (from M9) region persistence.
//!
//! Generation runs on a worker thread so the tick never blocks on it. Chunks are generated on
//! demand for the area around each player and cached; unmodified chunks are never written to
//! disk because they regenerate from the seed.
//!
//! The store API is complete here because M2 (chunk streaming) and M3 (mining) consume it; the
//! M0 binary only calls the spawn-area path.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use bevy::prelude::*;

use deepcore_shared::voxel::{BlockId, BlockPos, Chunk, ChunkPos, Dir};

use crate::World;

/// Chunk store for the running server.
#[derive(Resource, Default)]
pub struct ChunkStore {
    chunks: HashMap<ChunkPos, Arc<Chunk>>,
    /// Chunks modified since the last save. Pristine chunks regenerate from the seed.
    pub dirty: Vec<ChunkPos>,
}

impl ChunkStore {
    /// Reads a block, generating the chunk if it is not loaded.
    pub fn block_at(&mut self, world: &World, pos: BlockPos) -> BlockId {
        if !pos.in_bounds() {
            return BlockId::AIR;
        }
        let chunk_pos = ChunkPos::from_block(pos);
        self.ensure(world, chunk_pos);
        let (lx, ly, lz) = pos.local();
        self.chunks.get(&chunk_pos).map_or(BlockId::AIR, |c| c.get(lx, ly, lz))
    }

    /// Reads a block without generating anything.
    #[must_use]
    pub fn peek(&self, pos: BlockPos) -> Option<BlockId> {
        let chunk_pos = ChunkPos::from_block(pos);
        let (lx, ly, lz) = pos.local();
        self.chunks.get(&chunk_pos).map(|c| c.get(lx, ly, lz))
    }

    /// Ensures a chunk is loaded, generating it on this thread if needed.
    pub fn ensure(&mut self, world: &World, pos: ChunkPos) -> bool {
        if self.chunks.contains_key(&pos) || !pos.in_bounds() {
            return false;
        }
        let chunk = world.generator.generate(pos);
        self.chunks.insert(pos, Arc::new(chunk));
        true
    }

    /// Writes a block and records the chunk as dirty.
    pub fn set_block(&mut self, world: &World, pos: BlockPos, block: BlockId) -> bool {
        if !pos.in_bounds() {
            return false;
        }
        let chunk_pos = ChunkPos::from_block(pos);
        self.ensure(world, chunk_pos);
        let Some(existing) = self.chunks.get(&chunk_pos) else {
            return false;
        };
        let mut chunk = Chunk::clone(existing);
        let (lx, ly, lz) = pos.local();
        chunk.set(lx, ly, lz, block);
        self.chunks.insert(chunk_pos, Arc::new(chunk));
        if !self.dirty.contains(&chunk_pos) {
            self.dirty.push(chunk_pos);
        }
        true
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

    /// Shared handle to a loaded chunk.
    #[must_use]
    pub fn get(&self, pos: ChunkPos) -> Option<Arc<Chunk>> {
        self.chunks.get(&pos).cloned()
    }

    /// True when the block above `pos` is air, i.e. the block is exposed to the sky.
    #[must_use]
    pub fn exposed(&self, pos: BlockPos) -> bool {
        match self.peek(pos.offset_dir(Dir::PosY)) {
            Some(b) => b.is_air(),
            None => false,
        }
    }
}

/// Generation statistics.
#[derive(Resource, Default)]
pub struct WorldStats {
    /// Chunks generated since startup.
    pub generated: AtomicU32,
}

/// Registers world storage and the startup world build.
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkStore>()
            .init_resource::<WorldStats>()
            .add_systems(Startup, prepare_spawn_area);
    }
}

/// Generates the chunks around the camp so the first players land on solid ground, and logs
/// the world summary. Chunks elsewhere are generated lazily as players approach.
fn prepare_spawn_area(world: Res<World>, mut store: ResMut<ChunkStore>, stats: Res<WorldStats>) {
    let spawn = world.generator.spawn_point();
    let centre = ChunkPos::from_block(BlockPos::from_world(spawn));
    let mut count = 0;
    for dy in -1..=1 {
        for dz in -2..=2 {
            for dx in -2..=2 {
                if store.ensure(&world, ChunkPos::new(centre.x + dx, centre.y + dy, centre.z + dz))
                {
                    count += 1;
                }
            }
        }
    }
    stats.generated.fetch_add(count, Ordering::Relaxed);
    info!("camp prepared at {spawn:?}: {count} chunks generated, {} cached", store.len());
}
