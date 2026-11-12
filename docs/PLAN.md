# Deepcore — Plan

## Understanding of the game

Deepcore is a co-op, first-person, 3D voxel mining game written entirely in Rust. A team of up
to 16 players spawns at a surface camp above one shared, persistent, procedurally generated mine
(1024 × 1024 × 512 blocks). Everyone starts with a steel pickaxe. The only loop in the game is:

> dig deeper → haul ore home → build a structure → it unlocks a better tool or refining step →
> dig deeper

The loop runs for six Eras (Hand Tools → Smelting → Steam → Electric → Chemical → Nuclear). Depth
is the progression axis: every layer is harder, richer and more dangerous, and is gated behind the
minimum tool tier the previous Era produces. The end goal is nuclear power: find pitchblende in the
Deep Crust, run it through the full mill → leach → precipitate → convert → enrich → fabricate chain,
and use the output to build a Nuclear Drill (3×3 face mining, 12× speed) and a stable Reactor that
powers the whole Base.

Everything the players touch is a thing they physically build and operate in the world. There is no
menu-only progression.

## Non-negotiable rules I am holding myself to

1. **The server is the source of truth.** Mining, placing, inventories, machines, power, hazards,
   research, damage — all decided on the server. Clients send intents; clients predict only their
   own movement.
2. **`shared` holds every rule of the game.** `shared` is pure logic: no window, no renderer, no
   sockets, no replicon. `server` decides, `client` shows. The same movement code runs on both.
3. **Data over code.** Blocks, items, recipes, machines, tech nodes, layers and hazards are RON in
   `assets/data/`. Adding an ore must never require a Rust change. Content is validated at load and
   the load fails loudly on a bad reference.
4. **No stubs.** No `todo!()`, no empty systems, no "implement later" in anything I call finished.
   Anything not finished is *absent* and listed in `PROGRESS.md`.
5. **Verify APIs, never guess.** Bevy breaks its API every release. I pinned the whole stack by
   reading the actual crate sources for the exact versions (see below), not from memory.
6. **Compile constantly.** `cargo check` after every meaningful change; `clippy -D warnings` and
   `cargo test` before closing a milestone.

## Pinned stack (verified against crates.io + crate sources, 2026-11-12)

| Crate | Version | Notes |
|---|---|---|
| `bevy` | 0.20.1 | latest stable; `default-features = false` + explicit feature list |
| `bevy_replicon` | 0.45.5 | supports bevy ^0.20 |
| `bevy_replicon_renet` | 0.21.0 | supports bevy ^0.20, bevy_replicon ^0.45 |
| `bevy_egui` | 0.43.1 | supports bevy ^0.20 |
| `ron` / `postcard` / `zstd` | 0.12.3 / 1.1.3 / 0.14.3 | content + wire + chunk compression |
| `noise` / `rand` / `rand_chacha` | 0.9.0 / 0.10.3 / 0.10.0 | deterministic worldgen |
| `sqlx` | 0.9.0 (sqlite) | persistence **and** lobby (one sqlite linkage) |
| `axum` / `argon2` / `uuid` | 0.8.9 / 0.6.0 / 1.x | lobby service |

Two deliberate deviations from the prompt's table, recorded per its own instruction:

- **`rusqlite` → `sqlx`** for save files. `rusqlite` (bundled) and `sqlx-sqlite` both declare
  `links = "sqlite3"`, and cargo refuses to build a workspace lockfile containing both. Rather than
  downgrade anything, the server uses `sqlx` for both jobs. Save writes happen on a background
  tokio runtime, which also satisfies "never block the tick on disk".
- **`avian3d` not used.** Dropped items snap to the ground; the prompt marks item physics optional
  and a second physics engine is not worth the dependency.

Movement is hand-written AABB-vs-voxel collision (as the prompt requires), so no physics engine is
needed at all.

## Crate layout

```
deepcore/
├── Cargo.toml            # workspace, pinned versions, profiles, lints
├── assets/data/*.ron     # ALL content (single source of truth)
├── crates/
│   ├── shared/           # pure rules; no rendering, no sockets, no replicon
│   │   ├── content/      # RON loaders + registries (BlockId/ItemId/RecipeId) + validator
│   │   ├── voxel/        # Chunk (palette compressed), ChunkPos/BlockPos math
│   │   ├── worldgen/     # layers, caves, ore veins — deterministic from seed
│   │   ├── movement/     # AABB collision + player controller (both sides)
│   │   ├── inventory/    # stacks, slots, transfer rules
│   │   ├── crafting/     # recipe matching + machine processing state machine
│   │   ├── power/        # steam + electric network solving (incl. brownout)
│   │   ├── hazards/      # radiation, heat, oxygen/gas, flooding, cave-ins
│   │   └── protocol/     # replicated component shapes, intents, events, PROTOCOL_VERSION
│   ├── server/           # headless authoritative binary
│   │   ├── net/          # handshake, content hash, visibility
│   │   ├── world/        # chunk store, generation, persistence
│   │   └── sim/          # mining validation, machine ticks, power, hazards
│   ├── client/           # windowed binary
│   │   ├── render/ input/ prediction/ ui/ hud/ audio/
│   └── lobby/            # axum + sqlx accounts / tokens / server browser (M10)
└── tools/atlas_gen/      # procedural block texture atlas
```

One Bevy plugin per feature (`MiningPlugin`, `InventoryPlugin`, `PowerGridPlugin`, …). Files stay
small and named by purpose.

## Runtime model

- Server: fixed 30 Hz sim in `FixedUpdate`; machines, power and hazards tick there.
- Client: uncapped frames; local movement predicted every frame with the *same* `shared::movement`
  code; reconciliation by rewind + replay of unacknowledged inputs.
- Chunks are 32³, palette-compressed, generated from a seed, meshed on `AsyncComputeTaskPool`.
- Block entities (machines, chests) are ECS entities with a `BlockPos`, never voxel data.

## Milestones (from the prompt, in my own words)

| # | Milestone | Acceptance criteria |
|---|---|---|
| M0 | Workspace + skeleton | 4 crates, pinned versions, profiles, `scripts/check.sh`; client window w/ lit cube; headless server logging 30 Hz ticks; check.sh green |
| M1 | Voxel world (local) | palette chunks + pos math tested; deterministic worldgen for all 6 layers w/ caves + veins; procedural atlas; async greedy meshing; FPS controller w/ AABB collision; raycast targeting; mining/placing w/ tier rules; hotbar + depth meter; dig to bottom of layer 2, blocked by layer-3 granite with steel pickaxe |
| M2 | Multiplayer foundation | authoritative server + handshake (protocol + content hash); chunk streaming w/ interest mgmt; batched block changes; replicated players + prediction/reconciliation; mining/placing via validated intents; main menu Host/Join; latency+loss sim; bots; 2 clients see each other; responsive at 150 ms; 8 bots 10 min no desync |
| M3 | Content, inventory, crafting (Era 0) | RON loaders + validator + balance_report; egui backpack/hotbar w/ drag/split/quick-move; pickups, death crate, respawn; workbench hand-crafting; trees/timber, ladders, supports, oil lamps, darkness |
| M4 | Machines, research, Era 1 | block entities + multiblock ghost preview w/ rotation; machine UI w/ slots + idle reasons; fuel burning; Kiln, Stone Furnace, Anvil, Blast Furnace; Research Desk + tech tree graph UI; team reaches Hardened Steel Pickaxe and layer 3 |
| M5 | Era 2 steam + logistics | pipe networks + steam solver; Hand Pump, Boiler, Steam Engine, Crusher, Ore Washer w/ yield bonuses; conveyors/item pipes; rails, minecarts, Mine Lift; Steam Drill w/ canisters; flooding + oxygen |
| M6 | Era 3 electricity | cable networks + electric solver, batteries, brownout scaling, network overlay; Turbine, Generator, Battery Bank, Arc Furnace, Electrolysis Cell, Circuit Assembler, Charging Station; upgrade modules; Ore Scanner; Vent Fan, Electric Pump, gas hazard |
| M7 | Era 4 chemistry | Chemical Plant, Alloy Furnace, Machine Shop, Rare-Earth Separator, Decon Shower; heat + cave-ins; Cooling Suit, Respirator, Hazmat Suit, Geiger Counter; Tungsten-Carbide Drill |
| M8 | Era 5 nuclear | Leaching Tank, Precipitator, Conversion Plant, Centrifuge Cascade (scales with length); Fuel Fabricator + control rods; Reactor multiblock w/ console, scram, overheat; Cooling Pool, Waste Vault; Nuclear Drill 3×3; radiation; **headless integration test proves the whole chain** |
| M9 | Persistence + polish | region files + SQLite, autosave, crash-safe temp+rename; audio, particles, machine animations; menus/settings; objective list; balance pass |
| M10 | Full stack | lobby (axum+sqlx): argon2 accounts, sessions, rate limiting, server registration/heartbeat, browser + token verify; Dockerfiles; README |

## Test strategy

- Unit: chunk indexing, palette round-trip, worldgen determinism, break-time formula, inventory
  moves/stacking (`proptest`), recipe matching, steam/electric solving incl. brownout, radiation.
- Headless Bevy app tests: build an `App` with server plugins, no window, script intents, step the
  schedule, assert outcomes.
- Progression integration tests: one per Era, raw inputs → Era unlock. The M8 nuclear test is the
  most important test in the project.
- Network: server+client in one process over a local transport with simulated latency, assert
  convergence.
