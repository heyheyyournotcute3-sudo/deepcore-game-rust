//! Registries: resolve string ids from the RON files into compact numeric ids.
//!
//! Everything that runs per tick uses numeric ids. String ids exist only in data files and in
//! the UI. Loading is strict: every dangling reference is an error with a readable message,
//! collected together so one run reports every problem rather than the first.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::defs::{
    BlockDef, ContentSet, FuelDef, HazardDef, ItemCategory, ItemDef, LayerDef, MachineDef, Pattern,
    RecipeDef, TechDef, ToolDef,
};

/// Numeric item id, index into [`Registry::items`].
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct ItemId(pub u16);

/// Numeric recipe id, index into [`Registry::recipes`].
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct RecipeId(pub u16);

/// Numeric machine id, index into [`Registry::machines`].
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct MachineId(pub u16);

/// Numeric tech node id, index into [`Registry::tech`].
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct TechId(pub u16);

/// Numeric layer id, index into [`Registry::layers`].
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct LayerId(pub u8);

/// A block with its references resolved.
#[derive(Debug, Clone)]
pub struct ResolvedBlock {
    /// Original definition.
    pub def: BlockDef,
    /// Resolved drop table.
    pub drops: Vec<(ItemId, u32)>,
}

/// An item with its references resolved.
#[derive(Debug, Clone)]
pub struct ResolvedItem {
    /// Original definition.
    pub def: ItemDef,
}

/// A machine with its references resolved.
#[derive(Debug, Clone)]
pub struct ResolvedMachine {
    /// Original definition.
    pub def: MachineDef,
    /// Resolved build cost.
    pub build_cost: Vec<(ItemId, u32)>,
    /// Resolved tech node.
    pub tech: TechId,
}

/// A recipe with its references resolved.
#[derive(Debug, Clone)]
pub struct ResolvedRecipe {
    /// Original definition.
    pub def: RecipeDef,
    /// Machine that runs it; `None` for hand crafting.
    pub machine: Option<MachineId>,
    /// Resolved inputs.
    pub inputs: Vec<(ItemId, u32)>,
    /// Resolved outputs.
    pub outputs: Vec<(ItemId, u32)>,
    /// Tech node that unlocks it.
    pub tech: TechId,
}

/// A tech node with its references resolved.
#[derive(Debug, Clone)]
pub struct ResolvedTech {
    /// Original definition.
    pub def: TechDef,
    /// Resolved parents.
    pub parents: Vec<TechId>,
    /// Resolved cost.
    pub cost: Vec<(ItemId, u32)>,
}

/// A layer with its references resolved.
#[derive(Debug, Clone)]
pub struct ResolvedLayer {
    /// Original definition.
    pub def: LayerDef,
    /// Resolved base block.
    pub base_block: crate::voxel::BlockId,
    /// Resolved fillers.
    pub fillers: Vec<(crate::voxel::BlockId, f32)>,
    /// Resolved ores.
    pub ores: Vec<(crate::voxel::BlockId, f32, (u32, u32))>,
}

/// A fuel with its item resolved.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedFuel {
    /// Item that burns.
    pub item: ItemId,
    /// Seconds of burn time per unit.
    pub burn_seconds: f32,
    /// Heat output.
    pub heat: f32,
}

/// A content validation problem.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{kind:?}: {message}")]
pub struct ContentError {
    /// Category of problem.
    pub kind: ErrorKind,
    /// Human readable detail.
    pub message: String,
}

/// Categories of content validation problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// An id referenced somewhere does not exist.
    UnknownReference,
    /// Two definitions share an id.
    DuplicateId,
    /// A recipe has no machine that can run it.
    UnrunnableRecipe,
    /// A tech node names a parent that does not exist.
    MissingParent,
    /// An ore has no use in any recipe or build cost.
    UnusedOre,
    /// A tech node is part of a dependency cycle.
    TechCycle,
    /// A structural problem, e.g. an empty palette or a negative number.
    Malformed,
}

impl ContentError {
    fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into() }
    }
}

/// The loaded and validated content database.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    /// Blocks, indexed by `BlockId`.
    pub blocks: Vec<ResolvedBlock>,
    /// Items, indexed by `ItemId`.
    pub items: Vec<ResolvedItem>,
    /// Machines, indexed by `MachineId`.
    pub machines: Vec<ResolvedMachine>,
    /// Recipes, indexed by `RecipeId`.
    pub recipes: Vec<ResolvedRecipe>,
    /// Tech nodes, indexed by `TechId`.
    pub tech: Vec<ResolvedTech>,
    /// Layers, indexed by `LayerId`.
    pub layers: Vec<ResolvedLayer>,
    /// Fuels by item.
    pub fuels: Vec<ResolvedFuel>,
    /// Global hazards.
    pub hazards: Vec<HazardDef>,
    /// String → numeric lookups.
    block_ids: HashMap<String, crate::voxel::BlockId>,
    /// String → numeric item lookup.
    item_ids: HashMap<String, ItemId>,
    /// String → numeric machine lookup.
    machine_ids: HashMap<String, MachineId>,
    /// String → numeric recipe lookup.
    recipe_ids: HashMap<String, RecipeId>,
    /// String → numeric tech lookup.
    tech_ids: HashMap<String, TechId>,
    /// String → numeric layer lookup.
    layer_ids: HashMap<String, LayerId>,
}

impl Registry {
    /// Builds a registry from raw content, validating every cross reference.
    ///
    /// # Errors
    /// Returns every validation problem found, not just the first.
    pub fn build(set: ContentSet) -> Result<Self, Vec<ContentError>> {
        let mut errors = Vec::new();
        let mut reg = Self::default();

        // --- blocks (index 0 must be air) ---
        if set.blocks.first().map(|b| b.id.as_str()) != Some("air") {
            errors.push(ContentError::new(
                ErrorKind::Malformed,
                "blocks.ron must start with the `air` block so BlockId(0) is air",
            ));
        }
        for (i, def) in set.blocks.iter().enumerate() {
            let id = crate::voxel::BlockId(i as u16);
            if reg.block_ids.insert(def.id.clone(), id).is_some() {
                errors.push(ContentError::new(
                    ErrorKind::DuplicateId,
                    format!("duplicate block id `{}`", def.id),
                ));
            }
            reg.blocks.push(ResolvedBlock { def: def.clone(), drops: Vec::new() });
        }

        // --- items ---
        for (i, def) in set.items.iter().enumerate() {
            let id = ItemId(i as u16);
            if reg.item_ids.insert(def.id.clone(), id).is_some() {
                errors.push(ContentError::new(
                    ErrorKind::DuplicateId,
                    format!("duplicate item id `{}`", def.id),
                ));
            }
            reg.items.push(ResolvedItem { def: def.clone() });
        }

        // --- machines ---
        for (i, def) in set.machines.iter().enumerate() {
            let id = MachineId(i as u16);
            if reg.machine_ids.insert(def.id.clone(), id).is_some() {
                errors.push(ContentError::new(
                    ErrorKind::DuplicateId,
                    format!("duplicate machine id `{}`", def.id),
                ));
            }
            reg.machines.push(ResolvedMachine {
                def: def.clone(),
                build_cost: Vec::new(),
                tech: TechId(0),
            });
        }

        // --- recipes ---
        for (i, def) in set.recipes.iter().enumerate() {
            let id = RecipeId(i as u16);
            if reg.recipe_ids.insert(def.id.clone(), id).is_some() {
                errors.push(ContentError::new(
                    ErrorKind::DuplicateId,
                    format!("duplicate recipe id `{}`", def.id),
                ));
            }
            reg.recipes.push(ResolvedRecipe {
                def: def.clone(),
                machine: None,
                inputs: Vec::new(),
                outputs: Vec::new(),
                tech: TechId(0),
            });
        }

        // --- tech ---
        for (i, def) in set.tech.iter().enumerate() {
            let id = TechId(i as u16);
            if reg.tech_ids.insert(def.id.clone(), id).is_some() {
                errors.push(ContentError::new(
                    ErrorKind::DuplicateId,
                    format!("duplicate tech id `{}`", def.id),
                ));
            }
            reg.tech.push(ResolvedTech { def: def.clone(), parents: Vec::new(), cost: Vec::new() });
        }

        // --- layers ---
        for (i, def) in set.layers.iter().enumerate() {
            let id = LayerId(i as u8);
            if reg.layer_ids.insert(def.id.clone(), id).is_some() {
                errors.push(ContentError::new(
                    ErrorKind::DuplicateId,
                    format!("duplicate layer id `{}`", def.id),
                ));
            }
            reg.layers.push(ResolvedLayer {
                def: def.clone(),
                base_block: crate::voxel::BlockId::AIR,
                fillers: Vec::new(),
                ores: Vec::new(),
            });
        }

        // --- resolve references ---
        reg.resolve_fuels(&set.fuels, &mut errors);
        reg.resolve_blocks(&mut errors);
        reg.resolve_items(&mut errors);
        reg.resolve_machines(&mut errors);
        reg.resolve_recipes(&mut errors);
        reg.resolve_tech(&mut errors);
        reg.resolve_layers(&mut errors);

        if errors.is_empty() {
            reg.validate_semantics(&mut errors);
        }
        if errors.is_empty() { Ok(reg) } else { Err(errors) }
    }

    /// Resolves the fuel table from the raw definitions.
    fn resolve_fuels(&mut self, fuels: &[FuelDef], errors: &mut Vec<ContentError>) {
        for fuel in fuels {
            match self.item_ids.get(&fuel.item) {
                Some(&item) => self.fuels.push(ResolvedFuel {
                    item,
                    burn_seconds: fuel.burn_seconds,
                    heat: fuel.heat,
                }),
                None => errors.push(ContentError::new(
                    ErrorKind::UnknownReference,
                    format!("fuel references unknown item `{}`", fuel.item),
                )),
            }
        }
    }

    fn resolve_blocks(&mut self, errors: &mut Vec<ContentError>) {
        for block in &mut self.blocks {
            let mut drops = Vec::new();
            for (item, count) in &block.def.drops {
                match self.item_ids.get(item) {
                    Some(&id) => drops.push((id, *count)),
                    None => errors.push(ContentError::new(
                        ErrorKind::UnknownReference,
                        format!("block `{}` drops unknown item `{item}`", block.def.id),
                    )),
                }
            }
            block.drops = drops;
        }
    }

    fn resolve_items(&mut self, errors: &mut Vec<ContentError>) {
        for item in &self.items {
            if item.def.stack == 0 {
                errors.push(ContentError::new(
                    ErrorKind::Malformed,
                    format!("item `{}` has a stack size of 0", item.def.id),
                ));
            }
            if let ItemCategory::Tool(tool) = &item.def.category
                && tool.tier == 0
            {
                errors.push(ContentError::new(
                    ErrorKind::Malformed,
                    format!("tool `{}` has tier 0; tiers start at 1", item.def.id),
                ));
            }
        }
    }

    fn resolve_machines(&mut self, errors: &mut Vec<ContentError>) {
        for machine in &mut self.machines {
            let mut cost = Vec::new();
            for (item, count) in &machine.def.build_cost {
                match self.item_ids.get(item) {
                    Some(&id) => cost.push((id, *count)),
                    None => errors.push(ContentError::new(
                        ErrorKind::UnknownReference,
                        format!("machine `{}` costs unknown item `{item}`", machine.def.id),
                    )),
                }
            }
            machine.build_cost = cost;
            match self.tech_ids.get(&machine.def.tech) {
                Some(&id) => machine.tech = id,
                None => errors.push(ContentError::new(
                    ErrorKind::UnknownReference,
                    format!(
                        "machine `{}` requires unknown tech `{}`",
                        machine.def.id, machine.def.tech
                    ),
                )),
            }
        }
    }

    fn resolve_recipes(&mut self, errors: &mut Vec<ContentError>) {
        for recipe in &mut self.recipes {
            if recipe.def.machine == "hand" {
                recipe.machine = None;
            } else {
                match self.machine_ids.get(&recipe.def.machine) {
                    Some(&id) => recipe.machine = Some(id),
                    None => errors.push(ContentError::new(
                        ErrorKind::UnrunnableRecipe,
                        format!(
                            "recipe `{}` names unknown machine `{}`",
                            recipe.def.id, recipe.def.machine
                        ),
                    )),
                }
            }
            let mut inputs = Vec::new();
            for (item, count) in &recipe.def.inputs {
                match self.item_ids.get(item) {
                    Some(&id) => inputs.push((id, *count)),
                    None => errors.push(ContentError::new(
                        ErrorKind::UnknownReference,
                        format!("recipe `{}` input unknown item `{item}`", recipe.def.id),
                    )),
                }
            }
            let mut outputs = Vec::new();
            for (item, count) in &recipe.def.outputs {
                match self.item_ids.get(item) {
                    Some(&id) => outputs.push((id, *count)),
                    None => errors.push(ContentError::new(
                        ErrorKind::UnknownReference,
                        format!("recipe `{}` output unknown item `{item}`", recipe.def.id),
                    )),
                }
            }
            recipe.inputs = inputs;
            recipe.outputs = outputs;
            match self.tech_ids.get(&recipe.def.tech) {
                Some(&id) => recipe.tech = id,
                None => errors.push(ContentError::new(
                    ErrorKind::UnknownReference,
                    format!(
                        "recipe `{}` requires unknown tech `{}`",
                        recipe.def.id, recipe.def.tech
                    ),
                )),
            }
            if recipe.inputs.is_empty() && recipe.outputs.is_empty() {
                errors.push(ContentError::new(
                    ErrorKind::Malformed,
                    format!("recipe `{}` has no inputs and no outputs", recipe.def.id),
                ));
            }
        }
    }

    fn resolve_tech(&mut self, errors: &mut Vec<ContentError>) {
        for tech in &mut self.tech {
            let mut parents = Vec::new();
            for parent in &tech.def.parents {
                match self.tech_ids.get(parent) {
                    Some(&id) => parents.push(id),
                    None => errors.push(ContentError::new(
                        ErrorKind::MissingParent,
                        format!("tech `{}` names unknown parent `{parent}`", tech.def.id),
                    )),
                }
            }
            tech.parents = parents;
            let mut cost = Vec::new();
            for (item, count) in &tech.def.cost {
                match self.item_ids.get(item) {
                    Some(&id) => cost.push((id, *count)),
                    None => errors.push(ContentError::new(
                        ErrorKind::UnknownReference,
                        format!("tech `{}` costs unknown item `{item}`", tech.def.id),
                    )),
                }
            }
            tech.cost = cost;
        }
    }

    fn resolve_layers(&mut self, errors: &mut Vec<ContentError>) {
        for layer in &mut self.layers {
            match self.block_ids.get(&layer.def.base_block) {
                Some(&id) => layer.base_block = id,
                None => errors.push(ContentError::new(
                    ErrorKind::UnknownReference,
                    format!(
                        "layer `{}` uses unknown base block `{}`",
                        layer.def.id, layer.def.base_block
                    ),
                )),
            }
            let mut fillers = Vec::new();
            for (block, weight) in &layer.def.fillers {
                match self.block_ids.get(block) {
                    Some(&id) => fillers.push((id, *weight)),
                    None => errors.push(ContentError::new(
                        ErrorKind::UnknownReference,
                        format!("layer `{}` uses unknown filler `{block}`", layer.def.id),
                    )),
                }
            }
            layer.fillers = fillers;
            let mut ores = Vec::new();
            for ore in &layer.def.ores {
                match self.block_ids.get(&ore.block) {
                    Some(&id) => ores.push((id, ore.frequency, ore.vein_size)),
                    None => errors.push(ContentError::new(
                        ErrorKind::UnknownReference,
                        format!("layer `{}` has unknown ore block `{}`", layer.def.id, ore.block),
                    )),
                }
            }
            layer.ores = ores;
        }
    }

    fn validate_semantics(&self, errors: &mut Vec<ContentError>) {
        // Tech cycles: a cycle would make the tree unreachable. Colours are 0 = unvisited,
        // 1 = on the current DFS stack, 2 = finished; a back edge to a 1 is a cycle.
        let n = self.tech.len();
        let mut colour = vec![0u8; n];
        for root in 0..n {
            if colour[root] != 0 {
                continue;
            }
            let mut stack: Vec<(usize, usize)> = vec![(root, 0)];
            colour[root] = 1;
            while let Some((node, child)) = stack.pop() {
                if child < self.tech[node].parents.len() {
                    stack.push((node, child + 1));
                    let next = self.tech[node].parents[child].0 as usize;
                    match colour[next] {
                        0 => {
                            colour[next] = 1;
                            stack.push((next, 0));
                        }
                        1 => errors.push(ContentError::new(
                            ErrorKind::TechCycle,
                            format!(
                                "tech `{}` is part of a dependency cycle through `{}`",
                                self.tech[node].def.id, self.tech[next].def.id
                            ),
                        )),
                        _ => {}
                    }
                } else {
                    colour[node] = 2;
                }
            }
        }

        // Every ore block must have a use: it drops an item that some recipe consumes, or the
        // block itself is a build cost / tech cost.
        let mut used_items: Vec<ItemId> = Vec::new();
        for r in &self.recipes {
            used_items.extend(r.inputs.iter().map(|(i, _)| *i));
        }
        for m in &self.machines {
            used_items.extend(m.build_cost.iter().map(|(i, _)| *i));
        }
        for t in &self.tech {
            used_items.extend(t.cost.iter().map(|(i, _)| *i));
        }
        for block in &self.blocks {
            let is_ore = block.def.pattern == Pattern::OreFleck((0.0, 0.0, 0.0))
                || matches!(block.def.pattern, Pattern::OreFleck(_));
            if !is_ore {
                continue;
            }
            let has_use = block.drops.iter().any(|(item, _)| used_items.contains(item));
            if !has_use {
                errors.push(ContentError::new(
                    ErrorKind::UnusedOre,
                    format!(
                        "ore block `{}` drops nothing that any recipe, machine or tech node uses",
                        block.def.id
                    ),
                ));
            }
        }
    }

    // --- lookups ---

    /// Numeric id for a block name.
    #[must_use]
    pub fn block_id(&self, name: &str) -> Option<crate::voxel::BlockId> {
        self.block_ids.get(name).copied()
    }

    /// Numeric id for an item name.
    #[must_use]
    pub fn item_id(&self, name: &str) -> Option<ItemId> {
        self.item_ids.get(name).copied()
    }

    /// Numeric id for a machine name.
    #[must_use]
    pub fn machine_id(&self, name: &str) -> Option<MachineId> {
        self.machine_ids.get(name).copied()
    }

    /// Numeric id for a recipe name.
    #[must_use]
    pub fn recipe_id(&self, name: &str) -> Option<RecipeId> {
        self.recipe_ids.get(name).copied()
    }

    /// Numeric id for a tech name.
    #[must_use]
    pub fn tech_id(&self, name: &str) -> Option<TechId> {
        self.tech_ids.get(name).copied()
    }

    /// Numeric id for a layer name.
    #[must_use]
    pub fn layer_id(&self, name: &str) -> Option<LayerId> {
        self.layer_ids.get(name).copied()
    }

    /// Block definition by numeric id.
    #[must_use]
    pub fn block(&self, id: crate::voxel::BlockId) -> Option<&ResolvedBlock> {
        self.blocks.get(id.0 as usize)
    }

    /// Item definition by numeric id.
    #[must_use]
    pub fn item(&self, id: ItemId) -> Option<&ResolvedItem> {
        self.items.get(id.0 as usize)
    }

    /// Machine definition by numeric id.
    #[must_use]
    pub fn machine(&self, id: MachineId) -> Option<&ResolvedMachine> {
        self.machines.get(id.0 as usize)
    }

    /// Recipe definition by numeric id.
    #[must_use]
    pub fn recipe(&self, id: RecipeId) -> Option<&ResolvedRecipe> {
        self.recipes.get(id.0 as usize)
    }

    /// Tech definition by numeric id.
    #[must_use]
    pub fn tech(&self, id: TechId) -> Option<&ResolvedTech> {
        self.tech.get(id.0 as usize)
    }

    /// Layer definition by numeric id.
    #[must_use]
    pub fn layer(&self, id: LayerId) -> Option<&ResolvedLayer> {
        self.layers.get(id.0 as usize)
    }

    /// Display name for an item, falling back to a placeholder for unknown ids.
    #[must_use]
    pub fn item_name(&self, id: ItemId) -> &str {
        self.item(id).map_or("<unknown item>", |i| i.def.name.as_str())
    }

    /// Display name for a block.
    #[must_use]
    pub fn block_name(&self, id: crate::voxel::BlockId) -> &str {
        self.block(id).map_or("<unknown block>", |b| b.def.id.as_str())
    }

    /// Burn time in seconds for an item, or `None` if it is not a fuel.
    #[must_use]
    pub fn fuel_for(&self, item: ItemId) -> Option<ResolvedFuel> {
        self.fuels.iter().copied().find(|f| f.item == item)
    }

    /// Tool statistics for an item, or `None` if it is not a tool.
    #[must_use]
    pub fn tool(&self, item: ItemId) -> Option<&ToolDef> {
        match &self.item(item)?.def.category {
            ItemCategory::Tool(tool) => Some(tool),
            ItemCategory::Material
            | ItemCategory::Refined
            | ItemCategory::Machine
            | ItemCategory::Block
            | ItemCategory::Gear(_)
            | ItemCategory::Module(_) => None,
        }
    }

    /// Recipes a machine can run.
    #[must_use]
    pub fn recipes_for_machine(&self, machine: MachineId) -> Vec<RecipeId> {
        self.recipes
            .iter()
            .enumerate()
            .filter(|(_, r)| r.machine == Some(machine))
            .map(|(i, _)| RecipeId(i as u16))
            .collect()
    }

    /// Recipes craftable by hand.
    #[must_use]
    pub fn hand_recipes(&self) -> Vec<RecipeId> {
        self.recipes
            .iter()
            .enumerate()
            .filter(|(_, r)| r.machine.is_none())
            .map(|(i, _)| RecipeId(i as u16))
            .collect()
    }

    /// Tech nodes whose parents are all researched but which are not researched yet.
    #[must_use]
    pub fn available_tech(&self, researched: &[bool]) -> Vec<TechId> {
        self.tech
            .iter()
            .enumerate()
            .filter(|(i, t)| {
                !researched.get(*i).copied().unwrap_or(false)
                    && t.parents
                        .iter()
                        .all(|p| researched.get(p.0 as usize).copied().unwrap_or(false))
            })
            .map(|(i, _)| TechId(i as u16))
            .collect()
    }

    /// True when a recipe is unlocked by the researched set.
    #[must_use]
    pub fn recipe_unlocked(&self, recipe: RecipeId, researched: &[bool]) -> bool {
        self.recipe(recipe)
            .is_some_and(|r| researched.get(r.tech.0 as usize).copied().unwrap_or(false))
    }

    /// Root tech nodes (no parents).
    #[must_use]
    pub fn root_tech(&self) -> Vec<TechId> {
        self.tech
            .iter()
            .enumerate()
            .filter(|(_, t)| t.parents.is_empty())
            .map(|(i, _)| TechId(i as u16))
            .collect()
    }

    /// Recipes that consume an item as an input.
    #[must_use]
    pub fn recipes_for_item_as_input(&self, item: ItemId) -> Vec<RecipeId> {
        self.recipes
            .iter()
            .enumerate()
            .filter(|(_, r)| r.inputs.iter().any(|(i, _)| *i == item))
            .map(|(i, _)| RecipeId(i as u16))
            .collect()
    }

    /// Number of recipes that produce an item.
    #[must_use]
    pub fn producers_of(&self, item: ItemId) -> Vec<RecipeId> {
        self.recipes
            .iter()
            .enumerate()
            .filter(|(_, r)| r.outputs.iter().any(|(i, _)| *i == item))
            .map(|(i, _)| RecipeId(i as u16))
            .collect()
    }
}
