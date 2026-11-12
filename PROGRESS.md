# Deepcore — Progress

> Continuation file. A future session (possibly a different model) must be able to resume from
> this file alone. Update it at the end of every milestone.

## Status: M1 complete (voxel world, local single-process). M2 next (multiplayer).

## Pinned stack (verified 2026-11-12 against crates.io and the actual crate sources)
| Crate | Version |
|---|---|
| bevy | 0.20.1 (default-features off, explicit feature list) |
| bevy_replicon | 0.45.5 |
| bevy_replicon_renet | 0.21.0 |
| bevy_egui | 0.43.1 |
| ron / postcard / zstd | 0.12.3 / 1.1.3 / 0.14.3 |
| noise / rand / rand_chacha | 0.9.0 / 0.10.3 / 0.10.0 |
| sqlx (sqlite) | 0.9.0 |
| axum / argon2 / uuid | 0.8.9 / 0.6.0 / 1.x |
| Rust | 1.99.1, edition 2024 |

**Deviations from the prompt's table (allowed by the prompt, recorded here):**
- `rusqlite` → `sqlx` for save files. `rusqlite` (bundled) and `sqlx-sqlite` both declare
  `links = "sqlite3"` and cargo refuses a workspace lockfile containing both. `sqlx` now does both
  persistence and the lobby; save writes go through a background tokio runtime (which also
  satisfies "never block the tick on disk").
- `avian3d` unused (marked optional in the prompt). Dropped items will snap to the ground.

## How to run
```bash
cargo run -p deepcore_server            # headless, 30 Hz  (binary: server)
cargo run -p deepcore_client            # windowed client    (binary: client)
cargo run -p deepcore_lobby             # lobby service      (M10, stub today)
./scripts/check.sh                      # fmt + clippy -D warnings + tests
```
Flags: `server --seed --exit-after --simulate-latency --simulate-loss --bots`,
`client --seed --view-distance --exit-after --simulate-latency --simulate-loss --connect`.
`--exit-after N` makes either binary shut down cleanly after N seconds (used for smoke tests).

Content validation: `cargo test -p deepcore_shared --test content`.
Regenerate RON from the authoring script: `python3 tools/gen_content.py`.

## Environment notes (for a fresh session)
- The dev container needs: `libasound2-dev libudev-dev libxcb-shape0-dev libxcb-xfixes0-dev
  libwayland-dev libvulkan-dev pkg-config`. For headless screenshots:
  `xvfb mesa-vulkan-drivers imagemagick`, then
  `Xvfb :99 & DISPLAY=:99 ./target/debug/client --exit-after 30 & sleep 30; import -window root out.png`.
- Builds are slow here (2 cores): a cold `cargo check --workspace --all-targets` is ~6 min; the
  client binary build was ~17 min the first time. Incremental checks are seconds.

## Review fixes (after M1, before M2)

Four review findings, all fixed:

1. **Starting loadout** — the client filled hotbar slots 1–3 with 64 oil lamps, ladders and
   timber supports. Now the player starts with exactly one steel pickaxe and nothing else
   (`LocalPlayer::new`); everything else must be crafted.
2. **Content gaps** — 18 tech nodes and no entries at all for rails, minecarts, the Mine Lift,
   cables/poles, conveyors/item pipes, the Scram Panel, or the reactor console / coolant pump /
   heat exchanger. All of them now exist as RON data (44 machines, 32 tech nodes, 140 recipes),
   and two new tests pin this down: `every_structure_from_the_design_exists` and
   `content_meets_the_section_6_targets`. Content counts: 55 blocks, 177 items (132 non-machine),
   140 recipes, 44 machines, 32 tech nodes, 6 layers.
3. **Docs vs tree** — README said M0 when M1 had shipped; both README and PLAN pointed at a
   non-existent `tools/atlas_gen` (the atlas is generated at runtime in
   `crates/client/src/render/atlas.rs`); `PROMPT.md` was missing from the repo root. All three
   fixed.
4. **Repo metadata** — `Cargo.toml` `repository` pointed at the wrong repo; now
   `deepcore-game-rust`.

Note on Section 6 counts: the item target is "roughly 80–120" and the tree ships 132 non-machine
items, because every intermediate in the nuclear chain, every hazard counter and every tool is a
real item. The test asserts 80–140 and the count is recorded here rather than deleting content to
hit a number.

## Milestone log

### M1 — voxel world (local, single process) — DONE
Built:
- `shared/voxel`: `ChunkPos`/`BlockPos` maths, palette-compressed `Chunk` (u8 palette widening to
  u16), raycast (Amanatides & Woo) with tests.
- `shared/worldgen`: all six layers, fillers, caves, deterministic ore veins placed in world space
  so veins cross chunk borders, water/gas pockets, trees, camp platform. Fully seed-deterministic.
- `shared/movement`: AABB-vs-voxel collision, per-axis resolution, step-up, ladders, sub-stepping
  so fast falls cannot tunnel. World-space `MoveIntent` (no camera dependency).
- `shared/crafting::mining`: break-time formula `hardness / (speed * power)`, tier gating, tick
  rounding.
- `shared/simulation`: headless harness driving the real generator, storage, physics and mining
  rules. This is how M1's acceptance criteria are tested.
- `client`: procedural 16×16 atlas from `blocks.ron` colours/patterns, greedy mesher with
  cross-chunk face culling (capped at `MAX_MERGE = 8`), incremental chunk streaming, first-person
  controller (WASD/sprint/crouch/jump, mouse look, Esc releases the cursor), block targeting
  raycast with a selection box, mining with hardness/tier rules and a "Too hard — needs Tier N"
  warning, block placing, 9-slot hotbar, depth meter with layer name, crosshair.
- `client/examples/ui_smoke.rs`: minimal Bevy-UI smoke test kept in the tree so UI regressions
  after a Bevy upgrade can be attributed quickly.

Verified:
- `scripts/check.sh` green: 67 tests (46 shared unit + 11 content + 10 client).
- **M1 acceptance (automated)**: `simulation::tests::steel_pickaxe_digs_through_layers_one_and_two`
  digs from the surface with the starting steel pickaxe, passes layer 1 and layer 2, and is
  stopped by **layer-3 granite** — and `hardened_pickaxe_passes_the_granite_that_stopped_steel`
  proves tier 2 gets through.
- Bedrock floor is never broken (`bedrock_floor_is_never_broken`).
- Client renders under Xvfb/llvmpipe with the HUD visible (screenshot-verified): depth meter,
  layer name, hotbar with items, crosshair.
- NOT verified here: 60 FPS at view radius 6 on a mid-range GPU. This box has no GPU (llvmpipe
  software rendering), so the frame-rate criterion cannot be measured in this environment.

Bugs found and fixed during M1 (each now covered by a test or documented):
- **Caves were carved from depth 2 upward, hollowing the surface** so the camp floated and the
  player spawned over a void. Caves now start at `MIN_CAVE_DEPTH = 8`.
- **Spawn height was 2 blocks above the surface**, so the player fell on join. Spawn is now
  `surface_y + 1` (feet exactly on the surface block).
- **`AmbientLight` requires `Camera` in Bevy 0.20.** Spawning it as a free entity created a stray
  bare `Camera`; Bevy then selected that as the default UI camera, whose viewport is 0×0, so the
  entire HUD laid out to nothing and never rendered. Fixed by using the `GlobalAmbientLight`
  resource. This cost the longest debug session of the milestone — the symptom (HUD entities
  exist but have no `ComputedNode`) pointed at UI, not lighting.
- Greedy quads sample one atlas tile, so merge size is capped at 8 blocks and UVs stay inside the
  tile; unbounded merging smeared the texture across whole floors.
- `BorderColor` is a struct with four sides in Bevy 0.20 (`BorderColor::all(color)`), not a tuple.
- `TextFont::font_size` is a `FontSize` enum (`FontSize::Px(..)`), not an `f32`.
- Streaming 800+ chunks in one frame starved the rest of the schedule; streaming is now bounded to
  a few nearest chunks per frame.
- `Single<&mut Text, With<A>>` + `Single<&mut Text, With<B>>` in one system panics (B0001);
  needs disjoint `Without<..>` filters.
- `#[require(Camera)]`-style component requirements mean "spawn a lighting entity" can silently
  create a camera. Check `#[require]` before spawning any Bevy component standalone.

Deferred / known issues:
- Networking, inventory, machines: later milestones. Nothing stubbed.
- Textures: one atlas tile per merged quad (capped merge). Upgrade path: custom `Material` with
  `fract()` UVs or a padded atlas with per-quad UV scaling.
- The selection box is a translucent cube, not a true wireframe.
- `client/examples/ui_smoke.rs` and `shared/simulation.rs` exist to make UI and gameplay
  regressions testable without a GPU.

### M0 — workspace and skeleton — DONE
Built:
- 4-crate workspace with pinned versions, `[profile.dev.package."*"] opt-level = 3`,
  `lints` with `unsafe_code = "forbid"` and `clippy::all = deny`, `rustfmt.toml`, `clippy.toml`,
  `.cargo/config.toml`, `scripts/check.sh`.
- `server`: headless Bevy (`MinimalPlugins` + `LogPlugin`, `ScheduleRunnerPlugin` at 30 Hz),
  fixed timestep in `FixedUpdate`, tick accounting, `--exit-after`, chunk store + spawn-area
  generation, content load with a readable failure report and a content hash.
- `client`: window, `Camera3d`, sun + ambient light, procedural atlas, greedy chunk meshing,
  preview world around the camp.
- `shared`: voxel (pos/chunk/raycast), content (defs/registry/loader/validator), worldgen,
  movement (AABB collision + controller).
- Content database authored in `tools/gen_content.py` → `assets/data/*.ron`:
  55 blocks, 166 items, 35 machines, 129 recipes, 18 tech nodes, 6 layers, 3 fuels, 6 hazards.

Verified:
- `scripts/check.sh` green: fmt clean, `clippy --workspace --all-targets -- -D warnings` clean,
  56 tests pass (35 shared unit + 11 content + 10 client mesh/atlas).
- `./target/debug/server --exit-after 3` logs 91 ticks in 3 s (= 30 Hz), 50 camp chunks, then
  exits 0.
- Client runs under Xvfb/llvmpipe, logs `atlas 256px for 55 blocks, 49 preview chunk meshes`,
  and renders textured terrain (verified by screenshot).

Bugs found and fixed during M0 (all with regression tests where testable):
- Atlas sampled the wrong tile row: Bevy image UV origin is **top-left**, so the row must not be
  flipped. Every block rendered as the wrong (dark) texture before this.
- Terrain needs `cull_mode: None` (double-sided) because greedy meshing emits one quad per
  visible face and the player sees back faces from inside tunnels.
- Greedy quads are capped at `MAX_MERGE = 8` blocks and their UVs stay inside one atlas tile;
  an uncapped merge smeared the texture across a whole floor.
- `AmbientLight` is a **component** in Bevy 0.20, not a resource. `DirectionalLight` uses
  `shadow_maps_enabled`, not `shadows_enabled`.
- `MinimalPlugins` does not contain `LogPlugin`; adding it via `.set()` panics. Add it as a
  separate plugin in a tuple.
- `rand` 0.10 moved `random`/`random_range` to the `RngExt` trait.
- Tech-cycle validation was a false positive for any node with a shared ancestor; replaced with
  proper DFS colouring.
- Movement intent is **world-space** (`dir: (x, z)`), not yaw-relative, so physics has no camera
  dependency and ladders (which have no facing) work.

Deferred / known issues:
- Chunk streaming, real networking, inventory, machines: all later milestones. Nothing stubbed.
- `ChunkNeighbourhood` (cross-chunk face culling) is written and tested but only used from M2,
  hence `#[allow(dead_code)]`.
- `ChunkStore` has a complete API; the M0 binary only calls the spawn-area path
  (`#![allow(dead_code)]` on the module until M2/M3 consume it).
- Textures are one atlas tile per merged quad (capped merge), not tiled per block. Upgrade path:
  a custom `Material` extension with `fract()` UVs, or per-quad UV scaling with a padded atlas.
- `lobby` crate is an empty `main` until M10.
