//! Break-time rules, shared by the client (prediction, crack animation) and the server
//! (authority). One formula, one place.

use crate::content::{ItemId, Registry, ToolDef};

/// Result of asking whether a block can be mined with a given tool.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MineCheck {
    /// The block can be broken, in this many seconds.
    Allowed {
        /// Seconds of continuous mining required.
        seconds: f32,
    },
    /// The tool tier is too low for this block.
    TooHard {
        /// Tier the block needs.
        required_tier: u8,
    },
    /// The block cannot be broken at all.
    Unbreakable,
}

/// Computes the mining time for a block with a tool.
///
/// `break_time = hardness / (tool_speed * power_multiplier)`, per the design. Tool speed already
/// encodes the tier's multiplier; `power_multiplier` is the drill's power state (1.0 = full).
#[must_use]
pub fn break_time(hardness: f32, tool_speed: f32, power_multiplier: f32) -> f32 {
    let denominator = tool_speed * power_multiplier;
    if denominator <= 0.0 || hardness <= 0.0 {
        return 0.0;
    }
    hardness / denominator
}

/// Checks whether `tool` can mine a block with the given stats.
#[must_use]
pub fn check_mine(
    hardness: f32,
    min_tier: u8,
    unbreakable: bool,
    tool: Option<&ToolDef>,
    power_multiplier: f32,
) -> MineCheck {
    if unbreakable {
        return MineCheck::Unbreakable;
    }
    let Some(tool) = tool else {
        return MineCheck::TooHard { required_tier: min_tier };
    };
    if tool.tier < min_tier {
        return MineCheck::TooHard { required_tier: min_tier };
    }
    MineCheck::Allowed { seconds: break_time(hardness, tool.speed, power_multiplier) }
}

/// The mining time for a block, or `None` when the tool cannot break it.
#[must_use]
pub fn mine_seconds(
    registry: &Registry,
    block: crate::voxel::BlockId,
    tool: ItemId,
) -> Option<f32> {
    let def = registry.block(block)?;
    let tool_def = registry.tool(tool);
    match check_mine(def.def.hardness, def.def.min_tier, def.def.unbreakable, tool_def, 1.0) {
        MineCheck::Allowed { seconds } => Some(seconds),
        MineCheck::TooHard { .. } | MineCheck::Unbreakable => None,
    }
}

/// Rounds a duration up to whole server ticks, as the design requires.
#[must_use]
pub fn ticks_for(seconds: f32, tick_seconds: f32) -> u32 {
    if tick_seconds <= 0.0 {
        return 0;
    }
    (seconds / tick_seconds).ceil().max(0.0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_dir;

    fn tool(tier: u8, speed: f32) -> ToolDef {
        ToolDef { tier, speed, area: (1, 1), energy: crate::content::EnergyKind::None }
    }

    #[test]
    fn break_time_follows_the_formula() {
        assert!((break_time(2.0, 1.0, 1.0) - 2.0).abs() < 1e-6);
        assert!((break_time(2.0, 2.0, 1.0) - 1.0).abs() < 1e-6);
        assert!((break_time(6.0, 12.0, 1.0) - 0.5).abs() < 1e-6);
        // A drained drill is slow but still works.
        assert!((break_time(2.0, 4.0, 0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn tier_gate_blocks_deep_layers() {
        // Granite needs tier 2; the starting steel pickaxe is tier 1.
        let steel = tool(1, 1.0);
        let hardened = tool(2, 1.6);
        assert_eq!(
            check_mine(2.5, 2, false, Some(&steel), 1.0),
            MineCheck::TooHard { required_tier: 2 }
        );
        assert!(matches!(
            check_mine(2.5, 2, false, Some(&hardened), 1.0),
            MineCheck::Allowed { .. }
        ));
    }

    #[test]
    fn bedrock_is_unbreakable_even_for_the_best_tool() {
        let nuclear = tool(5, 12.0);
        assert_eq!(check_mine(0.0, 0, true, Some(&nuclear), 1.0), MineCheck::Unbreakable);
    }

    #[test]
    fn no_tool_means_too_hard() {
        assert_eq!(check_mine(1.0, 1, false, None, 1.0), MineCheck::TooHard { required_tier: 1 });
    }

    #[test]
    fn ticks_round_up() {
        assert_eq!(ticks_for(0.5, 1.0 / 30.0), 15);
        assert_eq!(ticks_for(0.01, 1.0 / 30.0), 1);
        assert_eq!(ticks_for(0.0, 1.0 / 30.0), 0);
    }

    #[test]
    fn the_design_ladder_is_actually_climbable() {
        // The acceptance test for M1: with the starting steel pickaxe the player can dig
        // through layers 1 and 2 but not into layer 3 granite.
        let registry = load_dir(crate::content::default_content_dir()).expect("content").0;
        let steel = registry.item_id("steel_pickaxe").expect("steel pickaxe");
        let dirt = registry.block_id("dirt").expect("dirt");
        let stone = registry.block_id("stone").expect("stone");
        let granite = registry.block_id("granite").expect("granite");
        let pitchblende = registry.block_id("pitchblende").expect("pitchblende");

        assert!(mine_seconds(&registry, dirt, steel).is_some(), "layer 1 diggable");
        assert!(mine_seconds(&registry, stone, steel).is_some(), "layer 2 diggable");
        assert!(
            mine_seconds(&registry, granite, steel).is_none(),
            "layer 3 must block the steel pickaxe"
        );

        let hardened = registry.item_id("hardened_pickaxe").expect("hardened");
        assert!(mine_seconds(&registry, granite, hardened).is_some(), "tier 2 opens layer 3");
        assert!(mine_seconds(&registry, pitchblende, hardened).is_none(), "layer 5 needs tier 4");

        let tungsten = registry.item_id("tungsten_drill").expect("tungsten drill");
        assert!(mine_seconds(&registry, pitchblende, tungsten).is_some(), "tier 4 opens layer 5");
    }
}
