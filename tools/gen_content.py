#!/usr/bin/env python3
"""Generates assets/data/*.ron for Deepcore.

Content is authored here (one place, easy to balance) and emitted as RON so the game reads
it as pure data. Run:  python3 tools/gen_content.py
"""

import os
import json

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets", "data")
os.makedirs(ROOT, exist_ok=True)

# ---------------------------------------------------------------- blocks
# (id, hardness, min_tier, drops, color, pattern, layer_hint, emissive, radiation, flags)
blocks = []
def B(id, hardness, min_tier, drops, color, pattern, layer_hint=None, emissive=0.0,
      radiation=None, transparent=False, replaceable=False, unbreakable=False):
    blocks.append(dict(id=id, hardness=hardness, min_tier=min_tier, drops=drops, color=color,
                       pattern=pattern, layer_hint=layer_hint, emissive=emissive,
                       radiation=radiation, transparent=transparent, replaceable=replaceable,
                       unbreakable=unbreakable))

B("air", 0.0, 0, [], (0.0, 0.0, 0.0), "Flat", transparent=True, replaceable=True)
B("bedrock", 0.0, 99, [], (0.10, 0.10, 0.12), "Speckle", unbreakable=True)
B("water", 0.0, 0, [], (0.16, 0.34, 0.62), "Flat", transparent=True, replaceable=True)
B("methane", 0.0, 0, [], (0.30, 0.36, 0.22), "Flat", transparent=True, replaceable=True)

# --- Layer 1: Topsoil (0-16)
B("grass", 0.6, 1, [("dirt", 1)], (0.30, 0.52, 0.22), "Speckle", 1)
B("dirt", 0.5, 1, [("dirt", 1)], (0.42, 0.30, 0.20), "Speckle", 1)
B("gravel", 0.6, 1, [("gravel", 1)], (0.48, 0.46, 0.44), "Blotch", 1)
B("clay", 1.0, 1, [("clay", 2)], (0.62, 0.55, 0.48), "Blotch", 1)
B("sand", 0.4, 1, [("sand", 1)], (0.80, 0.74, 0.52), "Speckle", 1)
B("coal_seam", 1.2, 1, [("coal", 2)], (0.14, 0.14, 0.15), "OreFleck((0.05, 0.05, 0.06))", 1)
B("log", 1.0, 1, [("timber", 2)], (0.34, 0.24, 0.14), "Strata", 1)
B("leaves", 0.2, 1, [("leaves", 1)], (0.18, 0.42, 0.16), "Speckle", 1, transparent=True)

# --- Layer 2: Shale Beds (16-64)
B("stone", 1.8, 1, [("stone_cobble", 1)], (0.50, 0.50, 0.52), "Speckle", 2)
B("limestone", 1.6, 1, [("limestone", 1)], (0.74, 0.72, 0.66), "Strata", 2)
B("shale", 1.5, 1, [("shale", 1)], (0.34, 0.36, 0.38), "Strata", 2)
B("malachite", 2.0, 1, [("raw_copper", 1)], (0.30, 0.44, 0.34), "OreFleck((0.20, 0.85, 0.55))", 2)
B("cassiterite", 2.2, 1, [("raw_tin", 1)], (0.36, 0.32, 0.28), "OreFleck((0.72, 0.70, 0.66))", 2)
B("hematite", 2.4, 1, [("raw_iron", 1)], (0.52, 0.28, 0.24), "OreFleck((0.85, 0.45, 0.35))", 2)

# --- Layer 3: Granite Depths (64-160)
B("granite", 2.5, 2, [("granite_cobble", 1)], (0.55, 0.50, 0.48), "Speckle", 3)
B("galena", 3.0, 2, [("raw_lead", 1)], (0.42, 0.44, 0.50), "OreFleck((0.75, 0.78, 0.88))", 3)
B("sphalerite", 3.0, 2, [("raw_zinc", 1)], (0.36, 0.38, 0.34), "OreFleck((0.90, 0.80, 0.40))", 3)
B("sulfur_deposit", 2.2, 2, [("sulfur", 2)], (0.72, 0.66, 0.24), "OreFleck((0.95, 0.92, 0.30))", 3)
B("quartz", 2.8, 2, [("quartz", 1)], (0.86, 0.86, 0.90), "OreFleck((0.98, 0.98, 1.0))", 3)
B("bauxite", 3.0, 2, [("raw_bauxite", 1)], (0.62, 0.42, 0.32), "Blotch", 3)
B("rock_salt", 2.0, 2, [("salt", 2)], (0.92, 0.90, 0.88), "Speckle", 3)

# --- Layer 4: Basalt Furnace (160-288)
B("basalt", 3.5, 3, [("basalt_cobble", 1)], (0.22, 0.22, 0.26), "Strata", 4)
B("pentlandite", 4.0, 3, [("raw_nickel", 1)], (0.34, 0.36, 0.30), "OreFleck((0.70, 0.85, 0.55))", 4)
B("wolframite", 4.5, 3, [("raw_tungsten", 1)], (0.28, 0.26, 0.30), "OreFleck((0.55, 0.55, 0.65))", 4)
B("chromite", 4.0, 3, [("raw_chromium", 1)], (0.26, 0.24, 0.28), "OreFleck((0.80, 0.75, 0.90))", 4)
B("graphite_vein", 3.0, 3, [("graphite", 2)], (0.16, 0.16, 0.18), "OreFleck((0.35, 0.35, 0.38))", 4)
B("fluorite", 3.5, 3, [("fluorite", 1)], (0.40, 0.62, 0.78), "OreFleck((0.55, 0.90, 1.0))", 4)
B("zircon", 4.5, 3, [("raw_zircon", 1)], (0.48, 0.42, 0.38), "OreFleck((0.95, 0.70, 0.45))", 4)
B("gold_vein", 5.0, 3, [("raw_gold", 1)], (0.46, 0.38, 0.20), "OreFleck((1.0, 0.85, 0.30))", 4)

# --- Layer 5: Deep Crust (288-448)
B("gneiss", 5.0, 4, [("gneiss_cobble", 1)], (0.38, 0.36, 0.40), "Strata", 5)
B("obsidian", 6.0, 4, [("obsidian", 1)], (0.10, 0.09, 0.14), "Speckle", 5)
B("pitchblende", 6.0, 4, [("uranium_ore", 1)], (0.15, 0.18, 0.12), "OreFleck((0.40, 0.90, 0.30))", 5,
  emissive=0.3, radiation=(4, 0.5))
B("monazite", 5.5, 4, [("monazite_sand", 1)], (0.52, 0.46, 0.30), "OreFleck((0.85, 0.65, 0.35))", 5)

# --- Layer 6: Mantle Edge (448-512)
B("mantle_rock", 7.0, 5, [("mantle_shard", 1)], (0.30, 0.14, 0.12), "Blotch", 6)
B("dense_pitchblende", 8.0, 5, [("uranium_ore", 3)], (0.12, 0.22, 0.10), "OreFleck((0.45, 1.0, 0.35))", 6,
  emissive=0.5, radiation=(6, 1.2))
B("core_crystal", 9.0, 5, [("core_crystal", 1)], (0.55, 0.30, 0.85), "Glow", 6, emissive=0.8)

# --- Placeable functional blocks
B("camp_lamp", 0.5, 1, [("camp_lamp", 1)], (0.95, 0.85, 0.55), "Glow", 1, emissive=0.9)
B("oil_lamp", 0.5, 1, [("oil_lamp", 1)], (0.95, 0.78, 0.45), "Glow", 1, emissive=0.7)
B("electric_lamp", 0.6, 1, [("electric_lamp", 1)], (0.80, 0.90, 1.0), "Glow", 3, emissive=0.9)
B("ladder", 0.4, 1, [("ladder", 1)], (0.55, 0.40, 0.24), "Strata", 1)
B("timber_support", 0.8, 1, [("timber_support", 1)], (0.38, 0.28, 0.18), "Strata", 1)
B("steel_support", 2.0, 1, [("steel_support", 1)], (0.55, 0.57, 0.60), "Speckle", 3)
B("glass", 0.5, 1, [("glass", 1)], (0.82, 0.90, 0.95), "Flat", 1, transparent=True)
B("brick", 1.5, 1, [("brick", 1)], (0.60, 0.30, 0.24), "Speckle", 1)
B("stone_brick", 2.0, 1, [("stone_brick", 1)], (0.46, 0.46, 0.48), "Speckle", 2)
B("lead_block", 3.0, 2, [("lead_block", 1)], (0.36, 0.38, 0.44), "Speckle", 3)
B("shielding_block", 4.0, 3, [("shielding_block", 1)], (0.30, 0.32, 0.38), "Speckle", 4)
B("graphite_moderator_block", 2.5, 2, [("graphite_moderator_block", 1)], (0.20, 0.20, 0.22), "Speckle", 4)
B("reactor_casing_block", 3.5, 3, [("reactor_casing_block", 1)], (0.62, 0.64, 0.68), "Speckle", 4)
B("sealant", 1.2, 1, [("sealant", 1)], (0.44, 0.48, 0.42), "Blotch", 2)
B("concrete", 2.2, 2, [("concrete", 1)], (0.62, 0.62, 0.60), "Speckle", 2)

# ---------------------------------------------------------------- items
items = []
def I(id, name, stack, category, desc=""):
    items.append(dict(id=id, name=name, stack=stack, category=category, description=desc))

def TOOL(tier, speed, area=(1, 1), energy="None"):
    e = energy
    if isinstance(energy, tuple):
        e = "%s(%s)" % (energy[0], energy[1])
    return 'Tool((tier: %d, speed: %.1f, area: (%d, %d), energy: %s))' % (tier, speed, area[0], area[1], e)

def GEAR(rad=0.0, heat=0.0, gas=False, oxy=0.0, light=0.0):
    return 'Gear((radiation_resist: %.2f, heat_resist: %.2f, gas_proof: %s, oxygen_bonus: %.1f, light: %.1f))' % (
        rad, heat, "true" if gas else "false", oxy, light)

def MOD(speed=1.0, power=1.0):
    return 'Module((speed_mult: %.2f, power_mult: %.2f))' % (speed, power)

# raw materials
I("dirt", "Dirt", 64, "Material")
I("gravel", "Gravel", 64, "Material")
I("clay", "Clay", 64, "Material", "Fires into bricks in a kiln.")
I("sand", "Sand", 64, "Material", "Melts into glass.")
I("timber", "Timber", 64, "Material", "Chopped from surface trees.")
I("leaves", "Leaves", 64, "Material")
I("stone_cobble", "Cobblestone", 64, "Material")
I("limestone", "Limestone", 64, "Material", "Flux for the blast furnace.")
I("shale", "Shale", 64, "Material")
I("granite_cobble", "Granite Cobble", 64, "Material")
I("basalt_cobble", "Basalt Cobble", 64, "Material")
I("gneiss_cobble", "Gneiss Cobble", 64, "Material")
I("obsidian", "Obsidian", 64, "Material")
I("mantle_shard", "Mantle Shard", 64, "Material")
I("coal", "Coal", 64, "Material", "Burns for 80 seconds.")
I("raw_copper", "Raw Copper", 64, "Material")
I("raw_tin", "Raw Tin", 64, "Material")
I("raw_iron", "Raw Iron", 64, "Material")
I("raw_lead", "Raw Lead", 64, "Material")
I("raw_zinc", "Raw Zinc", 64, "Material")
I("sulfur", "Sulfur", 64, "Material")
I("quartz", "Quartz", 64, "Material")
I("raw_bauxite", "Raw Bauxite", 64, "Material")
I("salt", "Rock Salt", 64, "Material")
I("raw_nickel", "Raw Nickel", 64, "Material")
I("raw_tungsten", "Raw Tungsten", 64, "Material")
I("raw_chromium", "Raw Chromium", 64, "Material")
I("graphite", "Graphite", 64, "Material")
I("fluorite", "Fluorite", 64, "Material")
I("raw_zircon", "Raw Zircon", 64, "Material")
I("raw_gold", "Raw Gold", 64, "Material")
I("uranium_ore", "Uranium Ore", 64, "Material", "Radioactive. Handle with lead.")
I("monazite_sand", "Monazite Sand", 64, "Material")
I("core_crystal", "Core Crystal", 16, "Material", "A trophy from the Mantle Edge.")

# refined
I("copper_ingot", "Copper Ingot", 64, "Refined")
I("tin_ingot", "Tin Ingot", 64, "Refined")
I("iron_ingot", "Iron Ingot", 64, "Refined")
I("lead_ingot", "Lead Ingot", 64, "Refined")
I("zinc_ingot", "Zinc Ingot", 64, "Refined")
I("bronze_ingot", "Bronze Ingot", 64, "Refined")
I("steel_ingot", "Steel Ingot", 64, "Refined")
I("brass_ingot", "Brass Ingot", 64, "Refined")
I("nickel_ingot", "Nickel Ingot", 64, "Refined")
I("tungsten_ingot", "Tungsten Ingot", 64, "Refined")
I("chromium_ingot", "Chromium Ingot", 64, "Refined")
I("aluminium_ingot", "Aluminium Ingot", 64, "Refined")
I("gold_ingot", "Gold Ingot", 64, "Refined")
I("stainless_steel", "Stainless Steel", 64, "Refined")
I("tungsten_carbide", "Tungsten Carbide", 64, "Refined")
I("silicon", "Silicon", 64, "Refined")
I("cadmium", "Cadmium", 64, "Refined", "Zinc refining byproduct. Control rods.")
I("rare_earth_magnet", "Rare-Earth Magnet", 64, "Refined")
I("graphite_moderator", "Graphite Moderator", 64, "Refined")
I("fluorine_compound", "Fluorine Compound", 64, "Refined")
I("chlorine", "Chlorine", 64, "Refined")
I("lye", "Lye", 64, "Refined")
I("sulfuric_acid", "Sulfuric Acid", 64, "Refined")
I("brick", "Brick", 64, "Refined")
I("fire_brick", "Fire Brick", 64, "Refined")
I("glass", "Glass", 64, "Refined")
I("stone_brick", "Stone Brick", 64, "Refined")
I("concrete", "Concrete", 64, "Refined")
I("sealant", "Sealant", 64, "Refined")
I("sand_cast", "Sand Cast", 64, "Refined", "Casting mould for machine parts.")

I("crushed_iron", "Crushed Iron Ore", 64, "Refined")
I("crushed_copper", "Crushed Copper Ore", 64, "Refined")
I("crushed_tin", "Crushed Tin Ore", 64, "Refined")
I("crushed_lead", "Crushed Lead Ore", 64, "Refined")
I("crushed_zinc", "Crushed Zinc Ore", 64, "Refined")
I("washed_iron", "Washed Iron Ore", 64, "Refined")
I("washed_copper", "Washed Copper Ore", 64, "Refined")
I("washed_tin", "Washed Tin Ore", 64, "Refined")
I("water_bottle", "Water Bottle", 16, "Refined")

# parts
I("copper_wire", "Copper Wire", 64, "Refined")
I("copper_pipe", "Copper Pipe", 64, "Refined")
I("iron_plate", "Iron Plate", 64, "Refined")
I("steel_plate", "Steel Plate", 64, "Refined")
I("copper_plate", "Copper Plate", 64, "Refined")
I("lead_plate", "Lead Plate", 64, "Refined")
I("bronze_gear", "Bronze Gear", 64, "Refined")
I("steel_gear", "Steel Gear", 64, "Refined")
I("iron_rod", "Iron Rod", 64, "Refined")
I("steel_rod", "Steel Rod", 64, "Refined")
I("brick_furnace_kit", "Brick Furnace Kit", 8, "Refined")
I("bearing", "Precision Bearing", 64, "Refined")
I("turbine_blade", "Turbine Blade", 16, "Refined")
I("centrifuge_rotor", "Centrifuge Rotor", 8, "Refined")
I("stainless_rotor", "Stainless Rotor", 8, "Refined")
I("circuit_mk1", "Circuit Mk1", 64, "Refined")
I("circuit_mk2", "Circuit Mk2", 64, "Refined")
I("battery_pack", "Battery Pack", 16, "Refined")
I("shielding_plate", "Dense Shielding Plate", 64, "Refined")
I("reactor_casing", "Reactor Casing", 64, "Refined")
I("zirconium_tubing", "Zirconium Tubing", 64, "Refined")
I("control_rod", "Control Rod", 16, "Refined")
I("fuel_rod", "Fuel Rod", 16, "Refined", "Radioactive.")
I("compact_core_cell", "Compact Core Cell", 4, "Refined", "Powers the Nuclear Drill.")
I("spent_fuel_rod", "Spent Fuel Rod", 16, "Refined", "Radioactive waste.")
I("spent_core_cell", "Spent Core Cell", 4, "Refined", "Radioactive waste.")
I("radioactive_tailings", "Radioactive Tailings", 64, "Refined", "Radioactive waste.")
I("waste_cask", "Waste Cask", 16, "Refined")
I("yellowcake", "Yellowcake", 64, "Refined")
I("uranium_solution", "Uranium Solution", 64, "Refined")
I("uf6_canister", "Uranium Hexafluoride Canister", 16, "Refined")
I("enriched_canister", "Enriched Canister", 16, "Refined", "Highly radioactive.")
I("depleted_canister", "Depleted Canister", 16, "Refined")
I("crushed_uranium_ore", "Crushed Uranium Ore", 64, "Refined")

# tools
I("steel_pickaxe", "Steel Pickaxe", 1, TOOL(1, 1.0), "Starting tool.")
I("hardened_pickaxe", "Hardened Steel Pickaxe", 1, TOOL(2, 1.6))
I("steam_drill", "Steam Drill", 1, TOOL(3, 3.0, energy=("Steam", "capacity: 300")))
I("tungsten_drill", "Tungsten-Carbide Drill", 1, TOOL(4, 5.0, energy=("Battery", "capacity: 600")))
I("nuclear_drill", "Nuclear Drill", 1, TOOL(5, 12.0, area=(3, 3), energy=("CoreCell", "breaks: 2000")))
I("ore_scanner", "Ore Scanner", 1, TOOL(1, 1.0), "Reveals ore within 16 blocks.")
I("geiger_counter", "Geiger Counter", 1, GEAR(), "Clicks faster near radiation.")
I("headlamp_oil", "Oil Headlamp", 1, GEAR(light=12.0), "Burns oil for light.")
I("headlamp_mk2", "Headlamp Mk2", 1, GEAR(light=18.0), "Battery powered.")

# gear
I("cooling_suit", "Cooling Suit", 1, GEAR(heat=0.6))
I("respirator", "Respirator", 1, GEAR(gas=True, oxy=30.0))
I("hazmat_suit", "Lead-Lined Hazmat Suit", 1, GEAR(rad=0.8, heat=0.3, gas=True, oxy=30.0))

# modules
I("module_speed", "Speed Module", 8, MOD(1.5, 1.8), "+50% speed, +80% power.")
I("module_efficiency", "Efficiency Module", 8, MOD(1.0, 0.7), "-30% power.")

# placeable blocks as items
I("camp_lamp", "Camp Lamp", 64, "Block")
I("oil_lamp", "Oil Lamp", 64, "Block")
I("electric_lamp", "Electric Lamp", 64, "Block")
I("ladder", "Ladder", 64, "Block")
I("timber_support", "Timber Support", 64, "Block")
I("steel_support", "Steel Support", 64, "Block")
I("rail", "Rail", 64, "Block")
I("minecart", "Minecart", 1, "Machine", "Hauls ore along rails back to the Base.")
I("lead_block", "Lead Block", 64, "Block")
I("shielding_block", "Shielding Block", 64, "Block")
I("graphite_moderator_block", "Graphite Moderator Block", 64, "Block")
I("reactor_casing_block", "Reactor Casing Block", 64, "Block")

# every machine is also an item, so it can be carried in a hotbar and placed
MACHINE_NAMES = {}

machines = []
def M(id, name, size, energy, ins, outs, cost, tech, cascade=None, modules=0, multiblock=False):
    machines.append(dict(id=id, name=name, size=size, energy=energy, input_slots=ins,
                         output_slots=outs, build_cost=cost, tech=tech, cascade=cascade,
                         module_slots=modules, multiblock=multiblock))

M("workbench", "Workbench", (2, 1, 1), "None", 0, 0, [("timber", 12)], "hand_tools")
M("storage_crate", "Storage Crate", (1, 1, 1), "None", 27, 0, [("timber", 8)], "hand_tools")
M("research_desk", "Research Desk", (2, 1, 1), "None", 9, 0, [("timber", 16), ("stone_cobble", 8)], "hand_tools")
M("brick_kiln", "Brick Kiln", (2, 2, 2), "Fuel(slots: 1)", 2, 2, [("clay", 24), ("stone_cobble", 12)], "smelting")
M("stone_furnace", "Stone Furnace", (2, 2, 2), "Fuel(slots: 1)", 2, 2, [("stone_cobble", 32), ("clay", 8)], "smelting")
M("anvil", "Anvil", (1, 1, 1), "None", 2, 2, [("iron_ingot", 6), ("stone_cobble", 4)], "smelting")
M("blast_furnace", "Blast Furnace", (2, 3, 2), "Fuel(slots: 1)", 3, 2,
  [("fire_brick", 40), ("iron_plate", 12)], "steelmaking", multiblock=True)
M("hand_pump", "Hand Pump", (1, 1, 1), "None", 0, 0, [("copper_ingot", 6), ("bronze_gear", 2)], "steam_power")
M("steam_boiler", "Steam Boiler", (2, 2, 2), "Fuel(slots: 1)", 1, 0, [("copper_plate", 12), ("brick", 24)], "steam_power")
M("steam_engine", "Steam Engine", (2, 2, 1), "Steam(capacity: 400)", 0, 0, [("steel_plate", 8), ("bronze_gear", 6)], "steam_power")
M("crusher", "Crusher", (2, 2, 2), "Steam(capacity: 400)", 2, 2, [("steel_plate", 10), ("bronze_gear", 4)], "ore_processing")
M("ore_washer", "Ore Washer", (2, 1, 2), "Steam(capacity: 400)", 2, 2, [("steel_plate", 8), ("copper_pipe", 6)], "ore_processing")
M("steam_turbine", "Steam Turbine", (2, 2, 2), "Steam(capacity: 400)", 0, 0, [("steel_plate", 16), ("turbine_blade", 4)], "electricity")
M("generator", "Generator", (2, 2, 2), "Steam(capacity: 400)", 0, 0, [("copper_wire", 24), ("steel_plate", 12)], "electricity")
M("battery_bank", "Battery Bank", (2, 2, 2), "None", 0, 0, [("battery_pack", 8), ("lead_plate", 12)], "electricity")
M("arc_furnace", "Arc Furnace", (2, 2, 2), "Electric(kw: 60)", 2, 2, [("steel_plate", 12), ("graphite", 8)], "electricity", modules=2)
M("electrolysis_cell", "Electrolysis Cell", (2, 2, 2), "Electric(kw: 40)", 2, 3, [("copper_plate", 10), ("glass", 6)], "electricity", modules=2)
M("circuit_assembler", "Circuit Assembler", (2, 2, 2), "Electric(kw: 30)", 3, 2, [("steel_plate", 8), ("copper_wire", 16)], "electricity", modules=2)
M("charging_station", "Charging Station", (1, 1, 1), "Electric(kw: 10)", 1, 1, [("copper_wire", 12), ("battery_pack", 1)], "electricity")
M("vent_fan", "Vent Fan", (1, 1, 1), "Electric(kw: 15)", 0, 0, [("steel_plate", 4), ("bronze_gear", 2)], "electricity")
M("electric_pump", "Electric Pump", (1, 1, 1), "Electric(kw: 20)", 0, 0, [("copper_pipe", 6), ("copper_wire", 8)], "electricity")
M("chemical_plant", "Chemical Plant", (3, 2, 2), "Electric(kw: 80)", 3, 3, [("steel_plate", 16), ("glass", 10), ("copper_pipe", 12)], "chemistry", modules=2, multiblock=True)
M("alloy_furnace", "Alloy Furnace", (2, 2, 2), "Electric(kw: 100)", 3, 2, [("steel_plate", 14), ("fire_brick", 20)], "advanced_metallurgy", modules=2)
M("machine_shop", "Machine Shop", (3, 2, 2), "Electric(kw: 90)", 3, 3, [("steel_plate", 18), ("bearing", 4)], "advanced_metallurgy", modules=2, multiblock=True)
M("rare_earth_separator", "Rare-Earth Separator", (2, 2, 2), "Electric(kw: 120)", 2, 2, [("steel_plate", 10), ("rare_earth_magnet", 2)], "rare_earths", modules=2)
M("decon_shower", "Decon Shower", (1, 1, 2), "Electric(kw: 25)", 0, 0, [("steel_plate", 6), ("copper_pipe", 4)], "hazard_gear")
M("leaching_tank", "Leaching Tank", (2, 2, 2), "Electric(kw: 40)", 2, 2, [("lead_plate", 10), ("copper_pipe", 8), ("sealant", 6)], "uranium_milling", modules=2)
M("precipitator", "Precipitator", (2, 2, 2), "Electric(kw: 45)", 2, 2, [("lead_plate", 12), ("glass", 8)], "uranium_milling", modules=2)
M("conversion_plant", "Conversion Plant", (2, 2, 2), "Electric(kw: 70)", 2, 2, [("fluorine_compound", 4), ("steel_plate", 10), ("lead_plate", 8)], "conversion", modules=2)
M("centrifuge", "Centrifuge", (1, 3, 1), "Electric(kw: 150)", 1, 2,
  [("stainless_rotor", 2), ("rare_earth_magnet", 4), ("circuit_mk2", 2)], "enrichment",
  cascade=(4, 16), modules=2)
M("fuel_fabricator", "Fuel Fabricator", (2, 2, 2), "Electric(kw: 80)", 3, 2, [("steel_plate", 12), ("circuit_mk2", 3)], "fuel_fabrication", modules=2)
M("reactor", "Reactor", (5, 5, 5), "None", 6, 4,
  [("reactor_casing", 40), ("lead_plate", 30), ("graphite_moderator", 12)], "reactor_engineering", multiblock=True)
M("cooling_pool", "Cooling Pool", (3, 1, 3), "None", 2, 2, [("concrete", 12), ("lead_plate", 6)], "waste_handling")
M("waste_vault", "Waste Vault", (3, 3, 3), "None", 9, 0, [("lead_plate", 20), ("concrete", 20), ("shielding_plate", 8)], "waste_handling", multiblock=True)
M("mine_lift", "Mine Lift", (2, 2, 2), "Electric(kw: 30)", 0, 0,
  [("steel_plate", 8), ("steel_rod", 12), ("bronze_gear", 4)], "mine_transport", modules=1)
M("conveyor", "Conveyor", (1, 1, 1), "Electric(kw: 2)", 1, 1,
  [("steel_plate", 1), ("bronze_gear", 1)], "conveyor_logistics")
M("item_pipe", "Item Pipe", (1, 1, 1), "None", 1, 1,
  [("copper_pipe", 2)], "conveyor_logistics")
M("cable", "Cable", (1, 1, 1), "None", 0, 0, [("copper_wire", 4)], "power_grid")
M("pole", "Power Pole", (1, 3, 1), "None", 0, 0,
  [("steel_rod", 4), ("copper_wire", 2)], "power_grid")
M("scram_panel", "Scram Panel", (1, 2, 1), "Electric(kw: 5)", 0, 0,
  [("circuit_mk2", 2), ("steel_plate", 4), ("control_rod", 1)], "scram_system")
M("reactor_console", "Reactor Console", (2, 1, 1), "Electric(kw: 10)", 0, 0,
  [("circuit_mk2", 3), ("glass", 4), ("steel_plate", 6)], "reactor_console_tech")
M("coolant_pump", "Coolant Pump", (2, 2, 2), "Electric(kw: 60)", 1, 0,
  [("stainless_steel", 4), ("copper_pipe", 8), ("bearing", 2)], "coolant_loop", modules=1)
M("heat_exchanger", "Heat Exchanger", (2, 2, 2), "None", 1, 1,
  [("stainless_steel", 6), ("copper_pipe", 10)], "coolant_loop", modules=1)
M("nuclear_drill_charger", "Core Cell Charger", (1, 1, 1), "Electric(kw: 200)", 1, 1, [("circuit_mk2", 4), ("lead_plate", 6)], "reactor_engineering")

# machine items (category Machine)
for _m in machines:
    I(_m["id"], _m["name"], 8, "Machine", "Buildable structure.")

# ---------------------------------------------------------------- recipes
recipes = []
def R(id, machine, time, inputs, outputs, tech, power=0.0, steam=0.0):
    recipes.append(dict(id=id, machine=machine, time=time, inputs=inputs, outputs=outputs,
                        tech=tech, power_kw=power, steam_per_sec=steam))

# hand / workbench
R("hand_timber_planks", "hand", 1.0, [("timber", 2)], [("timber", 1)], "hand_tools")
R("hand_ladder", "workbench", 2.0, [("timber", 3)], [("ladder", 4)], "hand_tools")
R("hand_timber_support", "workbench", 2.0, [("timber", 4)], [("timber_support", 2)], "hand_tools")
R("hand_crate", "workbench", 3.0, [("timber", 8)], [("storage_crate", 1)], "hand_tools")
R("hand_oil_lamp", "workbench", 3.0, [("timber", 2), ("clay", 2)], [("oil_lamp", 1)], "hand_tools")
R("hand_workbench", "hand", 3.0, [("timber", 12)], [("workbench", 1)], "hand_tools")
R("hand_research_desk", "workbench", 4.0, [("timber", 16), ("stone_cobble", 8)], [("research_desk", 1)], "hand_tools")
R("hand_brick_raw", "workbench", 2.0, [("clay", 2), ("sand", 1)], [("brick", 2)], "hand_tools")
R("hand_sand_cast", "workbench", 2.0, [("sand", 4)], [("sand_cast", 1)], "hand_tools")
R("hand_stone_brick", "workbench", 2.0, [("stone_cobble", 4)], [("stone_brick", 4)], "hand_tools")
R("hand_sealant", "workbench", 3.0, [("gravel", 4), ("clay", 2)], [("sealant", 2)], "hand_tools")
R("hand_glass", "workbench", 3.0, [("sand", 2)], [("glass", 1)], "hand_tools")
R("hand_concrete", "workbench", 3.0, [("gravel", 4), ("sand", 2)], [("concrete", 2)], "hand_tools")

# brick kiln
R("kiln_brick", "brick_kiln", 4.0, [("clay", 2)], [("brick", 2)], "smelting")
R("kiln_fire_brick", "brick_kiln", 6.0, [("brick", 2), ("limestone", 1)], [("fire_brick", 2)], "smelting")
R("kiln_glass", "brick_kiln", 5.0, [("sand", 2)], [("glass", 2)], "smelting")

# stone furnace smelting
R("smelt_copper", "stone_furnace", 6.0, [("raw_copper", 1)], [("copper_ingot", 1)], "smelting")
R("smelt_tin", "stone_furnace", 6.0, [("raw_tin", 1)], [("tin_ingot", 1)], "smelting")
R("smelt_iron", "stone_furnace", 6.0, [("raw_iron", 1)], [("iron_ingot", 1)], "smelting")
R("smelt_lead", "stone_furnace", 7.0, [("raw_lead", 1)], [("lead_ingot", 1)], "smelting")
R("smelt_zinc", "stone_furnace", 7.0, [("raw_zinc", 1)], [("zinc_ingot", 1)], "smelting")
R("smelt_bronze", "stone_furnace", 8.0, [("copper_ingot", 2), ("tin_ingot", 1)], [("bronze_ingot", 2)], "smelting")
R("smelt_brass", "stone_furnace", 8.0, [("copper_ingot", 2), ("zinc_ingot", 1)], [("brass_ingot", 2)], "steam_power")

# anvil
R("anvil_iron_plate", "anvil", 3.0, [("iron_ingot", 2)], [("iron_plate", 2)], "smelting")
R("anvil_copper_plate", "anvil", 3.0, [("copper_ingot", 2)], [("copper_plate", 2)], "smelting")
R("anvil_lead_plate", "anvil", 3.0, [("lead_ingot", 2)], [("lead_plate", 2)], "smelting")
R("anvil_iron_rod", "anvil", 3.0, [("iron_ingot", 1)], [("iron_rod", 2)], "smelting")
R("anvil_bronze_gear", "anvil", 4.0, [("bronze_ingot", 2)], [("bronze_gear", 1)], "smelting")
R("anvil_copper_wire", "anvil", 2.0, [("copper_ingot", 1)], [("copper_wire", 4)], "steam_power")
R("anvil_copper_pipe", "anvil", 3.0, [("copper_plate", 2)], [("copper_pipe", 2)], "steam_power")
R("anvil_steel_plate", "anvil", 4.0, [("steel_ingot", 2)], [("steel_plate", 2)], "steelmaking")
R("anvil_steel_rod", "anvil", 3.0, [("steel_ingot", 1)], [("steel_rod", 2)], "steelmaking")
R("anvil_steel_gear", "anvil", 4.0, [("steel_ingot", 2)], [("steel_gear", 1)], "steelmaking")

# blast furnace
R("blast_steel", "blast_furnace", 12.0, [("iron_ingot", 2), ("coal", 2), ("limestone", 1)], [("steel_ingot", 1)], "steelmaking")
R("blast_fire_brick", "blast_furnace", 8.0, [("brick", 4), ("limestone", 2)], [("fire_brick", 4)], "steelmaking")

# tool crafting
R("craft_hardened_pickaxe", "anvil", 10.0, [("steel_ingot", 6), ("iron_rod", 2), ("bronze_gear", 2)], [("hardened_pickaxe", 1)], "steelmaking")
R("craft_steam_drill", "anvil", 14.0, [("steel_ingot", 8), ("brass_ingot", 4), ("copper_pipe", 4), ("bronze_gear", 4)], [("steam_drill", 1)], "steam_power")
R("craft_ore_scanner", "circuit_assembler", 12.0, [("circuit_mk1", 2), ("copper_wire", 8), ("glass", 2)], [("ore_scanner", 1)], "electricity")
R("craft_tungsten_drill", "machine_shop", 20.0, [("tungsten_carbide", 4), ("stainless_steel", 4), ("circuit_mk2", 2), ("bearing", 2)], [("tungsten_drill", 1)], "advanced_metallurgy")
R("craft_nuclear_drill", "machine_shop", 40.0, [("compact_core_cell", 1), ("tungsten_carbide", 6), ("stainless_steel", 8), ("rare_earth_magnet", 4), ("shielding_plate", 2)], [("nuclear_drill", 1)], "fuel_fabrication")
R("craft_headlamp_oil", "workbench", 4.0, [("timber", 1), ("copper_wire", 2), ("glass", 1)], [("headlamp_oil", 1)], "smelting")
R("craft_headlamp_mk2", "circuit_assembler", 10.0, [("circuit_mk1", 1), ("battery_pack", 1), ("glass", 2)], [("headlamp_mk2", 1)], "electricity")

# machine build recipes (machine items are placed from build cost directly; these let players
# rebuild a machine from its parts without re-gathering)
R("build_workbench", "workbench", 5.0, [("timber", 12)], [("workbench", 1)], "hand_tools")
R("build_stone_furnace", "workbench", 8.0, [("stone_cobble", 32), ("clay", 8)], [("stone_furnace", 1)], "smelting")
R("build_brick_kiln", "workbench", 8.0, [("clay", 24), ("stone_cobble", 12)], [("brick_kiln", 1)], "smelting")
R("build_anvil", "workbench", 8.0, [("iron_ingot", 6), ("stone_cobble", 4)], [("anvil", 1)], "smelting")
R("build_blast_furnace", "workbench", 12.0, [("fire_brick", 40), ("iron_plate", 12)], [("blast_furnace", 1)], "steelmaking")
R("build_steam_boiler", "workbench", 12.0, [("copper_plate", 12), ("brick", 24)], [("steam_boiler", 1)], "steam_power")
R("build_steam_engine", "workbench", 12.0, [("steel_plate", 8), ("bronze_gear", 6)], [("steam_engine", 1)], "steam_power")
R("build_hand_pump", "workbench", 8.0, [("copper_ingot", 6), ("bronze_gear", 2)], [("hand_pump", 1)], "steam_power")
R("build_crusher", "workbench", 12.0, [("steel_plate", 10), ("bronze_gear", 4)], [("crusher", 1)], "ore_processing")
R("build_ore_washer", "workbench", 12.0, [("steel_plate", 8), ("copper_pipe", 6)], [("ore_washer", 1)], "ore_processing")
R("build_generator", "workbench", 14.0, [("copper_wire", 24), ("steel_plate", 12)], [("generator", 1)], "electricity")
R("build_battery_bank", "workbench", 12.0, [("battery_pack", 8), ("lead_plate", 12)], [("battery_bank", 1)], "electricity")
R("build_arc_furnace", "workbench", 14.0, [("steel_plate", 12), ("graphite", 8)], [("arc_furnace", 1)], "electricity")
R("build_electrolysis_cell", "workbench", 14.0, [("copper_plate", 10), ("glass", 6)], [("electrolysis_cell", 1)], "electricity")
R("build_circuit_assembler", "workbench", 14.0, [("steel_plate", 8), ("copper_wire", 16)], [("circuit_assembler", 1)], "electricity")
R("build_vent_fan", "workbench", 8.0, [("steel_plate", 4), ("bronze_gear", 2)], [("vent_fan", 1)], "electricity")
R("build_electric_pump", "workbench", 8.0, [("copper_pipe", 6), ("copper_wire", 8)], [("electric_pump", 1)], "electricity")
R("build_chemical_plant", "workbench", 20.0, [("steel_plate", 16), ("glass", 10), ("copper_pipe", 12)], [("chemical_plant", 1)], "chemistry")
R("build_alloy_furnace", "workbench", 20.0, [("steel_plate", 14), ("fire_brick", 20)], [("alloy_furnace", 1)], "advanced_metallurgy")
R("build_machine_shop", "workbench", 22.0, [("steel_plate", 18), ("bearing", 4)], [("machine_shop", 1)], "advanced_metallurgy")
R("build_rare_earth_separator", "workbench", 20.0, [("steel_plate", 10), ("rare_earth_magnet", 2)], [("rare_earth_separator", 1)], "rare_earths")
R("build_mine_lift", "workbench", 16.0, [("steel_plate", 8), ("steel_rod", 12), ("bronze_gear", 4)], [("mine_lift", 1)], "mine_transport")
R("build_conveyor", "workbench", 4.0, [("steel_plate", 1), ("bronze_gear", 1)], [("conveyor", 1)], "conveyor_logistics")
R("build_item_pipe", "workbench", 3.0, [("copper_pipe", 2)], [("item_pipe", 1)], "conveyor_logistics")
R("build_cable", "workbench", 2.0, [("copper_wire", 4)], [("cable", 1)], "power_grid")
R("build_pole", "workbench", 5.0, [("steel_rod", 4), ("copper_wire", 2)], [("pole", 1)], "power_grid")
R("build_scram_panel", "workbench", 14.0, [("circuit_mk2", 2), ("steel_plate", 4), ("control_rod", 1)], [("scram_panel", 1)], "scram_system")
R("build_reactor_console", "workbench", 14.0, [("circuit_mk2", 3), ("glass", 4), ("steel_plate", 6)], [("reactor_console", 1)], "reactor_console_tech")
R("build_coolant_pump", "workbench", 16.0, [("stainless_steel", 4), ("copper_pipe", 8), ("bearing", 2)], [("coolant_pump", 1)], "coolant_loop")
R("build_heat_exchanger", "workbench", 16.0, [("stainless_steel", 6), ("copper_pipe", 10)], [("heat_exchanger", 1)], "coolant_loop")
R("build_rail", "workbench", 2.0, [("steel_rod", 2)], [("rail", 4)], "mine_transport")
R("build_minecart", "workbench", 6.0, [("iron_plate", 4), ("bronze_gear", 2)], [("minecart", 1)], "mine_transport")
R("build_decon_shower", "workbench", 10.0, [("steel_plate", 6), ("copper_pipe", 4)], [("decon_shower", 1)], "hazard_gear")

# steam era processing
R("crush_iron", "crusher", 5.0, [("raw_iron", 1)], [("crushed_iron", 1)], "ore_processing")
R("crush_copper", "crusher", 5.0, [("raw_copper", 1)], [("crushed_copper", 1)], "ore_processing")
R("crush_tin", "crusher", 5.0, [("raw_tin", 1)], [("crushed_tin", 1)], "ore_processing")
R("crush_lead", "crusher", 5.0, [("raw_lead", 1)], [("crushed_lead", 1)], "ore_processing")
R("crush_zinc", "crusher", 5.0, [("raw_zinc", 1)], [("crushed_zinc", 1)], "ore_processing")
R("crush_uranium", "crusher", 8.0, [("uranium_ore", 1)], [("crushed_uranium_ore", 1)], "uranium_milling")
R("wash_iron", "ore_washer", 5.0, [("crushed_iron", 1)], [("washed_iron", 1)], "ore_processing")
R("wash_copper", "ore_washer", 5.0, [("crushed_copper", 1)], [("washed_copper", 1)], "ore_processing")
R("wash_tin", "ore_washer", 5.0, [("crushed_tin", 1)], [("washed_tin", 1)], "ore_processing")
R("smelt_washed_iron", "stone_furnace", 8.0, [("washed_iron", 1)], [("iron_ingot", 2)], "ore_processing")
R("smelt_washed_copper", "stone_furnace", 8.0, [("washed_copper", 1)], [("copper_ingot", 2)], "ore_processing")
R("smelt_washed_tin", "stone_furnace", 8.0, [("washed_tin", 1)], [("tin_ingot", 2)], "ore_processing")

# electric era processing
R("arc_silicon", "arc_furnace", 10.0, [("quartz", 2), ("coal", 1)], [("silicon", 1)], "electricity", power=60)
R("arc_steel", "arc_furnace", 6.0, [("raw_iron", 2), ("coal", 1)], [("steel_ingot", 2)], "electricity", power=60)
R("arc_gold", "arc_furnace", 8.0, [("raw_gold", 1), ("sulfuric_acid", 1)], [("gold_ingot", 1)], "electricity", power=60)
R("arc_nickel", "arc_furnace", 8.0, [("raw_nickel", 1)], [("nickel_ingot", 1)], "electricity", power=60)
R("electro_aluminium", "electrolysis_cell", 12.0, [("raw_bauxite", 2)], [("aluminium_ingot", 1)], "electricity", power=40)
R("electro_chlorine", "electrolysis_cell", 10.0, [("salt", 2)], [("chlorine", 1), ("lye", 1)], "chemistry", power=40)
R("circuit_mk1", "circuit_assembler", 8.0, [("silicon", 2), ("copper_wire", 4), ("glass", 1)], [("circuit_mk1", 1)], "electricity", power=30)
R("circuit_mk2", "circuit_assembler", 14.0, [("circuit_mk1", 2), ("gold_ingot", 1), ("quartz", 2)], [("circuit_mk2", 1)], "electricity", power=30)
R("battery_pack", "circuit_assembler", 12.0, [("lead_plate", 4), ("sulfuric_acid", 2), ("copper_plate", 2)], [("battery_pack", 1)], "electricity", power=30)

# chemistry
R("chem_sulfuric_acid", "chemical_plant", 10.0, [("sulfur", 2), ("water_bottle", 1)], [("sulfuric_acid", 2)], "chemistry", power=80)
R("chem_fluorine", "chemical_plant", 12.0, [("fluorite", 2), ("sulfuric_acid", 1)], [("fluorine_compound", 1)], "chemistry", power=80)
R("chem_water_bottle", "hand_pump", 2.0, [], [("water_bottle", 1)], "chemistry")
R("chem_lye", "chemical_plant", 8.0, [("salt", 2), ("water_bottle", 1)], [("lye", 2)], "chemistry", power=80)

# advanced metallurgy
R("alloy_stainless", "alloy_furnace", 16.0, [("steel_ingot", 4), ("nickel_ingot", 2), ("chromium_ingot", 1)], [("stainless_steel", 3)], "advanced_metallurgy", power=100)
R("alloy_tungsten_carbide", "alloy_furnace", 20.0, [("tungsten_ingot", 2), ("graphite", 1)], [("tungsten_carbide", 1)], "advanced_metallurgy", power=100)
R("alloy_chromium", "alloy_furnace", 12.0, [("raw_chromium", 2), ("aluminium_ingot", 1)], [("chromium_ingot", 1)], "advanced_metallurgy", power=100)
R("alloy_tungsten", "alloy_furnace", 14.0, [("raw_tungsten", 2), ("coal", 2)], [("tungsten_ingot", 1)], "advanced_metallurgy", power=100)
R("shop_bearing", "machine_shop", 8.0, [("steel_ingot", 1), ("bronze_gear", 1)], [("bearing", 2)], "advanced_metallurgy", power=90)
R("shop_turbine_blade", "machine_shop", 14.0, [("stainless_steel", 2), ("steel_plate", 2)], [("turbine_blade", 1)], "advanced_metallurgy", power=90)
R("shop_centrifuge_rotor", "machine_shop", 16.0, [("stainless_steel", 2), ("bearing", 2)], [("centrifuge_rotor", 1)], "enrichment", power=90)
R("shop_stainless_rotor", "machine_shop", 18.0, [("centrifuge_rotor", 1), ("rare_earth_magnet", 2)], [("stainless_rotor", 1)], "enrichment", power=90)
R("shop_zirconium_tubing", "machine_shop", 12.0, [("raw_zircon", 2), ("steel_rod", 1)], [("zirconium_tubing", 2)], "fuel_fabrication", power=90)
R("shop_shielding_plate", "machine_shop", 12.0, [("lead_plate", 4), ("concrete", 2)], [("shielding_plate", 1)], "hazard_gear", power=90)
R("shop_reactor_casing", "machine_shop", 10.0, [("stainless_steel", 2), ("steel_plate", 2)], [("reactor_casing", 1)], "reactor_engineering", power=90)
R("shop_depleted_shield", "machine_shop", 10.0, [("depleted_canister", 1), ("lead_plate", 2)], [("shielding_plate", 2)], "waste_handling", power=90)

# rare earths and cadmium
R("sep_monazite", "rare_earth_separator", 20.0, [("monazite_sand", 4), ("sulfuric_acid", 2)], [("rare_earth_magnet", 1)], "rare_earths", power=120)
R("sep_cadmium", "rare_earth_separator", 12.0, [("zinc_ingot", 4), ("sulfuric_acid", 1)], [("cadmium", 1)], "rare_earths", power=120)

# hazard gear
R("gear_cooling_suit", "machine_shop", 16.0, [("copper_pipe", 8), ("steel_plate", 4), ("sealant", 4)], [("cooling_suit", 1)], "hazard_gear", power=90)
R("gear_respirator", "machine_shop", 12.0, [("steel_plate", 2), ("sealant", 2), ("glass", 2)], [("respirator", 1)], "hazard_gear", power=90)
R("gear_hazmat", "machine_shop", 24.0, [("lead_plate", 12), ("stainless_steel", 4), ("respirator", 1), ("sealant", 6)], [("hazmat_suit", 1)], "hazard_gear", power=90)
R("gear_geiger", "circuit_assembler", 10.0, [("circuit_mk1", 1), ("copper_wire", 4), ("lead_plate", 1)], [("geiger_counter", 1)], "hazard_gear", power=30)
R("module_speed", "circuit_assembler", 12.0, [("circuit_mk1", 1), ("bronze_gear", 2)], [("module_speed", 1)], "electricity", power=30)
R("module_efficiency", "circuit_assembler", 12.0, [("circuit_mk1", 1), ("copper_wire", 4)], [("module_efficiency", 1)], "electricity", power=30)

# nuclear chain
R("leach_uranium", "leaching_tank", 20.0, [("crushed_uranium_ore", 2), ("sulfuric_acid", 1)], [("uranium_solution", 1), ("radioactive_tailings", 1)], "uranium_milling", power=40)
R("precipitate_yellowcake", "precipitator", 16.0, [("uranium_solution", 1), ("lye", 1)], [("yellowcake", 1)], "uranium_milling", power=45)
R("convert_uf6", "conversion_plant", 24.0, [("yellowcake", 1), ("fluorine_compound", 1)], [("uf6_canister", 1)], "conversion", power=70)
R("enrich_uranium", "centrifuge", 30.0, [("uf6_canister", 1)], [("enriched_canister", 1), ("depleted_canister", 1)], "enrichment", power=150)
R("fabricate_fuel_rod", "fuel_fabricator", 26.0, [("enriched_canister", 1), ("zirconium_tubing", 2)], [("fuel_rod", 1)], "fuel_fabrication", power=80)
R("fabricate_core_cell", "fuel_fabricator", 40.0, [("enriched_canister", 1), ("stainless_steel", 2), ("shielding_plate", 1)], [("compact_core_cell", 1)], "fuel_fabrication", power=80)
R("fabricate_control_rod", "machine_shop", 18.0, [("cadmium", 2), ("stainless_steel", 2)], [("control_rod", 1)], "reactor_engineering", power=90)
R("waste_cask", "machine_shop", 14.0, [("spent_fuel_rod", 1), ("concrete", 2), ("lead_plate", 1)], [("waste_cask", 1)], "waste_handling", power=90)
R("cool_spent_cell", "cooling_pool", 60.0, [("spent_core_cell", 1)], [("waste_cask", 1)], "waste_handling")
R("recharge_core_cell", "nuclear_drill_charger", 60.0, [("spent_core_cell", 1), ("enriched_canister", 1)], [("compact_core_cell", 1)], "reactor_engineering", power=200)
R("build_reactor", "workbench", 30.0, [("reactor_casing", 40), ("lead_plate", 30), ("graphite_moderator", 12)], [("reactor", 1)], "reactor_engineering")
R("build_centrifuge", "workbench", 16.0, [("stainless_rotor", 2), ("rare_earth_magnet", 4), ("circuit_mk2", 2)], [("centrifuge", 1)], "enrichment")
R("build_leaching_tank", "workbench", 16.0, [("lead_plate", 10), ("copper_pipe", 8), ("sealant", 6)], [("leaching_tank", 1)], "uranium_milling")
R("build_precipitator", "workbench", 16.0, [("lead_plate", 12), ("glass", 8)], [("precipitator", 1)], "uranium_milling")
R("build_conversion_plant", "workbench", 18.0, [("fluorine_compound", 4), ("steel_plate", 10), ("lead_plate", 8)], [("conversion_plant", 1)], "conversion")
R("build_fuel_fabricator", "workbench", 18.0, [("steel_plate", 12), ("circuit_mk2", 3)], [("fuel_fabricator", 1)], "fuel_fabrication")
R("build_cooling_pool", "workbench", 12.0, [("concrete", 12), ("lead_plate", 6)], [("cooling_pool", 1)], "waste_handling")
R("build_waste_vault", "workbench", 20.0, [("lead_plate", 20), ("concrete", 20), ("shielding_plate", 8)], [("waste_vault", 1)], "waste_handling")
R("build_core_cell_charger", "workbench", 14.0, [("circuit_mk2", 4), ("lead_plate", 6)], [("nuclear_drill_charger", 1)], "reactor_engineering")
R("build_graphite_moderator", "workbench", 8.0, [("graphite", 4), ("brick", 2)], [("graphite_moderator", 1)], "reactor_engineering")

# ---------------------------------------------------------------- tech tree
tech = []
def T(id, name, era, parents, cost, unlocks, desc=""):
    tech.append(dict(id=id, name=name, era=era, parents=parents, cost=cost, unlocks=unlocks, description=desc))

T("hand_tools", "Hand Tools", 0, [], [], ["workbench", "storage_crate", "research_desk", "steel_pickaxe"],
  "Chop timber, build a bench, and dig into the Shale Beds.")
T("smelting", "Smelting", 1, ["hand_tools"],
  [("raw_copper", 10), ("raw_iron", 10), ("clay", 20)],
  ["stone_furnace", "brick_kiln", "anvil"], "Fire clay into bricks and smelt your first ingots.")
T("steelmaking", "Steelmaking", 1, ["smelting"],
  [("iron_ingot", 20), ("coal", 20), ("limestone", 10)],
  ["blast_furnace", "hardened_pickaxe", "steel_plate"], "Blast iron with coal and flux into steel. Opens the Granite Depths.")
T("steam_power", "Steam Power", 2, ["steelmaking"],
  [("steel_plate", 20), ("copper_pipe", 10), ("bronze_gear", 5)],
  ["steam_boiler", "steam_engine", "hand_pump", "steam_drill"], "Boil water, drive an engine, and drill with steam.")
T("ore_processing", "Ore Processing", 2, ["steam_power"],
  [("steel_plate", 24), ("bronze_gear", 8), ("copper_pipe", 12)],
  ["crusher", "ore_washer"], "Crush and wash ore to double your ingot yield.")
T("logistics", "Logistics", 2, ["steam_power"],
  [("steel_rod", 20), ("timber", 40), ("bronze_gear", 10)],
  ["steel_rod", "timber_support", "steel_support"], "Rails, minecarts and a lift to haul ore home.")
T("mine_transport", "Mine Transport", 2, ["logistics"],
  [("steel_rod", 24), ("iron_plate", 12), ("bronze_gear", 6)],
  ["mine_lift", "minecart", "rail"], "Rails, minecarts and a powered lift to haul ore to the Base.")
T("conveyor_logistics", "Conveyor Logistics", 3, ["logistics"],
  [("steel_plate", 20), ("bronze_gear", 10), ("copper_pipe", 8)],
  ["conveyor", "item_pipe"], "Conveyors and item pipes link machines into automated lines.")
T("electricity", "Electricity", 3, ["ore_processing"],
  [("steel_plate", 30), ("copper_wire", 60), ("bronze_gear", 12)],
  ["steam_turbine", "generator", "battery_bank", "arc_furnace", "electrolysis_cell",
   "circuit_assembler", "charging_station", "electric_lamp", "module_speed"], "Turn steam into kilowatts and light the deep tunnels.")
T("power_grid", "Power Grid", 3, ["electricity"],
  [("copper_wire", 40), ("steel_rod", 20)],
  ["cable", "pole"], "Cables and poles carry kilowatts between machines.")
T("arc_smelting", "Arc Smelting", 3, ["electricity"],
  [("steel_plate", 16), ("graphite", 10), ("copper_wire", 20)],
  ["arc_furnace", "silicon"], "Electric arc furnaces smelt faster and reduce quartz to silicon.")
T("aluminium_refining", "Aluminium Refining", 3, ["arc_smelting"],
  [("raw_bauxite", 20), ("steel_plate", 10)],
  ["electrolysis_cell", "aluminium_ingot"], "Electrolysis turns bauxite into aluminium.")
T("electronics", "Electronics", 3, ["arc_smelting"],
  [("silicon", 10), ("copper_wire", 30), ("glass", 8)],
  ["circuit_assembler", "circuit_mk1"], "Assemble silicon and wire into circuits.")
T("battery_storage", "Battery Storage", 3, ["power_grid"],
  [("lead_plate", 16), ("sulfuric_acid", 6)],
  ["battery_bank", "battery_pack"], "Lead-acid packs buffer the grid and power portable tools.")
T("deep_safety", "Deep Safety", 3, ["electricity"],
  [("steel_plate", 20), ("copper_wire", 40), ("glass", 20)],
  ["vent_fan", "electric_pump", "ore_scanner", "headlamp_mk2"], "Vent gas, pump water, and scan for ore.")
T("chemistry", "Chemistry", 4, ["electricity"],
  [("steel_plate", 30), ("glass", 20), ("copper_pipe", 20)],
  ["chemical_plant", "sulfuric_acid", "fluorine_compound"], "Acids and fluorine compounds for the nuclear chain.")
T("advanced_metallurgy", "Advanced Metallurgy", 4, ["chemistry"],
  [("steel_ingot", 40), ("nickel_ingot", 10), ("chromium_ingot", 6)],
  ["alloy_furnace", "machine_shop", "stainless_steel", "tungsten_carbide", "tungsten_drill"],
  "Stainless steel and tungsten carbide. Opens the Deep Crust.")
T("steel_alloys", "Steel Alloys", 4, ["advanced_metallurgy"],
  [("steel_ingot", 30), ("nickel_ingot", 8), ("chromium_ingot", 4)],
  ["stainless_steel", "alloy_furnace"], "Nickel and chromium turn steel into stainless.")
T("tungsten_tooling", "Tungsten Tooling", 4, ["advanced_metallurgy"],
  [("raw_tungsten", 12), ("graphite", 8)],
  ["tungsten_carbide", "tungsten_drill"], "Carbide bits cut the Deep Crust.")
T("chemical_feedstock", "Chemical Feedstock", 4, ["chemistry"],
  [("sulfur", 20), ("fluorite", 10), ("salt", 20)],
  ["sulfuric_acid", "fluorine_compound"], "Bulk acids and fluorine compounds for refining.")
T("hazard_gear", "Hazard Gear", 4, ["advanced_metallurgy"],
  [("steel_plate", 20), ("copper_pipe", 10), ("sealant", 10)],
  ["cooling_suit", "respirator", "hazmat_suit", "geiger_counter", "decon_shower", "shielding_plate"],
  "Survive heat, gas and radiation.")
T("rare_earths", "Rare Earths", 4, ["advanced_metallurgy"],
  [("monazite_sand", 20), ("sulfuric_acid", 10)],
  ["rare_earth_separator", "rare_earth_magnet", "cadmium"], "Separate monazite into magnets; stockpile cadmium.")
T("uranium_milling", "Uranium Milling", 5, ["chemistry", "hazard_gear"],
  [("sulfuric_acid", 20), ("lead_plate", 20), ("lye", 10)],
  ["leaching_tank", "precipitator", "yellowcake"], "Crush, leach and precipitate uranium ore.")
T("conversion", "Conversion", 5, ["uranium_milling", "rare_earths"],
  [("yellowcake", 10), ("fluorine_compound", 10)],
  ["conversion_plant", "uf6_canister"], "Convert yellowcake into uranium hexafluoride.")
T("enrichment", "Enrichment", 5, ["conversion"],
  [("stainless_steel", 20), ("rare_earth_magnet", 10), ("circuit_mk2", 6)],
  ["centrifuge", "stainless_rotor", "enriched_canister"], "A centrifuge cascade enriches UF6 into reactor fuel.")
T("fuel_fabrication", "Fuel Fabrication", 5, ["enrichment"],
  [("enriched_canister", 2), ("zirconium_tubing", 6)],
  ["fuel_fabricator", "fuel_rod", "compact_core_cell", "nuclear_drill"],
  "Fabricate fuel rods and the compact core cell that powers the Nuclear Drill.")
T("reactor_engineering", "Reactor Engineering", 5, ["fuel_fabrication", "advanced_metallurgy"],
  [("reactor_casing", 20), ("control_rod", 4), ("graphite_moderator", 10)],
  ["reactor", "nuclear_drill_charger"], "Build a stable reactor and a scram panel.")
T("reactor_console_tech", "Reactor Console", 5, ["reactor_engineering"],
  [("circuit_mk2", 4), ("glass", 10), ("steel_plate", 12)],
  ["reactor_console"], "A console to run the reactor and watch its core temperature.")
T("scram_system", "Scram System", 5, ["reactor_console_tech"],
  [("control_rod", 4), ("circuit_mk2", 3), ("steel_plate", 10)],
  ["scram_panel", "control_rod"], "An automatic scram panel prevents a meltdown.")
T("coolant_loop", "Coolant Loop", 5, ["reactor_engineering"],
  [("stainless_steel", 8), ("copper_pipe", 20), ("bearing", 4)],
  ["coolant_pump", "heat_exchanger"], "Pump coolant through the core and recover the heat.")
T("enrichment_cascade", "Enrichment Cascade", 5, ["enrichment"],
  [("stainless_rotor", 6), ("rare_earth_magnet", 12), ("circuit_mk2", 6)],
  ["centrifuge", "stainless_rotor"], "Longer cascades enrich faster; each block adds throughput.")
T("waste_handling", "Waste Handling", 5, ["reactor_engineering"],
  [("waste_cask", 4), ("lead_plate", 20)],
  ["cooling_pool", "waste_vault"], "Cool and vault spent fuel before it poisons the Base.")

# ---------------------------------------------------------------- layers
layers = []
def L(id, name, index, depth, base, fillers, ores, hazards, temp):
    layers.append(dict(id=id, name=name, index=index, depth=depth, base_block=base,
                       fillers=fillers, ores=ores, hazards=hazards, ambient_temp=temp))

L("topsoil", "Topsoil", 1, (0, 16), "dirt",
  [("gravel", 0.35), ("clay", 0.15), ("sand", 0.10)],
  [("coal_seam", 0.030, (6, 24)), ("clay", 0.020, (6, 20)), ("sand", 0.020, (6, 20))],
  [], 12.0)
L("shale_beds", "Shale Beds", 2, (16, 64), "stone",
  [("shale", 0.40), ("limestone", 0.25)],
  [("coal_seam", 0.035, (8, 32)), ("malachite", 0.030, (4, 18)), ("cassiterite", 0.018, (4, 12)),
   ("hematite", 0.030, (4, 20))],
  [], 15.0)
L("granite_depths", "Granite Depths", 3, (64, 160), "granite",
  [("stone", 0.20)],
  [("hematite", 0.030, (6, 22)), ("galena", 0.020, (4, 14)), ("sphalerite", 0.020, (4, 14)),
   ("sulfur_deposit", 0.018, (4, 16)), ("quartz", 0.018, (4, 12)), ("bauxite", 0.016, (6, 20)),
   ("rock_salt", 0.026, (6, 24))],
  ["Flooding((pocket_chance: 0.010, max_volume: 512))"], 18.0)
L("basalt_furnace", "Basalt Furnace", 4, (160, 288), "basalt",
  [("obsidian", 0.06)],
  [("pentlandite", 0.020, (4, 14)), ("wolframite", 0.010, (3, 10)), ("chromite", 0.018, (4, 14)),
   ("graphite_vein", 0.020, (4, 16)), ("fluorite", 0.010, (3, 10)), ("zircon", 0.009, (3, 10)),
   ("gold_vein", 0.008, (3, 10))],
  ["Gas((pocket_chance: 0.012, oxygen_per_sec: 6.0))",
   "Heat((extra_degrees: 25.0))"], 28.0)
L("deep_crust", "Deep Crust", 5, (288, 448), "gneiss",
  [("obsidian", 0.10)],
  [("pitchblende", 0.006, (3, 8)), ("monazite", 0.012, (3, 12))],
  ["Radiation((dose_per_sec: 0.4))", "CaveIn((max_span: 7))", "Heat((extra_degrees: 40.0))"], 45.0)
L("mantle_edge", "Mantle Edge", 6, (448, 512), "mantle_rock",
  [],
  [("dense_pitchblende", 0.014, (4, 12)), ("core_crystal", 0.004, (1, 3))],
  ["Radiation((dose_per_sec: 1.2))", "Heat((extra_degrees: 80.0))", "CaveIn((max_span: 5))"], 70.0)

# ---------------------------------------------------------------- fuels
fuels = [
    dict(item="coal", burn_seconds=80.0, heat=1.0),
    dict(item="timber", burn_seconds=20.0, heat=0.6),
    dict(item="graphite", burn_seconds=120.0, heat=1.4),
]

# ---------------------------------------------------------------- hazards (global)
hazards = [
    "Darkness",
    "Flooding((pocket_chance: 0.010, max_volume: 512))",
    "Gas((pocket_chance: 0.012, oxygen_per_sec: 6.0))",
    "Heat((extra_degrees: 25.0))",
    "CaveIn((max_span: 7))",
    "Radiation((dose_per_sec: 0.4))",
]

# ---------------------------------------------------------------- emit
def convert_hazard(h):
    """Turn `Flooding((a: 1))` into RON's `Flooding(a: 1)`."""
    if h in ("Darkness",):
        return h
    head, rest = h.split("((", 1)
    return "%s(%s" % (head, rest.replace("))", ")", 1))

def fmt_color(c):
    return "(%.3f, %.3f, %.3f)" % c

def fmt_drops(drops):
    return "[" + ", ".join('("%s", %d)' % (i, n) for i, n in drops) + "]"

out_blocks = []
for b in blocks:
    fields = []
    fields.append('    (id: "%s"' % b["id"])
    fields.append("    hardness: %.2f" % b["hardness"])
    fields.append("    min_tier: %d" % b["min_tier"])
    fields.append("    drops: %s" % fmt_drops(b["drops"]))
    fields.append("    color: %s" % fmt_color(b["color"]))
    fields.append("    pattern: %s" % b["pattern"])
    if b["layer_hint"] is not None:
        fields.append("    layer_hint: Some(%d)" % b["layer_hint"])
    if b["emissive"]:
        fields.append("    emissive: %.2f" % b["emissive"])
    if b["radiation"]:
        r = b["radiation"]
        fields.append("    radiation: Some((radius: %d, dose_per_sec: %.2f))" % (r[0], r[1]))
    if b["unbreakable"]:
        fields.append("    unbreakable: true")
    if b["transparent"]:
        fields.append("    transparent: true")
    if b["replaceable"]:
        fields.append("    replaceable: true")
    out_blocks.append(",\n".join(fields) + ")")
open(os.path.join(ROOT, "blocks.ron"), "w").write("[\n" + ",\n".join(out_blocks) + ",\n]\n")

out_items = []
for it in items:
    s = '    (id: "%s", name: "%s", stack: %d, category: %s' % (it["id"], it["name"], it["stack"], it["category"])
    if it["description"]:
        s += ', description: "%s"' % it["description"]
    out_items.append(s + ")")
open(os.path.join(ROOT, "items.ron"), "w").write("[\n" + ",\n".join(out_items) + ",\n]\n")

out_m = []
for m in machines:
    s = '    (id: "%s", name: "%s", size: (%d, %d, %d), energy: %s, input_slots: %d, output_slots: %d' % (
        m["id"], m["name"], m["size"][0], m["size"][1], m["size"][2], m["energy"],
        m["input_slots"], m["output_slots"])
    s += ", build_cost: %s, tech: \"%s\"" % (fmt_drops(m["build_cost"]), m["tech"])
    if m["cascade"]:
        s += ", cascade: Some((min: %d, max: %d))" % m["cascade"]
    if m["module_slots"]:
        s += ", module_slots: %d" % m["module_slots"]
    if m["multiblock"]:
        s += ", multiblock: true"
    out_m.append(s + ")")
open(os.path.join(ROOT, "machines.ron"), "w").write("[\n" + ",\n".join(out_m) + ",\n]\n")

out_r = []
for r in recipes:
    s = '    (id: "%s", machine: "%s", time: %.1f, inputs: %s, outputs: %s, tech: "%s"' % (
        r["id"], r["machine"], r["time"], fmt_drops(r["inputs"]), fmt_drops(r["outputs"]), r["tech"])
    if r.get("power"):
        s += ", power_kw: %.1f" % r["power"]
    if r.get("steam"):
        s += ", steam_per_sec: %.1f" % r["steam"]
    out_r.append(s + ")")
open(os.path.join(ROOT, "recipes.ron"), "w").write("[\n" + ",\n".join(out_r) + ",\n]\n")

out_t = []
for t in tech:
    out_t.append('    (id: "%s", name: "%s", era: %d, parents: [%s], cost: %s, unlocks: [%s], description: "%s")' % (
        t["id"], t["name"], t["era"], ", ".join('"%s"' % p for p in t["parents"]),
        fmt_drops(t["cost"]), ", ".join('"%s"' % u for u in t["unlocks"]), t["description"]))
open(os.path.join(ROOT, "tech_tree.ron"), "w").write("[\n" + ",\n".join(out_t) + ",\n]\n")

out_l = []
for l in layers:
    s = '    (id: "%s", name: "%s", index: %d, depth: (%d, %d), base_block: "%s"' % (
        l["id"], l["name"], l["index"], l["depth"][0], l["depth"][1], l["base_block"])
    s += ", fillers: [%s]" % ", ".join('("%s", %.2f)' % (f, w) for f, w in l["fillers"])
    s += ", ores: [%s]" % ", ".join(
        '(block: "%s", frequency: %.4f, vein_size: (%d, %d))' % (o[0], o[1], o[2][0], o[2][1])
        for o in l["ores"])
    s += ", hazards: [%s]" % ", ".join(convert_hazard(h) for h in l["hazards"])
    s += ", ambient_temp: %.1f)" % l["ambient_temp"]
    out_l.append(s)
open(os.path.join(ROOT, "layers.ron"), "w").write("[\n" + ",\n".join(out_l) + ",\n]\n")

out_f = ['    (item: "%s", burn_seconds: %.1f, heat: %.2f)' % (f["item"], f["burn_seconds"], f["heat"]) for f in fuels]
open(os.path.join(ROOT, "fuels.ron"), "w").write("[\n" + ",\n".join(out_f) + ",\n]\n")

open(os.path.join(ROOT, "hazards.ron"), "w").write("[\n" + ",\n".join("    " + convert_hazard(h) for h in hazards) + ",\n]\n")

print("blocks=%d items=%d machines=%d recipes=%d tech=%d layers=%d fuels=%d hazards=%d" % (
    len(blocks), len(items), len(machines), len(recipes), len(tech), len(layers), len(fuels), len(hazards)))
