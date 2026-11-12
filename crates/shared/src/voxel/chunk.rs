//! Palette-compressed chunk storage.
//!
//! A chunk stores a `Vec<BlockId>` palette plus one byte per block (or two bytes when the
//! palette grows past 255 entries). Air is always palette entry 0 so an all-air chunk costs
//! only the palette. Chunks remember whether they were modified so the server can skip saving
//! pristine terrain, which regenerates from the seed.

use serde::{Deserialize, Serialize};

use crate::voxel::pos::{CHUNK_VOLUME, index_of};

/// Numeric block id. `BlockId(0)` is always air.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct BlockId(pub u16);

impl BlockId {
    /// Air.
    pub const AIR: Self = Self(0);

    /// True when the block is air.
    #[must_use]
    pub const fn is_air(self) -> bool {
        self.0 == 0
    }

    /// Raw numeric value.
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// A 32×32×32 block of the world, palette compressed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    palette: Vec<BlockId>,
    /// One index per block into `palette`. Widened to `u16` when the palette exceeds 255.
    data: PaletteData,
    modified: bool,
    /// Number of non-air blocks, kept incrementally so meshing can skip empty chunks fast.
    solid_count: u32,
}

/// Storage for palette indices, either narrow (u8) or wide (u16).
#[derive(Debug, Clone, Serialize, Deserialize)]
enum PaletteData {
    /// Up to 256 palette entries.
    Narrow(Vec<u8>),
    /// More than 256 palette entries.
    Wide(Vec<u16>),
}

impl PaletteData {
    fn get(&self, i: usize) -> u16 {
        match self {
            Self::Narrow(v) => u16::from(v[i]),
            Self::Wide(v) => v[i],
        }
    }

    fn set(&mut self, i: usize, value: u16) {
        match self {
            Self::Narrow(v) => v[i] = value as u8,
            Self::Wide(v) => v[i] = value,
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Narrow(v) => v.len(),
            Self::Wide(v) => v.len(),
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    /// Creates an all-air chunk.
    #[must_use]
    pub fn new() -> Self {
        Self {
            palette: vec![BlockId::AIR],
            data: PaletteData::Narrow(vec![0; CHUNK_VOLUME]),
            modified: false,
            solid_count: 0,
        }
    }

    /// Creates a chunk filled with a single block type.
    #[must_use]
    pub fn filled(block: BlockId) -> Self {
        let mut chunk = Self::new();
        if !block.is_air() {
            chunk.palette.push(block);
            for i in 0..CHUNK_VOLUME {
                chunk.data.set(i, 1);
            }
            chunk.solid_count = CHUNK_VOLUME as u32;
        }
        chunk
    }

    /// Number of non-air blocks in the chunk.
    #[must_use]
    pub const fn solid_count(&self) -> u32 {
        self.solid_count
    }

    /// True when the chunk contains no solid blocks.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.solid_count == 0
    }

    /// True when the chunk differs from freshly generated terrain.
    #[must_use]
    pub const fn is_modified(&self) -> bool {
        self.modified
    }

    /// Clears the modified flag, e.g. after a save.
    pub fn clear_modified(&mut self) {
        self.modified = false;
    }

    /// Number of palette entries currently in use.
    #[must_use]
    pub fn palette_len(&self) -> usize {
        self.palette.len()
    }

    /// The palette, air first.
    #[must_use]
    pub fn palette(&self) -> &[BlockId] {
        &self.palette
    }

    /// Gets the block at local coordinates. Out-of-range coordinates return air.
    #[must_use]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        if x >= 32 || y >= 32 || z >= 32 {
            return BlockId::AIR;
        }
        self.palette[self.data.get(index_of(x, y, z)) as usize]
    }

    /// Sets the block at local coordinates and marks the chunk modified.
    ///
    /// Returns the previous block. Out-of-range coordinates are ignored.
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) -> BlockId {
        if x >= 32 || y >= 32 || z >= 32 {
            return BlockId::AIR;
        }
        let idx = index_of(x, y, z);
        let old_index = self.data.get(idx);
        let old = self.palette[old_index as usize];
        if old == block {
            return old;
        }
        let new_index = self.palette_index_for(block);
        self.data.set(idx, new_index);
        if old.is_air() && !block.is_air() {
            self.solid_count += 1;
        } else if !old.is_air() && block.is_air() {
            self.solid_count -= 1;
        }
        self.modified = true;
        old
    }

    /// Finds or inserts a palette entry for `block`.
    fn palette_index_for(&mut self, block: BlockId) -> u16 {
        if let Some(i) = self.palette.iter().position(|&b| b == block) {
            return i as u16;
        }
        let index = self.palette.len();
        if index > u16::MAX as usize {
            // Impossible in practice: a chunk has 32768 blocks, so the palette can never
            // exceed that, which is well under u16::MAX. Documented invariant.
            tracing::error!("chunk palette overflowed u16; reusing air");
            return 0;
        }
        self.palette.push(block);
        if index > u8::MAX as usize {
            // Widen narrow storage to wide storage in place.
            if let PaletteData::Narrow(narrow) = &self.data {
                let wide: Vec<u16> = narrow.iter().map(|&v| u16::from(v)).collect();
                self.data = PaletteData::Wide(wide);
            }
        }
        index as u16
    }

    /// Packs the chunk for the wire: palette + indices, then compressed by the caller.
    ///
    /// Returns `(palette, indices)` with indices widened to `u16` so the format is stable
    /// regardless of how many palette entries a chunk happens to use.
    #[must_use]
    pub fn to_raw_parts(&self) -> (Vec<BlockId>, Vec<u16>) {
        let indices = (0..self.data.len()).map(|i| self.data.get(i)).collect();
        (self.palette.clone(), indices)
    }

    /// Rebuilds a chunk from [`Chunk::to_raw_parts`] output.
    #[must_use]
    pub fn from_raw_parts(palette: Vec<BlockId>, indices: Vec<u16>) -> Option<Self> {
        if palette.is_empty() || indices.len() != CHUNK_VOLUME {
            return None;
        }
        let max = palette.len() as u16;
        if indices.iter().any(|&i| i >= max) {
            return None;
        }
        let solid_count =
            indices.iter().filter(|&&i| palette[i as usize] != BlockId::AIR).count() as u32;
        let narrow = max <= u16::from(u8::MAX) + 1;
        let data = if narrow {
            PaletteData::Narrow(indices.iter().map(|&i| i as u8).collect())
        } else {
            PaletteData::Wide(indices)
        };
        Some(Self { palette, data, modified: true, solid_count })
    }

    /// Serialises the chunk to a compressed byte blob (postcard + zstd).
    ///
    /// # Errors
    /// Returns [`crate::voxel::VoxelError`] if serialisation or compression fails.
    pub fn encode(&self) -> Result<Vec<u8>, crate::voxel::VoxelError> {
        let (palette, indices) = self.to_raw_parts();
        let payload = postcard::to_allocvec(&(palette, indices))?;
        Ok(zstd::bulk::compress(&payload, 3)?)
    }

    /// Decodes a blob produced by [`Chunk::encode`].
    ///
    /// # Errors
    /// Returns [`crate::voxel::VoxelError`] if the blob is corrupt or not a chunk.
    pub fn decode(bytes: &[u8]) -> Result<Self, crate::voxel::VoxelError> {
        let raw = zstd::bulk::decompress(bytes, 1 << 22)?;
        let (palette, indices): (Vec<BlockId>, Vec<u16>) = postcard::from_bytes(&raw)?;
        Self::from_raw_parts(palette, indices).ok_or(crate::voxel::VoxelError::MalformedChunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_chunk_is_all_air() {
        let c = Chunk::new();
        assert!(c.is_empty());
        assert_eq!(c.get(0, 0, 0), BlockId::AIR);
        assert_eq!(c.palette_len(), 1);
    }

    #[test]
    fn set_and_get_round_trip() {
        let mut c = Chunk::new();
        c.set(1, 2, 3, BlockId(7));
        assert_eq!(c.get(1, 2, 3), BlockId(7));
        assert_eq!(c.solid_count(), 1);
        c.set(1, 2, 3, BlockId::AIR);
        assert_eq!(c.solid_count(), 0);
        assert_eq!(c.get(1, 2, 3), BlockId::AIR);
    }

    #[test]
    fn palette_compression_keeps_palette_small() {
        let mut c = Chunk::new();
        for x in 0..32 {
            for z in 0..32 {
                c.set(x, 0, z, if x % 2 == 0 { BlockId(1) } else { BlockId(2) });
            }
        }
        assert_eq!(c.palette_len(), 3, "air + two block types");
        assert_eq!(c.solid_count(), 32 * 32);
    }

    #[test]
    fn palette_widens_past_255_entries() {
        let mut c = Chunk::new();
        for i in 0..300usize {
            let (x, y, z) = crate::voxel::pos::coords_of(i);
            c.set(x, y, z, BlockId((i + 1) as u16));
        }
        assert_eq!(c.palette_len(), 301);
        for i in 0..300usize {
            let (x, y, z) = crate::voxel::pos::coords_of(i);
            assert_eq!(c.get(x, y, z), BlockId((i + 1) as u16));
        }
    }

    #[test]
    fn raw_parts_round_trip() {
        let mut c = Chunk::filled(BlockId(3));
        c.set(5, 5, 5, BlockId(9));
        let (p, i) = c.to_raw_parts();
        let r = Chunk::from_raw_parts(p, i).expect("valid parts");
        assert_eq!(r.get(5, 5, 5), BlockId(9));
        assert_eq!(r.get(0, 0, 0), BlockId(3));
        assert_eq!(r.solid_count(), c.solid_count());
    }

    #[test]
    fn from_raw_parts_rejects_garbage() {
        assert!(Chunk::from_raw_parts(vec![], vec![0; CHUNK_VOLUME]).is_none());
        assert!(Chunk::from_raw_parts(vec![BlockId::AIR], vec![0; 10]).is_none());
        // Index out of palette range.
        assert!(Chunk::from_raw_parts(vec![BlockId::AIR], vec![5; CHUNK_VOLUME]).is_none());
    }

    #[test]
    fn encode_decode_round_trip() {
        let mut c = Chunk::filled(BlockId(4));
        c.set(31, 31, 31, BlockId::AIR);
        let bytes = c.encode().expect("encode");
        let d = Chunk::decode(&bytes).expect("decode");
        assert_eq!(d.get(0, 0, 0), BlockId(4));
        assert_eq!(d.get(31, 31, 31), BlockId::AIR);
        assert_eq!(d.solid_count(), c.solid_count());
    }

    #[test]
    fn modified_flag_tracks_edits() {
        let mut c = Chunk::new();
        assert!(!c.is_modified());
        c.set(0, 0, 0, BlockId(1));
        assert!(c.is_modified());
        c.clear_modified();
        assert!(!c.is_modified());
    }
}
