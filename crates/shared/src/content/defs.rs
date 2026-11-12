//! Typed definitions of the RON content files.
//!
//! These structs are the exact shape of the files in `assets/data/`. They use string ids for
//! cross references; [`crate::content::Registry`] resolves those to numeric ids at load time
//! and rejects anything dangling.

use serde::{Deserialize, Serialize};

/// Texture pattern used by the procedural atlas generator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    /// Flat colour.
    Flat,
    /// Fine noise, for stone and dirt.
    Speckle,
    /// Coarse blotches, for gravel and ore host rock.
    Blotch,
    /// Bright flecks of the given colour over the base colour, for ore blocks.
    OreFleck((f32, f32, f32)),
    /// Horizontal strata lines, for shale and basalt.
    Strata,
    /// A soft glow, for uranium and lamps.
    Glow,
}

/// Definition of a block type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDef {
    /// Stable string id, e.g. `"granite"`.
    pub id: String,
    /// Seconds to break with a tier-1 tool.
    pub hardness: f32,
    /// Minimum tool tier required to break it at all.
    pub min_tier: u8,
    /// Items dropped when broken, as `(item, count)`.
    #[serde(default)]
    pub drops: Vec<(String, u32)>,
    /// Base colour, linear RGB in `0..1`.
    pub color: (f32, f32, f32),
    /// Procedural texture pattern.
    pub pattern: Pattern,
    /// Layer hint for world generation, purely informational.
    #[serde(default)]
    pub layer_hint: Option<u8>,
    /// Emissive strength in `0..1`; `0` means not emissive.
    #[serde(default)]
    pub emissive: f32,
    /// Radiation emitted when exposed to air.
    #[serde(default)]
    pub radiation: Option<RadiationDef>,
    /// True for blocks that cannot be broken (bedrock, world walls).
    #[serde(default)]
    pub unbreakable: bool,
    /// True for blocks that are transparent to light and meshing (air, water, glass).
    #[serde(default)]
    pub transparent: bool,
    /// True for blocks that are replaced automatically (air, water, gas).
    #[serde(default)]
    pub replaceable: bool,
}

/// Radiation source attached to a block.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RadiationDef {
    /// Radius in blocks.
    pub radius: u32,
    /// Dose per second at the source, falling off with distance.
    pub dose_per_sec: f32,
}

/// How a tool or machine gets its energy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EnergyKind {
    /// Nothing required.
    None,
    /// Fuel item burned in a slot.
    Fuel {
        /// Fuel slot count.
        slots: u8,
    },
    /// Steam drawn from a pipe network.
    Steam {
        /// Steam capacity in units.
        capacity: u32,
    },
    /// Electricity drawn from a cable network.
    Electric {
        /// Draw in kilowatts.
        kw: f32,
    },
    /// A portable battery pack.
    Battery {
        /// Stored energy in kJ.
        capacity: u32,
    },
    /// A drill core cell.
    CoreCell {
        /// Block breaks per cell.
        breaks: u32,
    },
}

/// Item category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemCategory {
    /// Raw ore or gathered material.
    Material,
    /// Smelted or processed material.
    Refined,
    /// A tool.
    Tool(ToolDef),
    /// A placeable machine.
    Machine,
    /// A placeable block.
    Block,
    /// Wearable hazard gear.
    Gear(GearDef),
    /// An upgrade module for machines.
    Module(ModuleDef),
}

/// Protective gear statistics. All resistances are fractions in `0..=1`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GearDef {
    /// Fraction of radiation dose blocked.
    #[serde(default)]
    pub radiation_resist: f32,
    /// Fraction of heat gain blocked.
    #[serde(default)]
    pub heat_resist: f32,
    /// True when the wearer can breathe methane pockets.
    #[serde(default)]
    pub gas_proof: bool,
    /// Extra oxygen capacity in seconds.
    #[serde(default)]
    pub oxygen_bonus: f32,
    /// Headlamp light radius in blocks; `0` for no light.
    #[serde(default)]
    pub light: f32,
}

/// Machine upgrade module statistics.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ModuleDef {
    /// Multiplier on machine speed.
    #[serde(default = "one")]
    pub speed_mult: f32,
    /// Multiplier on power draw.
    #[serde(default = "one")]
    pub power_mult: f32,
}

fn one() -> f32 {
    1.0
}

/// Tool statistics.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ToolDef {
    /// Tool tier, compared against [`BlockDef::min_tier`].
    pub tier: u8,
    /// Speed multiplier in the break-time formula.
    pub speed: f32,
    /// Mining area, `(width, height)` of the face broken per action.
    pub area: (u8, u8),
    /// How the tool is powered.
    pub energy: EnergyKind,
}

/// Definition of an item type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    /// Stable string id, e.g. `"steel_ingot"`.
    pub id: String,
    /// Human readable name.
    pub name: String,
    /// Maximum stack size.
    pub stack: u32,
    /// Category and category-specific data.
    pub category: ItemCategory,
    /// Optional flavour text shown in tooltips.
    #[serde(default)]
    pub description: String,
}

/// A machine or placeable structure definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineDef {
    /// Stable string id, e.g. `"blast_furnace"`.
    pub id: String,
    /// Human readable name.
    pub name: String,
    /// Footprint in blocks, `(x, y, z)`.
    pub size: (u8, u8, u8),
    /// Energy requirement.
    pub energy: EnergyKind,
    /// Number of input slots.
    pub input_slots: u8,
    /// Number of output slots.
    pub output_slots: u8,
    /// Items required to build it.
    pub build_cost: Vec<(String, u32)>,
    /// Tech node that unlocks it.
    pub tech: String,
    /// Cascade configuration, for centrifuges.
    #[serde(default)]
    pub cascade: Option<CascadeDef>,
    /// Upgrade module slots available.
    #[serde(default)]
    pub module_slots: u8,
    /// True for structures that occupy several blocks and are assembled as one unit.
    #[serde(default)]
    pub multiblock: bool,
}

/// Cascade constraints for centrifuge-style machines.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CascadeDef {
    /// Minimum useful cascade length.
    pub min: u8,
    /// Maximum cascade length.
    pub max: u8,
}

/// A crafting recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDef {
    /// Stable string id, e.g. `"smelt_iron"`.
    pub id: String,
    /// Machine that runs it, or `"hand"` for the workbench and the player's hands.
    pub machine: String,
    /// Seconds per craft.
    pub time: f32,
    /// Inputs consumed, as `(item, count)`.
    pub inputs: Vec<(String, u32)>,
    /// Outputs produced, as `(item, count)`.
    pub outputs: Vec<(String, u32)>,
    /// Tech node that unlocks it.
    pub tech: String,
    /// Electric draw in kW; `0` for non-electric machines.
    #[serde(default)]
    pub power_kw: f32,
    /// Steam draw per second; `0` for non-steam machines.
    #[serde(default)]
    pub steam_per_sec: f32,
}

/// A node in the shared tech tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechDef {
    /// Stable string id.
    pub id: String,
    /// Human readable name.
    pub name: String,
    /// Era this node belongs to, `0..=5`.
    pub era: u8,
    /// Parent nodes that must be researched first.
    #[serde(default)]
    pub parents: Vec<String>,
    /// Items delivered to the Research Desk to unlock it.
    pub cost: Vec<(String, u32)>,
    /// Machines, blocks or tools this node unlocks.
    pub unlocks: Vec<String>,
    /// One-line description shown in the tech tree UI.
    #[serde(default)]
    pub description: String,
}

/// An ore vein entry inside a layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OreEntry {
    /// Block id of the ore.
    pub block: String,
    /// Vein frequency, `0..1`.
    pub frequency: f32,
    /// Vein size range in blocks, `(min, max)`.
    pub vein_size: (u32, u32),
}

/// Hazard configuration for a layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HazardDef {
    /// No light below layer 1; the player needs a lamp or headlamp.
    Darkness,
    /// Water pockets that release and spread.
    Flooding {
        /// Chance per chunk of a water pocket.
        pocket_chance: f32,
        /// Maximum spread volume in blocks.
        max_volume: u32,
    },
    /// Methane pockets that drain oxygen.
    Gas {
        /// Chance per chunk of a gas pocket.
        pocket_chance: f32,
        /// Oxygen drained per second while inside.
        oxygen_per_sec: f32,
    },
    /// Ambient heat, rising with depth.
    Heat {
        /// Added degrees over the layer's ambient temperature at the bottom.
        extra_degrees: f32,
    },
    /// Unsupported ceiling spans that can collapse.
    CaveIn {
        /// Maximum unsupported span in blocks before collapse.
        max_span: u32,
    },
    /// Background radiation.
    Radiation {
        /// Background dose per second.
        dose_per_sec: f32,
    },
}

/// One of the six mine layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDef {
    /// Stable string id, e.g. `"granite_depths"`.
    pub id: String,
    /// Human readable name.
    pub name: String,
    /// Depth range below the surface, `(top, bottom)`.
    pub depth: (u32, u32),
    /// Base block id.
    pub base_block: String,
    /// Secondary filler blocks with their share of the volume.
    #[serde(default)]
    pub fillers: Vec<(String, f32)>,
    /// Ore veins in this layer.
    #[serde(default)]
    pub ores: Vec<OreEntry>,
    /// Hazards present in this layer.
    #[serde(default)]
    pub hazards: Vec<HazardDef>,
    /// Ambient temperature in degrees Celsius.
    #[serde(default)]
    pub ambient_temp: f32,
    /// Number of layers down (1-based), used by the HUD.
    pub index: u8,
}

/// A burnable fuel definition, referenced from `items.ron` ids.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelDef {
    /// Item id that burns.
    pub item: String,
    /// Seconds of burn time per unit.
    pub burn_seconds: f32,
    /// Heat output, used by boilers and furnaces.
    #[serde(default)]
    pub heat: f32,
}

/// The whole content database as loaded from disk.
#[derive(Debug, Clone, Default)]
pub struct ContentSet {
    /// Blocks.
    pub blocks: Vec<BlockDef>,
    /// Items.
    pub items: Vec<ItemDef>,
    /// Machines.
    pub machines: Vec<MachineDef>,
    /// Recipes.
    pub recipes: Vec<RecipeDef>,
    /// Tech nodes.
    pub tech: Vec<TechDef>,
    /// Layers.
    pub layers: Vec<LayerDef>,
    /// Fuels.
    pub fuels: Vec<FuelDef>,
    /// Hazards that are global rather than per-layer.
    pub hazards: Vec<HazardDef>,
}
