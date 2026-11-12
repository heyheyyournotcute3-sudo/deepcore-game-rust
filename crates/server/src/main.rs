//! Deepcore dedicated server.
//!
//! Headless: `MinimalPlugins` plus the pieces we need (time, log, asset IO for content). The
//! simulation runs at a fixed 30 Hz in `FixedUpdate`. The server is the source of truth for
//! every gameplay rule.

mod sim;
mod world;

use std::time::Duration;

use bevy::MinimalPlugins;
use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use clap::Parser;

use deepcore_shared::TICK_SECONDS;
use deepcore_shared::content::{Registry, default_content_dir, load_dir};
use deepcore_shared::worldgen::WorldGen;

/// Command line options for the server.
#[derive(Debug, Parser, Resource)]
#[command(name = "server", about = "Deepcore dedicated server")]
pub struct Cli {
    /// Port to listen on for game traffic.
    #[arg(long, default_value_t = 47000)]
    pub port: u16,
    /// World seed. The same seed always produces the same mine.
    #[arg(long, default_value_t = 0x0D33C0)]
    pub seed: u64,
    /// Directory holding the RON content files.
    #[arg(long)]
    pub content: Option<String>,
    /// Simulated one-way latency in milliseconds, for testing prediction.
    #[arg(long, default_value_t = 0)]
    pub simulate_latency: u64,
    /// Simulated packet loss in percent, for testing reliability.
    #[arg(long, default_value_t = 0)]
    pub simulate_loss: u8,
    /// Number of headless bot clients to run for load testing.
    #[arg(long, default_value_t = 0)]
    pub bots: u32,
    /// Exit after this many seconds; `0` runs until killed. Used by CI smoke tests.
    #[arg(long, default_value_t = 0)]
    pub exit_after: u64,
}

/// Content loaded at startup, shared by every system that needs it.
#[derive(Resource)]
pub struct Content {
    /// The validated registry.
    pub registry: Registry,
    /// Content hash sent to clients during the handshake.
    pub hash: u64,
}

/// The world generator for this server.
#[derive(Resource)]
pub struct World {
    /// Deterministic generator for the seed.
    pub generator: WorldGen,
}

/// Server start time, used to implement `--exit-after`.
#[derive(Resource)]
pub struct RunLimit {
    /// Seconds to run for, or `0` for unlimited.
    pub seconds: u64,
    /// Elapsed simulated time.
    pub elapsed: f32,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let content_dir =
        cli.content.clone().map_or_else(default_content_dir, std::path::PathBuf::from);
    let (registry, loaded) = load_dir(&content_dir).map_err(|errors| {
        let joined: Vec<String> = errors.iter().map(ToString::to_string).collect();
        anyhow::anyhow!(
            "content failed to load from {}:\n  {}",
            content_dir.display(),
            joined.join("\n  ")
        )
    })?;

    let hash = loaded.hash;
    let blocks = registry.blocks.len();
    let items = registry.items.len();
    let recipes = registry.recipes.len();
    let machines = registry.machines.len();
    let tech = registry.tech.len();

    let generator = WorldGen::new(cli.seed, registry.clone());

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins
            .set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(f64::from(TICK_SECONDS)))),
        LogPlugin { level: bevy::log::Level::INFO, ..default() },
    ))
    .insert_resource(Time::<Fixed>::from_seconds(f64::from(TICK_SECONDS)))
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(Content { registry, hash })
    .insert_resource(World { generator })
    .insert_resource(RunLimit { seconds: cli.exit_after, elapsed: 0.0 })
    .insert_resource(cli)
    .add_plugins((sim::SimPlugin, world::WorldPlugin));

    info!(
        "Deepcore server: seed {:#x}, content hash {hash:#018x}, {blocks} blocks, {items} items, \
         {recipes} recipes, {machines} machines, {tech} tech nodes",
        app.world().resource::<World>().generator.seed()
    );
    info!("tick rate {}/s", deepcore_shared::TICK_HZ);

    app.run();
    Ok(())
}
