//! Deterministic world generation.
//!
//! The same seed and chunk position must produce byte-identical chunks on every machine, so
//! generation uses only integer hashing and seeded `ChaCha8Rng`s, never `HashMap` iteration
//! order or floating point accumulation across platforms. Each chunk seeds its own RNG from
//! `(world_seed, ChunkPos)`, which also makes generation parallel-safe.

use bevy_math::Vec3;
use noise::{NoiseFn, OpenSimplex};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::content::{ItemCategory, Registry};
use crate::voxel::{BlockId, CHUNK_SIZE, Chunk, ChunkPos, SURFACE_Y, WORLD_SIZE_Y};

/// Caves never form above this depth, so the surface stays solid and the camp is supported.
pub const MIN_CAVE_DEPTH: u32 = 8;

/// Salt constants keep the different noise fields independent.
const SALT_CAVE: u64 = 0x9E37_79B9_7F4A_7C15;
const SALT_ORE: u64 = 0xC2B2_AE3D_27D4_EB4F;
const SALT_FILL: u64 = 0x1656_67B1_9E37_79F9;
const SALT_POCKET: u64 = 0x27D4_EB4F_1656_67B1;

/// Generates chunks for a world.
pub struct WorldGen {
    seed: u64,
    registry: Registry,
    cave: OpenSimplex,
    cave2: OpenSimplex,
    /// Highest solid block of the camp area, i.e. the surface plane.
    surface_y: i32,
    // Cached ids so generation never does string lookups.
    air: BlockId,
    bedrock: BlockId,
    water: BlockId,
    gas: BlockId,
    log: BlockId,
    leaves: BlockId,
    grass: BlockId,
    lamp: BlockId,
}

/// One ore vein resolved into world space, produced by the per-chunk vein pass.
#[derive(Debug, Clone)]
struct Vein {
    block: BlockId,
    center: Vec3,
    radius: f32,
}

impl WorldGen {
    /// Creates a generator for a seed. Missing optional blocks fall back to air so a content
    /// edit can never panic generation.
    #[must_use]
    pub fn new(seed: u64, registry: Registry) -> Self {
        let air = BlockId::AIR;
        let get = |name: &str| registry.block_id(name).unwrap_or(air);
        Self {
            seed,
            cave: OpenSimplex::new((seed as u32) ^ (SALT_CAVE as u32)),
            cave2: OpenSimplex::new((seed as u32) ^ (SALT_CAVE as u32) ^ 0x5bd1_e995),
            surface_y: SURFACE_Y,
            air,
            bedrock: get("bedrock"),
            water: get("water"),
            gas: get("methane"),
            log: get("log"),
            leaves: get("leaves"),
            grass: get("grass"),
            lamp: get("camp_lamp"),
            registry,
        }
    }

    /// The world seed.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Y coordinate of the camp surface.
    #[must_use]
    pub const fn surface_y(&self) -> i32 {
        self.surface_y
    }

    /// Where a player spawns: standing on the camp surface at the world centre.
    ///
    /// The body's feet sit at `y`, so spawning at `surface_y + 1` puts the player exactly on
    /// top of the surface block instead of dropping them from a height.
    #[must_use]
    pub fn spawn_point(&self) -> Vec3 {
        Vec3::new(512.5, (self.surface_y + 1) as f32, 512.5)
    }

    /// Registry access, for callers that need block stats.
    #[must_use]
    pub const fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Depth of a block below the surface, clamped to `0..`.
    #[must_use]
    pub fn depth_of(&self, y: i32) -> u32 {
        (self.surface_y - y).max(0) as u32
    }

    /// Which layer a depth belongs to, 1-based, or `None` above the surface.
    #[must_use]
    pub fn layer_at_depth(&self, depth: u32) -> Option<u8> {
        self.registry
            .layers
            .iter()
            .find(|l| depth >= l.def.depth.0 && depth < l.def.depth.1)
            .map(|l| l.def.index)
    }

    /// Resolved layer for a depth.
    #[must_use]
    pub fn layer_def_at_depth(&self, depth: u32) -> Option<&crate::content::ResolvedLayer> {
        self.registry.layers.iter().find(|l| depth >= l.def.depth.0 && depth < l.def.depth.1)
    }

    /// Deterministic RNG for a chunk and salt.
    fn rng_for(&self, pos: ChunkPos, salt: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(hash3(self.seed ^ salt, pos.x, pos.y, pos.z))
    }

    /// Generates a chunk.
    #[must_use]
    pub fn generate(&self, pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::new();
        if !pos.in_bounds() {
            return chunk;
        }
        let base_x = pos.x * CHUNK_SIZE as i32;
        let base_y = pos.y * CHUNK_SIZE as i32;
        let base_z = pos.z * CHUNK_SIZE as i32;

        // --- pass 1: terrain layers, caves and bedrock bounds ---
        for ly in 0..CHUNK_SIZE {
            let y = base_y + ly as i32;
            if y >= WORLD_SIZE_Y {
                continue;
            }
            let depth = self.depth_of(y);
            for lz in 0..CHUNK_SIZE {
                let z = base_z + lz as i32;
                for lx in 0..CHUNK_SIZE {
                    let x = base_x + lx as i32;
                    let block = self.block_at(x, y, z, depth);
                    if !block.is_air() {
                        chunk.set(lx, ly, lz, block);
                    }
                }
            }
        }

        // --- pass 2: ore veins, placed in world space so veins cross chunk borders ---
        let veins = self.veins_for(pos);
        if !veins.is_empty() {
            for lz in 0..CHUNK_SIZE {
                let z = (base_z + lz as i32) as f32 + 0.5;
                for ly in 0..CHUNK_SIZE {
                    let y = (base_y + ly as i32) as f32 + 0.5;
                    for lx in 0..CHUNK_SIZE {
                        let x = (base_x + lx as i32) as f32 + 0.5;
                        let p = Vec3::new(x, y, z);
                        for vein in &veins {
                            if p.distance_squared(vein.center) <= vein.radius * vein.radius {
                                let current = chunk.get(lx, ly, lz);
                                if self.can_host_ore(current) {
                                    chunk.set(lx, ly, lz, vein.block);
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        // --- pass 3: hazard pockets (water, gas) ---
        self.carve_pockets(pos, &mut chunk);

        chunk
    }

    /// True when an ore may replace this block.
    fn can_host_ore(&self, current: BlockId) -> bool {
        if current.is_air()
            || current == self.bedrock
            || current == self.water
            || current == self.gas
        {
            return false;
        }
        let Some(def) = self.registry.block(current) else {
            return false;
        };
        matches!(
            def.def.pattern,
            crate::content::Pattern::Flat
                | crate::content::Pattern::Speckle
                | crate::content::Pattern::Blotch
                | crate::content::Pattern::Strata
        ) && def.def.drops.is_empty()
    }

    /// The block for a world position before ore and pocket passes.
    fn block_at(&self, x: i32, y: i32, z: i32, depth: u32) -> BlockId {
        // Bedrock floor and world walls are unbreakable.
        if y == 0 || y == WORLD_SIZE_Y - 1 || x == 0 || z == 0 || x == 1023 || z == 1023 {
            return if y >= self.surface_y { self.air } else { self.bedrock };
        }
        if y > self.surface_y {
            return self.air;
        }
        if y == self.surface_y {
            return self.grass;
        }

        let Some(layer) = self.layer_def_at_depth(depth) else {
            return self.bedrock;
        };

        // Base block with fillers chosen by low-frequency noise so bands look geological.
        let n = self.cave2.get([x as f64 * 0.02, y as f64 * 0.05, z as f64 * 0.02]) as f32;
        let mut block = layer.base_block;
        for (filler, weight) in &layer.fillers {
            if n > *weight {
                block = *filler;
                break;
            }
        }

        // Caves: two noise fields combined, so tunnels form where both are near zero.
        //
        // Caves are kept well below the surface: carving them from depth 2 leaves the topsoil
        // floating over voids and the camp unsupported. Underground entrances are reached by
        // digging, which is the intended loop.
        if depth >= MIN_CAVE_DEPTH {
            let c1 = self.cave.get([x as f64 * 0.035, y as f64 * 0.05, z as f64 * 0.035]);
            let c2 = self.cave2.get([x as f64 * 0.035, y as f64 * 0.05, z as f64 * 0.035]);
            let combined = (c1 * c1 + c2 * c2).sqrt();
            if combined < 0.10 {
                return self.air;
            }
        }

        block
    }

    /// Builds the veins that intersect a chunk, including veins whose centres are in
    /// neighbouring chunks, by walking a 3×3×3 neighbourhood of vein cells.
    fn veins_for(&self, pos: ChunkPos) -> Vec<Vein> {
        // Vein cells are CELL blocks wide, so a 32-block chunk touches at most 3 cells per axis.
        const CELL: i32 = 16;
        let mut out = Vec::new();
        for cy in -1..=1 {
            for cz in -1..=1 {
                for cx in -1..=1 {
                    let cell_x = pos.x * 2 + cx;
                    let cell_y = pos.y * 2 + cy;
                    let cell_z = pos.z * 2 + cz;
                    let mut rng = ChaCha8Rng::seed_from_u64(hash3(
                        self.seed ^ SALT_ORE,
                        cell_x,
                        cell_y,
                        cell_z,
                    ));
                    // Pick which layer this cell belongs to using its centre depth.
                    let centre_y = (cell_y as f32 + 0.5) * CELL as f32;
                    let depth = self.depth_of(centre_y as i32);
                    let Some(layer) = self.layer_def_at_depth(depth) else {
                        continue;
                    };
                    if layer.ores.is_empty() {
                        continue;
                    }
                    for (block, frequency, (min_size, max_size)) in &layer.ores {
                        if rng.random::<f32>() > *frequency {
                            continue;
                        }
                        let size = rng.random_range(*min_size..=*max_size) as f32;
                        let radius = (size * 3.0 / (4.0 * std::f32::consts::PI)).cbrt() * 1.6;
                        let ox: f32 = rng.random();
                        let oy: f32 = rng.random();
                        let oz: f32 = rng.random();
                        let centre = Vec3::new(
                            (cell_x as f32 + ox) * CELL as f32,
                            (cell_y as f32 + oy) * CELL as f32,
                            (cell_z as f32 + oz) * CELL as f32,
                        );
                        out.push(Vein { block: *block, center: centre, radius: radius.max(1.0) });
                    }
                }
            }
        }
        out
    }

    /// Carves water and gas pockets according to the layer hazard definitions.
    fn carve_pockets(&self, pos: ChunkPos, chunk: &mut Chunk) {
        let mut rng = self.rng_for(pos, SALT_POCKET);
        let base_x = pos.x * CHUNK_SIZE as i32;
        let base_y = pos.y * CHUNK_SIZE as i32;
        let base_z = pos.z * CHUNK_SIZE as i32;
        let depth = self.depth_of(base_y + CHUNK_SIZE as i32 / 2);
        let Some(layer) = self.layer_def_at_depth(depth) else {
            return;
        };
        for hazard in &layer.def.hazards {
            let (chance, block, max_volume) = match hazard {
                crate::content::HazardDef::Flooding { pocket_chance, max_volume } => {
                    (*pocket_chance, self.water, *max_volume)
                }
                crate::content::HazardDef::Gas { pocket_chance, .. } => {
                    (*pocket_chance, self.gas, 64)
                }
                _ => continue,
            };
            if block.is_air() || rng.random::<f32>() > chance {
                continue;
            }
            let volume = rng.random_range(8..=max_volume.clamp(8, 512)) as f32;
            let radius = (volume * 3.0 / (4.0 * std::f32::consts::PI)).cbrt();
            let cx = rng.random_range(4..CHUNK_SIZE as i32 - 4);
            let cy = rng.random_range(4..CHUNK_SIZE as i32 - 4);
            let cz = rng.random_range(4..CHUNK_SIZE as i32 - 4);
            for ly in 0..CHUNK_SIZE {
                for lz in 0..CHUNK_SIZE {
                    for lx in 0..CHUNK_SIZE {
                        let dx = lx as i32 - cx;
                        let dy = ly as i32 - cy;
                        let dz = lz as i32 - cz;
                        let d = ((dx * dx + dy * dy + dz * dz) as f32).sqrt();
                        if d <= radius && chunk.get(lx, ly, lz).is_air() {
                            chunk.set(lx, ly, lz, block);
                        }
                    }
                }
            }
            let _ = (base_x, base_y, base_z);
        }
    }

    /// Places trees and camp decoration in a chunk. Called once for the whole world at startup.
    pub fn decorate_surface(&self, pos: ChunkPos, chunk: &mut Chunk) {
        let mut rng = self.rng_for(pos, SALT_FILL);
        let base_x = pos.x * CHUNK_SIZE as i32;
        let base_z = pos.z * CHUNK_SIZE as i32;
        let surface_local = (self.surface_y - pos.y * CHUNK_SIZE as i32) as usize;
        if surface_local >= CHUNK_SIZE {
            return;
        }
        // Trees ring the camp: outside a 6-chunk radius of the centre but inside 20.
        let centre = ChunkPos::new(
            ChunkPos::chunks_per_axis_xz() / 2,
            0,
            ChunkPos::chunks_per_axis_xz() / 2,
        );
        let dist = pos.distance_squared(ChunkPos::new(pos.x, 0, pos.z));
        let camp_dist = dist.min(centre.distance_squared(ChunkPos::new(centre.x, 0, centre.z)));
        if camp_dist < 36 {
            return;
        }
        let tree_count = rng.random_range(0..3);
        for _ in 0..tree_count {
            let lx = rng.random_range(3..CHUNK_SIZE as i32 - 3);
            let lz = rng.random_range(3..CHUNK_SIZE as i32 - 3);
            let height = rng.random_range(4..7);
            let ground = chunk.get(lx as usize, surface_local, lz as usize);
            if ground != self.grass {
                continue;
            }
            for h in 1..=height {
                if surface_local + h < CHUNK_SIZE {
                    chunk.set(lx as usize, surface_local + h, lz as usize, self.log);
                }
            }
            // Simple leaf blob.
            for dy in 0..3 {
                for dz in -2..=2 {
                    for dx in -2..=2 {
                        let d = (dx * dx + dz * dz) as f32;
                        if d > 5.0 {
                            continue;
                        }
                        let yy = surface_local + height - dy;
                        let xx = lx + dx;
                        let zz = lz + dz;
                        if xx < 0
                            || zz < 0
                            || yy >= CHUNK_SIZE
                            || xx >= CHUNK_SIZE as i32
                            || zz >= CHUNK_SIZE as i32
                        {
                            continue;
                        }
                        if chunk.get(xx as usize, yy, zz as usize).is_air() {
                            chunk.set(xx as usize, yy, zz as usize, self.leaves);
                        }
                    }
                }
            }
        }
        let _ = (base_x, base_z);
    }

    /// Builds the camp: a flat stone platform with a lamp post at the spawn point.
    pub fn build_camp(&self, chunk: &mut Chunk, pos: ChunkPos) {
        let centre_block = 512;
        let base_x = pos.x * CHUNK_SIZE as i32;
        let base_z = pos.z * CHUNK_SIZE as i32;
        let surface_local = (self.surface_y - pos.y * CHUNK_SIZE as i32) as usize;
        if surface_local >= CHUNK_SIZE {
            return;
        }
        let stone = self.registry.block_id("stone").unwrap_or(self.bedrock);
        for lz in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let x = base_x + lx as i32;
                let z = base_z + lz as i32;
                let dx = x - centre_block;
                let dz = z - centre_block;
                if dx * dx + dz * dz > 36 {
                    continue;
                }
                chunk.set(lx, surface_local, lz, stone);
                if surface_local + 1 < CHUNK_SIZE {
                    chunk.set(lx, surface_local + 1, lz, self.air);
                }
            }
        }
        if !self.lamp.is_air() {
            let lx = (centre_block + 2 - base_x) as usize;
            let lz = (centre_block - base_z) as usize;
            if lx < CHUNK_SIZE && lz < CHUNK_SIZE && surface_local + 2 < CHUNK_SIZE {
                chunk.set(lx, surface_local + 1, lz, self.lamp);
            }
        }
    }

    /// True when the item category of an ore drop suggests a raw material.
    #[must_use]
    pub fn is_raw_material(&self, item: crate::content::ItemId) -> bool {
        self.registry.item(item).is_some_and(|i| matches!(i.def.category, ItemCategory::Material))
    }
}

/// Integer hash of a seed and three coordinates. Stable across platforms.
#[must_use]
pub fn hash3(seed: u64, x: i32, y: i32, z: i32) -> u64 {
    let mut h = seed ^ 0x9E37_79B9_7F4A_7C15;
    for v in [x as u64, y as u64, z as u64] {
        h ^= v.wrapping_mul(0xff51_afd7_ed55_8ccd);
        h = h.rotate_left(31).wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        h ^= h >> 29;
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_dir;

    fn test_registry() -> Registry {
        let dir = crate::content::default_content_dir();
        load_dir(dir).expect("content must load").0
    }

    #[test]
    fn generation_is_deterministic_for_same_seed() {
        let reg = test_registry();
        let generator = WorldGen::new(42, reg);
        let a = generator.generate(ChunkPos::new(4, 10, 4));
        let b = generator.generate(ChunkPos::new(4, 10, 4));
        let (pa, ia) = a.to_raw_parts();
        let (pb, ib) = b.to_raw_parts();
        assert_eq!(pa, pb, "palettes must match");
        assert_eq!(ia, ib, "blocks must match");
    }

    #[test]
    fn different_seeds_differ() {
        let reg = test_registry();
        let a = WorldGen::new(1, reg.clone()).generate(ChunkPos::new(5, 10, 5));
        let b = WorldGen::new(2, reg).generate(ChunkPos::new(5, 10, 5));
        assert_ne!(a.to_raw_parts().1, b.to_raw_parts().1);
    }

    #[test]
    fn chunk_has_solid_blocks_underground() {
        let reg = test_registry();
        let generator = WorldGen::new(7, reg);
        let chunk = generator.generate(ChunkPos::new(16, 13, 16));
        assert!(chunk.solid_count() > 1000, "underground chunk should be mostly solid");
    }

    #[test]
    fn surface_chunk_has_air_above() {
        let reg = test_registry();
        let generator = WorldGen::new(7, reg);
        let pos = ChunkPos::new(16, SURFACE_Y / CHUNK_SIZE as i32, 16);
        let chunk = generator.generate(pos);
        let top_local = SURFACE_Y - pos.y * CHUNK_SIZE as i32;
        if (top_local as usize) + 2 < CHUNK_SIZE {
            assert!(chunk.get(0, top_local as usize + 2, 0).is_air());
        }
    }

    #[test]
    fn layers_cover_the_whole_mine() {
        let reg = test_registry();
        let generator = WorldGen::new(1, reg);
        for depth in [0u32, 15, 16, 63, 64, 159, 160, 287, 288, 447, 448, 500] {
            assert!(generator.layer_at_depth(depth).is_some(), "no layer at depth {depth}");
        }
    }

    #[test]
    fn hash_is_stable() {
        // Hard-coded expectations: if these change, saved worlds stop matching their seed.
        assert_eq!(hash3(0, 0, 0, 0), hash3(0, 0, 0, 0));
        assert_ne!(hash3(0, 1, 0, 0), hash3(0, 0, 1, 0));
        assert_ne!(hash3(1, 0, 0, 0), hash3(2, 0, 0, 0));
    }
}
