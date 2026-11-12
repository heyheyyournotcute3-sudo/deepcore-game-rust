//! A headless simulation of the mining loop, used by tests and by the `sim_check` binary.
//!
//! This is not a second implementation of the game: it calls the same [`WorldGen`], the same
//! [`Chunk`] storage, the same [`step`] physics and the same break-time rules the client and
//! server use. It exists so the M1 acceptance criteria can be checked without a window.

use bevy_math::Vec3;

use crate::content::{ItemId, Registry};
use crate::crafting::{MineCheck, check_mine, ticks_for};
use crate::movement::{BodyState, MoveIntent, MovementConfig, query_from_blocks, step};
use crate::voxel::{BlockId, BlockPos, Chunk, ChunkPos, SURFACE_Y};
use crate::worldgen::WorldGen;

/// A single-chunk-column world: one chunk wide, from surface down to bedrock.
///
/// Digging straight down only ever touches one chunk column, which keeps the harness tiny while
/// still exercising real generation, real collision and real break times.
pub struct SimWorld {
    /// The registry, for block stats and tool stats.
    pub registry: Registry,
    /// The generator, for lazily generated chunks.
    pub generator: WorldGen,
    chunks: std::collections::HashMap<ChunkPos, Chunk>,
}

impl SimWorld {
    /// Builds a world for a seed, generating the chunk under the spawn column.
    #[must_use]
    pub fn new(seed: u64, registry: Registry) -> Self {
        let generator = WorldGen::new(seed, registry.clone());
        let mut world = Self { registry, generator, chunks: std::collections::HashMap::new() };
        world.ensure_column(512, 512);
        world
    }

    /// Generates every chunk in the vertical column above and below the given x/z.
    pub fn ensure_column(&mut self, x: i32, z: i32) {
        let top = ChunkPos::chunks_per_axis_y();
        for y in 0..top {
            let pos = ChunkPos::from_block(BlockPos::new(x, y * 32, z));
            if !self.chunks.contains_key(&pos) {
                self.chunks.insert(pos, self.generator.generate(pos));
            }
        }
    }

    /// Block at a world position.
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

    /// Removes a block.
    pub fn set_block(&mut self, pos: BlockPos, block: BlockId) {
        let chunk_pos = ChunkPos::from_block(pos);
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            let (x, y, z) = pos.local();
            chunk.set(x, y, z, block);
        }
    }

    /// Breaks a block with a tool, exactly as the server would, returning the item drops.
    ///
    /// Returns `None` when the tool cannot break the block (tier too low, or unbreakable).
    pub fn mine(&mut self, pos: BlockPos, tool: ItemId) -> Option<Vec<(ItemId, u32)>> {
        let def = self.registry.block(self.block_at(pos))?.clone();
        let tool_def = self.registry.tool(tool);
        match check_mine(def.def.hardness, def.def.min_tier, def.def.unbreakable, tool_def, 1.0) {
            MineCheck::Allowed { .. } => {
                self.set_block(pos, BlockId::AIR);
                Some(def.drops.clone())
            }
            MineCheck::TooHard { .. } | MineCheck::Unbreakable => None,
        }
    }

    /// Ticks needed to break a block with a tool, or `None` if it cannot be broken.
    #[must_use]
    pub fn ticks_to_break(&self, pos: BlockPos, tool: ItemId) -> Option<u32> {
        let def = self.registry.block(self.block_at(pos))?;
        let tool_def = self.registry.tool(tool);
        match check_mine(def.def.hardness, def.def.min_tier, def.def.unbreakable, tool_def, 1.0) {
            MineCheck::Allowed { seconds } => Some(ticks_for(seconds, crate::TICK_SECONDS).max(1)),
            MineCheck::TooHard { .. } | MineCheck::Unbreakable => None,
        }
    }

    /// Digs straight down from a starting position until the tool can no longer break the block.
    ///
    /// Returns `(blocks_destroyed, lowest_block_broken, stopping_block)`.
    pub fn dig_down(
        &mut self,
        start: BlockPos,
        tool: ItemId,
        max_blocks: u32,
    ) -> (u32, Option<BlockPos>, Option<(BlockPos, String)>) {
        let mut count = 0;
        let mut lowest = None;
        let mut pos = start;
        for _ in 0..max_blocks {
            let block = self.block_at(pos);
            if block.is_air() {
                // Fall through air until we reach something solid.
                pos.y -= 1;
                if pos.y <= 0 {
                    break;
                }
                continue;
            }
            if self.mine(pos, tool).is_some() {
                count += 1;
                lowest = Some(pos);
                if pos.y == 0 {
                    // Bedrock floor: nothing below it.
                    return (count, lowest, None);
                }
                pos.y -= 1;
            } else {
                let name = self
                    .registry
                    .block(block)
                    .map_or_else(|| "unknown".to_string(), |b| b.def.id.clone());
                return (count, lowest, Some((pos, name)));
            }
        }
        (count, lowest, None)
    }

    /// True when the player's body is standing on something solid.
    #[must_use]
    pub fn body_rests_on_ground(&self, body: &BodyState, config: &MovementConfig) -> bool {
        let query = query_from_blocks(|p| self.block_at(p), BlockId::AIR);
        let stepped = step(*body, &MoveIntent::default(), config, crate::TICK_SECONDS, &query);
        stepped.body.on_ground
    }
}

/// Depth below the surface of a block position.
#[must_use]
pub fn depth_of(pos: BlockPos) -> i32 {
    SURFACE_Y - pos.y
}

/// The spawn position used by the simulation.
#[must_use]
pub fn sim_spawn() -> Vec3 {
    Vec3::new(512.5, (SURFACE_Y + 1) as f32, 512.5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::default_content_dir;

    fn world() -> SimWorld {
        let registry =
            crate::content::load_dir(default_content_dir()).expect("content must load").0;
        SimWorld::new(0x000D_33C0, registry)
    }

    #[test]
    fn player_starts_on_solid_ground() {
        let world = world();
        let body = BodyState { position: sim_spawn(), ..Default::default() };
        let config = MovementConfig::default();
        // Let gravity settle the body for a second.
        let query = query_from_blocks(|p| world.block_at(p), BlockId::AIR);
        let mut settled = body;
        for _ in 0..60 {
            settled =
                step(settled, &MoveIntent::default(), &config, crate::TICK_SECONDS, &query).body;
        }
        assert!(settled.on_ground, "the player must land on the camp surface");
        // Feet rest on top of the surface block.
        let expected = (SURFACE_Y + 1) as f32;
        assert!(
            (settled.position.y - expected).abs() < 0.05,
            "feet should rest at {expected}, were at {}",
            settled.position.y
        );
        assert!(
            world.body_rests_on_ground(&settled, &config),
            "the settled body must be supported"
        );
    }

    #[test]
    fn steel_pickaxe_digs_through_layers_one_and_two() {
        // M1 acceptance: dig from the surface to the bottom of layer 2 (depth 64).
        let mut world = world();
        let pickaxe = world.registry.item_id("steel_pickaxe").expect("pickaxe");
        let start = BlockPos::new(512, SURFACE_Y - 1, 512);
        let (count, lowest, stop) = world.dig_down(start, pickaxe, 200);
        assert!(count > 40, "should break a real tunnel, broke {count}");
        let lowest = lowest.expect("must break at least one block");
        let depth = depth_of(lowest);
        assert!(
            depth >= 60,
            "steel pickaxe should reach the bottom of layer 2, reached depth {depth}"
        );
        // It must stop at layer 3 granite (depth 64..160, min_tier 2).
        let (stop_pos, stop_block) = stop.expect("must be blocked by layer 3");
        assert_eq!(stop_block, "granite", "layer 3 base block stops the pickaxe");
        assert!(
            (64..160).contains(&depth_of(stop_pos)),
            "the stop must be inside the Granite Depths, was depth {}",
            depth_of(stop_pos)
        );
    }

    #[test]
    fn hardened_pickaxe_passes_the_granite_that_stopped_steel() {
        let mut world = world();
        let steel = world.registry.item_id("steel_pickaxe").expect("steel");
        let hardened = world.registry.item_id("hardened_pickaxe").expect("hardened");
        let start = BlockPos::new(512, SURFACE_Y - 1, 512);
        let (_, _, steel_stop) = world.dig_down(start, steel, 400);
        let (steel_stop_pos, _) = steel_stop.expect("steel must be blocked");

        // Fresh world so the steel tunnel does not help.
        let registry = world.registry.clone();
        let mut world2 = SimWorld::new(0x000D_33C0, registry);
        let (_, lowest, _) = world2.dig_down(steel_stop_pos, hardened, 200);
        assert!(
            lowest.is_some_and(|p| p.y < steel_stop_pos.y),
            "the tier-2 pickaxe must dig past where tier 1 stopped"
        );
    }

    #[test]
    fn bedrock_floor_is_never_broken() {
        let mut world = world();
        let nuclear = world.registry.item_id("nuclear_drill").expect("drill");
        let start = BlockPos::new(512, 4, 512);
        let (_, lowest, stop) = world.dig_down(start, nuclear, 20);
        assert!(stop.is_some(), "digging must stop at bedrock");
        assert!(lowest.is_none_or(|p| p.y > 0), "bedrock at y = 0 must survive");
        assert_eq!(
            world.block_at(BlockPos::new(512, 0, 512)),
            world.registry.block_id("bedrock").unwrap()
        );
    }

    #[test]
    fn break_times_get_faster_with_better_tools() {
        let world = world();
        let steel = world.registry.item_id("steel_pickaxe").expect("steel");
        let hardened = world.registry.item_id("hardened_pickaxe").expect("hardened");
        let nuclear = world.registry.item_id("nuclear_drill").expect("nuclear");
        // Find a dirt block near the surface.
        let mut pos = BlockPos::new(512, SURFACE_Y - 1, 512);
        while world.block_at(pos).is_air() && pos.y > 1 {
            pos.y -= 1;
        }
        let steel_ticks = world.ticks_to_break(pos, steel).expect("breakable");
        let hardened_ticks = world.ticks_to_break(pos, hardened).expect("breakable");
        let nuclear_ticks = world.ticks_to_break(pos, nuclear).expect("breakable");
        assert!(hardened_ticks < steel_ticks, "tier 2 is faster than tier 1");
        assert!(nuclear_ticks < hardened_ticks, "the nuclear drill is fastest");
    }
}
