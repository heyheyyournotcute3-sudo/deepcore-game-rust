# Deepcore

A co-op, first-person, 3D voxel mining game written entirely in Rust with Bevy.

Players spawn at a surface camp above one huge, persistent, procedurally generated mine. Everyone
starts with a steel pickaxe and nothing else. Dig down, haul ore home, build machines — furnaces,
crushers, boilers, generators, refineries — and each one unlocks a better tool and a deeper, harder
layer. The end of the ladder is nuclear power: find uranium in the Deep Crust, run it through the
full mill → leach → precipitate → convert → enrich → fabricate chain, and build a **Nuclear Drill**
that bores 3×3 tunnels, plus a reactor that powers the whole Base.

The server is authoritative for every rule. Clients predict only their own movement.

## Status

**M1 (voxel world, local) complete.** The workspace builds, the client renders and meshes the
mine with a first-person controller, mining with tool-tier gates, a hotbar and a depth meter, and
the server runs headless at 30 Hz. The content database (55 blocks, 177 items, 140 recipes,
44 machines, 32 tech nodes, 6 layers) loads and validates against Section 6's targets.

Multiplayer (M2) is next. See [PROGRESS.md](PROGRESS.md) for the milestone log, known issues and
explicitly deferred work, and [docs/PLAN.md](docs/PLAN.md) for the architecture.

## Quick start

Requires Rust 1.99+ (edition 2024). No downloaded art assets: every texture is generated at
runtime from the block definitions.

```bash
# Headless server, 30 Hz, generates the camp at startup
cargo run -p deepcore_server

# Windowed client (local preview world until M2 lands networking)
cargo run -p deepcore_client

# Everything the project gates on: fmt, clippy -D warnings, tests
./scripts/check.sh
```

Useful flags:

```bash
cargo run -p deepcore_server -- --seed 1234 --exit-after 10
cargo run -p deepcore_client -- --seed 1234 --view-distance 6
cargo run -p deepcore_server -- --simulate-latency 150 --simulate-loss 2   # network testing
```

On Linux the client needs the usual windowing/audio development packages:

```bash
sudo apt install libasound2-dev libudev-dev libx11-dev libxcb-render0-dev \
  libxcb-shape0-dev libxcb-xfixes0-dev libwayland-dev libvulkan-dev pkg-config
```

## Layout

```
crates/shared/   every rule of the game: voxels, worldgen, movement, content, and later
                 inventory, crafting, power and hazards. No rendering, no sockets.
crates/server/   headless authoritative server binary (`server`)
crates/client/   windowed game binary (`client`): rendering, input, prediction, UI, HUD
crates/lobby/    axum + sqlx accounts and server browser (M10)
assets/data/     ALL content as RON: blocks, items, machines, recipes, tech tree, layers, fuels,
                 hazards. This is the single source of truth.
tools/           gen_content.py authors the RON files in assets/data/
                 (the block texture atlas is generated at runtime, see below)
```

## Adding content

Everything is data. To add an ore:

1. Add the block to `tools/gen_content.py` (or edit `assets/data/blocks.ron` directly) with a
   `pattern: OreFleck(...)` so it reads as an ore, and a `drops` entry.
2. Add the item it drops to `items.ron`.
3. Add a recipe that consumes it to `recipes.ron`, and reference the ore block in a layer's
   `ores` list in `layers.ron`.

Then run the validator, which fails with a readable message on any dangling reference or any ore
with no use:

```bash
cargo test -p deepcore_shared --test content
```

Regenerating the data files from the authoring script:

```bash
python3 tools/gen_content.py
```

## Placeholder art

There are no downloaded assets. Block textures are generated at runtime: every 16×16 tile is
painted from the block's `color` and `pattern` in `blocks.ron` and packed into an atlas by
`crates/client/src/render/atlas.rs`. Machines are built from Bevy primitives.

## Design rules the code follows

- **`shared` holds the rules, `server` decides, `client` shows.** The same `shared::movement` code
  runs on both sides so prediction matches authority.
- **Data over code.** Balance numbers live in RON, never in Rust.
- **Deterministic worldgen.** The same seed produces identical chunks on every machine.
- **No stubs.** Anything unfinished is absent and listed in `PROGRESS.md`.

## Licence

MIT OR Apache-2.0.
