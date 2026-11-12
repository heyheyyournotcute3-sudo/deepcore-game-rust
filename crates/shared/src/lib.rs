//! Deepcore shared game logic.
//!
//! This crate holds every rule of the game and nothing else: no window, no renderer, no
//! sockets, no replication framework. The server decides with it, the client predicts and
//! displays with it, and the tests verify it without opening a window.

pub mod content;
pub mod crafting;
pub mod movement;
pub mod simulation;
pub mod voxel;
pub mod worldgen;

pub use content::{Registry, default_content_dir, load_dir};
pub use voxel::{BlockId, BlockPos, Chunk, ChunkPos, Dir};

/// Protocol version. Bump whenever the shape of replicated components or intents changes.
pub const PROTOCOL_VERSION: u32 = 1;

/// The simulation tick rate of the server, in hertz.
pub const TICK_HZ: u32 = 30;

/// Fixed timestep of the simulation in seconds (`1/30`).
pub const TICK_SECONDS: f32 = 1.0 / TICK_HZ as f32;
