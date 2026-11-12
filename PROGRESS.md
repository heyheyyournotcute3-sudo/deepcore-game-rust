# Deepcore — Progress

> Continuation file. A future session (possibly a different model) must be able to resume from
> this file alone. Update it at the end of every milestone.

## Status: M0 complete (workspace + skeleton). M1 next.

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

## Milestone log

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
