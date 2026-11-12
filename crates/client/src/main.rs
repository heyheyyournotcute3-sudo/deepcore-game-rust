//! Deepcore client.
//!
//! Windowed game: a first-person view of the voxel mine with procedural textures, a HUD, and
//! (from M2) the network connection to a server. The client predicts only its own movement and
//! shows everything else as the server reports it.

mod hud;
mod input;
mod player;
mod render;
mod world;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use clap::Parser;

use deepcore_shared::Registry;
use deepcore_shared::content::{default_content_dir, load_dir};

use render::{ChunkRenderPlugin, ViewConfig};
use world::ClientWorld;

/// Command line options for the client.
#[derive(Debug, Parser, Resource)]
#[command(name = "client", about = "Deepcore client")]
pub struct Cli {
    /// Server address to join, e.g. `127.0.0.1:47000`. Empty plays the local preview world.
    #[arg(long, default_value = "")]
    pub connect: String,
    /// World seed for the local preview world.
    #[arg(long, default_value_t = 0x000D_33C0)]
    pub seed: u64,
    /// Directory holding the RON content files.
    #[arg(long)]
    pub content: Option<String>,
    /// Render distance in chunks.
    #[arg(long, default_value_t = 6)]
    pub view_distance: i32,
    /// Simulated latency in milliseconds.
    #[arg(long, default_value_t = 0)]
    pub simulate_latency: u64,
    /// Simulated packet loss in percent.
    #[arg(long, default_value_t = 0)]
    pub simulate_loss: u8,
    /// Close the window after this many seconds; `0` runs until closed. Used by smoke tests.
    #[arg(long, default_value_t = 0)]
    pub exit_after: u64,
}

/// Content loaded at startup.
#[derive(Resource)]
pub struct Content {
    /// The validated registry.
    pub registry: Registry,
    /// Content hash, checked against the server on connect.
    pub hash: u64,
}

/// World seed, used by the local preview world generator.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Seed(pub u64);

/// Frame timing, refreshed once a second for the F3 overlay.
#[derive(Resource, Default, Debug)]
pub struct FrameStats {
    /// Frames counted in the current window.
    frames: u32,
    /// Seconds elapsed in the current window.
    seconds: f32,
    /// Smoothed frames per second.
    pub fps: f32,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let content_dir =
        cli.content.clone().map_or_else(default_content_dir, std::path::PathBuf::from);
    let (registry, loaded) = load_dir(&content_dir).map_err(|errors| {
        let joined: Vec<String> = errors.iter().map(ToString::to_string).collect();
        anyhow::anyhow!("content failed to load:\n  {}", joined.join("\n  "))
    })?;

    let view_distance = cli.view_distance.clamp(1, 16);
    let seed = cli.seed;
    let exit_after = cli.exit_after;

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Deepcore".to_string(),
                    resolution: WindowResolution::new(1280, 720),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .insert_resource(Content { registry, hash: loaded.hash })
    .insert_resource(Seed(seed))
    .insert_resource(Cli { exit_after, ..cli })
    .insert_resource(ViewConfig { radius: view_distance, vertical: 4 })
    .init_resource::<ClientWorld>()
    .init_resource::<FrameStats>()
    .add_plugins((ChunkRenderPlugin, player::PlayerPlugin, hud::HudPlugin, input::InputPlugin))
    .add_systems(Startup, setup_environment)
    .add_systems(Update, (update_frame_stats, enforce_run_limit));

    app.run();
    Ok(())
}

/// Sun, sky and ambient light.
fn setup_environment(mut commands: Commands) {
    commands.spawn((
        DirectionalLight { illuminance: 11_000.0, shadow_maps_enabled: true, ..default() },
        Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::new(-0.4, -1.0, -0.3), Vec3::Y),
    ));
    // `AmbientLight` is a *camera override* in Bevy 0.20 (it requires `Camera`), so spawning it
    // as a free entity creates a stray camera that Bevy then picks as the UI camera. The global
    // resource is the correct way to light the whole scene.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.55, 0.62, 0.75),
        brightness: 260.0,
        ..default()
    });
    commands.insert_resource(ClearColor(Color::srgb(0.42, 0.56, 0.72)));
}

/// Tracks frame rate for the debug readout.
fn update_frame_stats(time: Res<Time>, mut stats: ResMut<FrameStats>) {
    stats.frames += 1;
    stats.seconds += time.delta_secs();
    if stats.seconds >= 1.0 {
        stats.fps = stats.frames as f32 / stats.seconds;
        stats.frames = 0;
        stats.seconds = 0.0;
    }
}

/// Closes the app after `--exit-after` seconds so smoke tests terminate on their own.
fn enforce_run_limit(cli: Res<Cli>, time: Res<Time>, mut exit: MessageWriter<AppExit>) {
    if cli.exit_after > 0 && time.elapsed_secs() >= cli.exit_after as f32 {
        info!("client run limit reached; exiting");
        exit.write(AppExit::Success);
    }
}
