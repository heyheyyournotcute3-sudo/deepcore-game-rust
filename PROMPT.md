# Deepcore — Build Prompt

This file is the original build specification for the project, kept at the repository root as
Section 3 of the spec requires. It is the source of truth for scope: anything the milestones ask
for belongs in the game, and anything deferred must be listed in `PROGRESS.md`.

---

## 1. Role, mission and operating rules

You are a senior Rust game engineer and technical game designer. Your job is to build **Deepcore**,
a complete, playable, online co-op 3D voxel mining game, from an empty folder, entirely in Rust.
You own the code, the architecture, the game balance and the tests. Work autonomously. Ask the
human only when a decision is irreversible or truly ambiguous.

### The game in one paragraph

Players spawn at a surface camp above a huge procedurally generated mine. Everyone starts with a
**steel pickaxe** and nothing else. They dig down through block layers, collect ores, and haul them
back to build machines: furnaces, crushers, boilers, generators, refineries. Each machine unlocks
better tools and new materials, and those open deeper, harder, more dangerous layers. The final
goal is **nuclear power**: find uranium in the deepest layer, then build the full refining chain
(crush, leach, convert, enrich, fabricate fuel). That chain powers a **Nuclear Drill** that mines
whole tunnels in seconds, plus a reactor that powers the base. Every step on that ladder must be a
structure the players physically build and operate in the world.

### How you work

1. **Read the whole prompt before writing any code.** Then write `docs/PLAN.md`: your understanding
   of the game, the crate layout, and the milestone list from Section 8, in your own words.
2. **Keep a `PROGRESS.md`.** After every milestone, record what was built, what was deferred, known
   bugs, and the exact commands to run the server and client. A future session, possibly a
   different model, must be able to continue from this file alone.
3. **Work milestone by milestone, in order.** Do not start a milestone until the previous one meets
   its acceptance criteria. The game must compile and run at the end of every milestone.
4. **Compile constantly.** Run `cargo check --workspace` after every meaningful change. Run
   `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` before closing a
   milestone. Fix errors immediately and never leave the tree broken.
5. **Verify library APIs, never guess them.** The Bevy ecosystem breaks its API every release.
   Before using a crate, check its newest version on crates.io and read its docs.rs page and
   examples for that exact version. If your memory of an API disagrees with the docs, the docs win.
   Pin exact versions in the workspace `Cargo.toml` once chosen.
6. **No stubs, no fake features.** Do not write `todo!()`, empty systems, or "implement later"
   comments in code you call finished. If something must be deferred, leave it out entirely and
   list it in `PROGRESS.md`.
7. **Data over code.** Blocks, items, recipes, machines and the tech tree live in RON data files
   (Section 6). Adding a new ore or recipe must never require touching Rust code.
8. **The server is the source of truth.** Every gameplay rule runs on the server: mining, crafting,
   inventories, machines, power and hazards. The client predicts only its own movement and sends
   intents.
9. **Keep files small and named by purpose.** Aim for under 400 lines per file and one Bevy plugin
   per feature (`MiningPlugin`, `InventoryPlugin`, `PowerGridPlugin`, ...).
10. **No external art required.** Generate placeholder textures procedurally (Section 3) so the
    game runs from a fresh clone with zero downloaded assets.

### What not to do

- Do not use a different language, a scripting layer, or a non-Rust engine.
- Do not use real-world chemistry or engineering detail beyond what is written here. Refining is a
  game abstraction: machines take items in, wait, and put items out.
- Do not add features outside this prompt (combat, PvP, monsters, microtransactions) unless a
  milestone asks for them.
- Do not rewrite working systems for style. Refactor only when a milestone needs it.

---

## 2. Tech stack

Use **Bevy** for both client and server. One engine and one ECS on both sides means the same
components, systems and data types run everywhere.

| Concern | Choice |
|---|---|
| Engine, ECS, rendering | `bevy` (latest stable that all deps support) |
| Headless server | `bevy` with `MinimalPlugins` |
| Replication | `bevy_replicon` |
| Transport | `bevy_replicon_renet` (renet, UDP) |
| Voxel storage and meshing | Own code in `shared`, optionally `block-mesh` |
| World generation noise | `noise` or `fastnoise-lite` |
| Player physics | Own AABB-vs-voxel collision |
| Item physics (optional) | `avian3d` |
| UI | `bevy_egui` |
| HUD | Bevy UI |
| Content data | `serde` + `ron` in `assets/data/` |
| Network messages | `serde` + `postcard` |
| Save files | region files + `postcard` + `zstd`; players/bases in SQLite |
| Accounts and server list (M10) | `axum` + `sqlx` |
| Logging | `tracing` |
| Randomness | `rand` + `rand_chacha`, seeded per chunk |
| Testing | `cargo test`, headless Bevy `App` tests, `proptest` |

Enable Bevy's `dynamic_linking` for dev builds only. Set `[profile.dev.package."*"] opt-level = 3`.
Use Rust edition 2024. If a crate has no release for the chosen Bevy version, use the alternative
named in the table and note the change in `PROGRESS.md`.

---

## 3. Workspace architecture

Four crates. The rule that keeps it sane: **`shared` holds every rule of the game, `server`
decides, `client` shows.**

```text
deepcore/
├── Cargo.toml
├── PROMPT.md PROGRESS.md docs/PLAN.md
├── assets/
│   ├── data/     # blocks.ron items.ron recipes.ron machines.ron
│   │             # tools.ron tech_tree.ron layers.ron hazards.ron
│   └── shaders/  # voxel.wgsl (optional custom material)
├── crates/
│   ├── shared/   # content/ voxel/ worldgen/ movement/ inventory/ crafting/
│   │             # power/ hazards/ protocol/ lib.rs
│   ├── server/   # net/ world/ sim/ main.rs
│   ├── client/   # render/ input/ prediction/ ui/ hud/ audio/ main.rs
│   └── lobby/    # axum service (M10)
└── tools/atlas_gen/
```

**Runtime model:** server tick is a fixed 30 Hz simulation in `FixedUpdate`. Client frame is
uncapped, with local movement predicted every frame using the shared movement code. Hosting from
the client runs the server plugins in-process. Chunks are 32³, palette compressed, meshed on
`AsyncComputeTaskPool`. Block entities are ECS entities with a `BlockPos`.

---

## 4. Game design: the world and its ores

One shared, persistent mine, **1024 × 1024 × 512 blocks**, unbreakable bedrock floor and walls.
The surface is a flat-ish camp (the **Base**). Depth is the progression axis.

**Core rules:** every block has a hardness and a minimum tool tier. Break time =
`hardness / (tool_speed × power_multiplier)`, rounded to server ticks. Mined blocks drop into the
miner's inventory when there is room, otherwise as a world pickup. Ore veins come from 3D noise in
blobs of 4–40 blocks. Noise-carved caves appear in every layer. Hauling ore to the Base matters.

### Layers

| # | Layer | Depth | Base block | Min tier | Ores | New hazard |
|---|---|---|---|---|---|---|
| 1 | Topsoil | 0–16 | Dirt, gravel, clay, sand | 1 | Coal, clay, sand | None |
| 2 | Shale Beds | 16–64 | Stone, limestone, shale | 1 | Coal, copper, tin, iron | Darkness |
| 3 | Granite Depths | 64–160 | Granite | 2 | Iron, lead, zinc, sulfur, quartz, bauxite, salt | Flooding |
| 4 | Basalt Furnace | 160–288 | Basalt | 3 | Nickel, tungsten, chromite, graphite, fluorite, zircon, gold | Heat, gas pockets |
| 5 | Deep Crust | 288–448 | Gneiss, obsidian | 4 | Uranium, monazite, rare earths | Radiation, cave-ins |
| 6 | Mantle Edge | 448–512 | Mantle rock | 5 | Dense uranium, core crystals | Extreme heat and radiation |

### Ores

Coal, Clay, Sand (L1); Malachite→copper, Cassiterite→tin, Hematite→iron (L2); Galena→lead,
Sphalerite→zinc (+cadmium byproduct), Sulfur, Quartz, Bauxite, Rock salt (L3); Pentlandite→nickel,
Wolframite→tungsten, Chromite, Graphite, Fluorite, Zircon, Gold (L4); Pitchblende→uranium,
Monazite→rare earths (L5); dense uranium, core crystals (L6).

Uranium is radioactive in the world: standing within 4 blocks of exposed pitchblende raises the
player's dose. The Geiger counter clicks faster nearer uranium. Every ore must have a use.

### Player basics

WASD, jump, sprint, crouch. Left mouse mines; right mouse places or interacts. 9-slot hotbar plus a
36-slot backpack. Vitals: health, stamina, oxygen, body heat, radiation dose. Death drops the
backpack (not the hotbar) as a recoverable crate. Players carry a headlamp and can place lamps.

---

## 5. Progression: from steel pickaxe to nuclear drill

| Era | Name | Key new structures | Tool unlocked | Tier | Speed | Reaches |
|---|---|---|---|---|---|---|
| 0 | Hand Tools | Workbench, Storage Crate, Ladder, Timber Support, Oil Lamp | Steel Pickaxe (start) | 1 | 1.0× | 1–2 |
| 1 | Smelting | Brick Kiln, Stone Furnace, Anvil, Blast Furnace | Hardened Steel Pickaxe | 2 | 1.6× | 3 |
| 2 | Steam | Hand Pump, Steam Boiler, Steam Engine, Crusher, Ore Washer, Rails and Minecart, Mine Lift | Steam Drill | 3 | 3.0× | 4 |
| 3 | Electric | Steam Turbine, Generator, Cables and Poles, Battery Bank, Arc Furnace, Electrolysis Cell, Circuit Assembler, Charging Station, Vent Fan, Electric Pump | Ore Scanner, Headlamp Mk2 | — | — | 4 safely |
| 4 | Chemical | Chemical Plant, Alloy Furnace, Machine Shop, Rare-Earth Separator, Decon Shower | Tungsten-Carbide Drill | 4 | 5.0× | 5 |
| 5 | Nuclear | Leaching Tank, Precipitator, Conversion Plant, Centrifuge Cascade, Fuel Fabricator, Reactor, Cooling Pool, Waste Vault | **Nuclear Drill** | 5 | 12.0×, 3×3 | 6 |

**The nuclear chain:** crusher → leaching tank (with sulfuric acid) → precipitator (with lye) →
conversion plant (with fluorine) → centrifuge cascade (4–16 linked blocks) → fuel fabricator →
fuel rod / compact core cell. Control rods come from cadmium + stainless steel. The reactor is a
5×5×5 multiblock with graphite moderator, fuel rods, control rods, a coolant pump and a heat
exchanger, run from a console; above the limit it scrams if a Scram Panel is built, otherwise it
overheats and leaves a radiation zone for 20 minutes. The Nuclear Drill's core cell powers about
2,000 block breaks. Spent fuel goes to a Cooling Pool then a Waste Vault.

**Power model:** fuel heat (Eras 0–1, one coal = 80 s); steam networks (Era 2, solve supply vs
demand per tick); electricity (Era 3+, generators produce kW, brownout scaling when demand exceeds
supply, batteries buffer). Recompute topology only on placement/removal.

**Machines:** input/output slots, optional fuel or power, current recipe and progress. Idle reasons
must be shown ("No power", "Output full", "Missing sulfuric acid"). Up to two upgrade modules from
Era 3 (Speed +50% speed/+80% power, Efficiency −30% power). Conveyors and item pipes from Era 2.

**Research:** team-shared tech tree at the Research Desk; each node needs parents plus a delivery
of items.

### Hazards

| Hazard | Where | Effect | Counter |
|---|---|---|---|
| Darkness | Layer 2+ | No visibility without light | Oil lamp, headlamp, placed lamps |
| Flooding | Layer 3+ | Water spreads by cellular flow (max 512 blocks) | Electric Pump, sealing blocks |
| Gas pockets | Layer 4+ | Methane drains oxygen; flames ignite it | Respirator, Vent Fan, electric lights |
| Heat | Layer 4+ | Body heat climbs; above limit health drains | Cooling Suit, Vent Fan, coolant lines |
| Cave-ins | Layer 5+ | Unsupported ceiling spans over 7 blocks collapse | Timber or steel supports |
| Radiation | Near uranium, waste, reactor | 100 = grain, 300 = no regen, 600 = damage | Hazmat Suit (−80%), lead, Decon Shower |

**Pacing targets** for 2–4 players: Era 1 ~30 min, Era 2 by 1.5 h, Era 3 by 3 h, Era 4 by 5 h,
first Nuclear Drill 8–12 h. Every tunable number lives in RON. Ship a `balance_report` tool.

---

## 6. Data-driven content

All content lives in RON under `assets/data/` and loads into registries in `shared`. String ids
resolve to compact numeric ids at load time. Validate on load and fail fast: unknown item
references, recipes with no machine, tech nodes with missing parents, ores with no use. The server
sends a content hash during the handshake; a mismatched client is refused with "Content mismatch".

Shapes (extend as needed, keep the files the single source of truth):

```ron
// blocks.ron
[(id: "granite", hardness: 2.5, min_tier: 2, drops: [("granite_cobble", 1)],
  color: (0.55, 0.50, 0.48), pattern: Speckle, layer_hint: Some(3))]

// items.ron
[(id: "nuclear_drill", name: "Nuclear Drill", stack: 1,
  category: Tool((tier: 5, speed: 12.0, area: (3, 3), energy: CoreCell(breaks: 2000))))]

// recipes.ron
[(id: "leach_uranium", machine: "leaching_tank", time: 20.0, power_kw: 40,
  inputs: [("crushed_uranium_ore", 2), ("sulfuric_acid", 1)],
  outputs: [("uranium_solution", 1), ("radioactive_tailings", 1)], tech: "uranium_milling")]

// machines.ron
[(id: "centrifuge", name: "Centrifuge", size: (1, 3, 1), energy: Electric(kw: 150),
  cascade: Some((min: 4, max: 16)), tech: "enrichment")]

// tech_tree.ron
[(id: "steam_power", name: "Steam Power", era: 2, parents: ["steelmaking"],
  cost: [("steel_plate", 20), ("copper_pipe", 10), ("bronze_gear", 5)],
  unlocks: ["steam_boiler", "steam_engine", "copper_pipe_block"])]

// layers.ron
[(id: "granite_depths", depth: (64, 160), base_block: "granite",
  ores: [(block: "galena", frequency: 0.012, vein_size: (4, 18))],
  hazards: [Flooding((pocket_chance: 0.002, max_volume: 512))], ambient_temp: 18.0)]
```

Target counts: ~25 ores/raw materials, 80–120 items, 100–150 recipes, 40–50 machines/blocks,
30–40 tech nodes. Run the validator and `balance_report` after every content change.

---

## 7. Multiplayer and networking

Co-op: up to 16 players share one world, one Base, one tech tree. Private inventories. The server
is authoritative except for predicted movement.

**Connection flow:** connect by IP, "Host game", or the lobby browser. Handshake sends protocol
version, content hash and display name; mismatches are rejected with a readable reason. The server
spawns the player at the Base and streams chunks nearest-first.

| Data | Mechanism | Channel |
|---|---|---|
| Player position/rotation/held item | Replicated components | Unreliable, every tick |
| Player vitals | Replicated, owner only | Reliable on change |
| Inventory | Server → owner message | Reliable |
| Machines | Replicated components | Reliable on change |
| Chunk data | Server → client message | Reliable, ordered |
| Block changes | Server → client, batched per tick | Reliable, ordered |
| Power/steam stats | Server → client, 1 Hz | Unreliable |
| Tech tree, chat, notifications | Server → all | Reliable |
| Player intents | Client → server | Reliable; movement unreliable + sequenced |

**Intents:** `MoveInput` (seq, dir, jump, sprint, crouch, yaw, pitch), `StartMining`, `StopMining`,
`PlaceBlock`, `Interact`, `MoveItems`, `Craft`, `SetMachineRecipe`, `ResearchDeposit`, `Chat`,
`Respawn`. The server validates reach (5 blocks, 6 with the drill), line of sight by voxel raycast,
tool tier, that the item is in the slot, that the recipe is unlocked, and machine proximity.
Invalid intents are dropped and logged; they must never crash or desync the server.

**Mining over the network:** the client predicts the crack animation with the shared formula; the
server runs the real timer and broadcasts the block change on completion. Area tools break the
whole face in one server action.

**Movement prediction:** `shared::movement` is pure and deterministic. The client applies each
input locally and buffers unacknowledged inputs; the server applies the same inputs and replicates
the authoritative position with the last processed `seq`; the client rewinds and replays.
Corrections under 0.3 blocks are smoothed over 100 ms.

**Interest management:** chunks within 6 horizontally and 4 vertically; unload as the player moves
away; replicon per-client visibility for entities.

**Persistence:** unmodified chunks are never saved. Modified chunks go to 8×8×8 region files.
SQLite stores players, tech tree, machines and reactor state. Autosave every 5 minutes and on clean
shutdown, without stalling the tick.

**Budgets:** under 50 KB/s per client steady state. `--simulate-latency` and `--simulate-loss` on
both binaries. `server --bots <N>` for headless load/desync testing. F3 debug overlay showing FPS,
tick time, ping, bandwidth, loaded chunks and prediction corrections.

---

## 8. Milestones

Build in this order. Networking comes early (M2) on purpose. Each milestone ends with a playable
build, green tests and an updated `PROGRESS.md`.

- **M0: Workspace and skeleton** — four crates, pinned versions, profiles, `scripts/check.sh`;
  client window with a lit cube; headless server at 30 Hz.
- **M1: Voxel world (local)** — palette chunks, deterministic worldgen for all six layers with
  caves and veins, procedural atlas, greedy meshing, FPS controller with AABB collision, raycast
  targeting, mining with tier rules, hotbar and depth meter. *Dig to the bottom of layer 2 and be
  blocked by layer-3 granite with the steel pickaxe.*
- **M2: Multiplayer foundation** — authoritative server, handshake (protocol + content hash),
  chunk streaming with interest management, batched block changes, replicated players with
  interpolation, prediction/reconciliation, mining and placing through validated intents, main
  menu with Host game / Join by IP, latency/loss flags, bots. *Two clients see each other move and
  dig; responsive at 150 ms; 8 bots for 10 minutes without desync.*
- **M3: Content, inventory, crafting (Era 0)** — RON loaders, validator, `balance_report`, egui
  inventory with drag/split/quick-move, pickups, death crate, respawn, workbench crafting, trees
  and timber, ladders, supports, oil lamps, darkness.
- **M4: Machines, research, Era 1** — block entities, multiblock ghost preview, machine UI with
  idle reasons, fuel burning, Kiln, Stone Furnace, Anvil, Blast Furnace, Research Desk and tech
  tree graph.
- **M5: Era 2, steam and logistics** — pipe networks and steam solver, Hand Pump, Boiler, Steam
  Engine, Crusher, Ore Washer with yield bonuses, conveyors and item pipes, rails/minecarts/Mine
  Lift, Steam Drill with canisters, flooding and oxygen.
- **M6: Era 3, electricity** — cable networks, electric solver, batteries, brownout scaling,
  network overlay, Turbine, Generator, Battery Bank, Arc Furnace, Electrolysis Cell, Circuit
  Assembler, Charging Station, upgrade modules, Ore Scanner, Vent Fan, Electric Pump, gas hazard.
- **M7: Era 4, chemistry** — Chemical Plant, Alloy Furnace, Machine Shop, Rare-Earth Separator,
  Decon Shower, heat and cave-ins, Cooling Suit, Respirator, Hazmat Suit, Geiger Counter,
  Tungsten-Carbide Drill.
- **M8: Era 5, nuclear** — Leaching Tank, Precipitator, Conversion Plant, Centrifuge Cascade, Fuel
  Fabricator, control rods, Reactor with console and scram, Cooling Pool, Waste Vault, Nuclear
  Drill, radiation. *A headless integration test feeds raw materials through every nuclear
  structure and produces a working Nuclear Drill and a stable reactor.*
- **M9: Persistence and polish** — region files and SQLite, autosave, crash-safe writes, audio,
  particles, animations, menus and settings, objective list, balance pass.
- **M10: Full stack** — lobby with axum + sqlx, argon2 accounts, session tokens, rate limiting,
  server registration and heartbeat, server browser, token verification, Dockerfiles.

---

## 9. Quality bar and definition of done

**Tests:** chunk indexing and palette round-trips; worldgen determinism; the break-time formula;
inventory moves and stacking with `proptest`; recipe matching; steam and power solving including
brownouts; radiation dose maths. Headless Bevy `App` tests scripting intents. One progression
integration test per Era (the M8 nuclear test is the most important). Network tests with simulated
latency asserting convergence.

**Performance budgets:** ≥60 FPS at view radius 6 on a mid-range GPU; chunk mesh <4 ms; server tick
<10 ms with 16 players and 200 machines; chunk generation <5 ms; <50 KB/s per client; <2 GB server
memory for 16 players. Never block the main thread on generation, meshing, disk or network.

**Code standards:** `clippy -D warnings` passes and `cargo fmt` is applied. No `unwrap()` on
network input, file input or content lookups. Every public type and plugin in `shared` has a
one-line doc comment. Replicated components and protocol messages are versioned by a single
`PROTOCOL_VERSION` constant; bump it whenever their shape changes.

**Definition of done:** a fresh clone builds with `cargo build --release` and runs with no
downloaded assets; two or more players progress from the steel pickaxe through all six Eras to a
working Nuclear Drill and a stable reactor; every ore has a use and every structure exists; the
world, machines, inventories and tech tree survive a restart; accounts and the server browser work
through the lobby with Dockerfiles; `README.md` explains build/host/join/adding content and
`PROGRESS.md` lists known issues.
