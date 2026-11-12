//! Deepcore client.
//!
//! Windowed game: first-person view of the voxel mine, procedural textures, and (from M2) the
//! network connection to a server. The client predicts only its own movement and shows
//! everything else as the server reports it.

mod render;

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use clap::Parser;

use deepcore_shared::Registry;
use deepcore_shared::content::{default_content_dir, load_dir};
use deepcore_shared::voxel::{BlockId, CHUNK_SIZE, Chunk, ChunkPos};
use deepcore_shared::worldgen::WorldGen;

use render::{SingleChunk, build_atlas, build_mesh};

/// Command line options for the client.
#[derive(Debug, Parser, Resource)]
#[command(name = "client", about = "Deepcore client")]
pub struct Cli {
    /// Server address to join, e.g. `127.0.0.1:47000`. Empty starts a local preview world.
    #[arg(long, default_value = "")]
    pub connect: String,
    /// World seed for the local preview world.
    #[arg(long, default_value_t = 0x0D33C0)]
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

/// Preview chunks generated locally before a network session exists.
#[derive(Resource, Default)]
pub struct PreviewWorld {
    /// Loaded chunks by position.
    pub chunks: HashMap<ChunkPos, Chunk>,
}

/// Frame timing readout, refreshed once a second.
#[derive(Resource, Default)]
struct FrameStats {
    frames: u32,
    seconds: f32,
    fps: f32,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let content_dir =
        cli.content.clone().map_or_else(default_content_dir, std::path::PathBuf::from);
    let (registry, loaded) = load_dir(&content_dir).map_err(|errors| {
        let joined: Vec<String> = errors.iter().map(ToString::to_string).collect();
        anyhow::anyhow!("content failed to load:\n  {}", joined.join("\n  "))
    })?;

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
    .insert_resource(cli)
    .init_resource::<PreviewWorld>()
    .init_resource::<FrameStats>()
    .add_systems(Startup, (setup_world, setup_camera).chain())
    .add_systems(Update, (update_frame_stats, enforce_run_limit));

    app.run();
    Ok(())
}

/// Builds the atlas, the world material and a small preview world around the spawn.
fn setup_world(
    mut commands: Commands,
    content: Res<Content>,
    cli: Res<Cli>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut preview: ResMut<PreviewWorld>,
) {
    let atlas = build_atlas(&content.registry, &mut images);
    // Double-sided: from inside a tunnel the player sees the back of the surrounding faces,
    // and greedy meshing emits one quad per visible face, so back faces must render.
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(atlas.image.clone()),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        cull_mode: None,
        ..default()
    });

    let generator = WorldGen::new(cli.seed, content.registry.clone());
    let spawn = generator.spawn_point();
    let centre = ChunkPos::from_block(deepcore_shared::voxel::BlockPos::from_world(spawn));

    let transparent: Vec<bool> =
        content.registry.blocks.iter().map(|b| b.def.transparent).collect();
    let is_transparent = move |b: BlockId| transparent.get(b.0 as usize).copied().unwrap_or(false);
    let atlas_for_uv = atlas.clone();
    let uv_of = move |b: BlockId| atlas_for_uv.uv_for(b);

    // Generate and mesh a small area around the camp so the first frame is not empty.
    let radius = cli.view_distance.min(3);
    let mut mesh_count = 0;
    for dy in -1..=0 {
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let pos = ChunkPos::new(centre.x + dx, centre.y + dy, centre.z + dz);
                if !pos.in_bounds() {
                    continue;
                }
                let chunk = generator.generate(pos);
                let mesh = build_mesh(&SingleChunk { chunk: &chunk }, &is_transparent, &uv_of);
                if !mesh.quads.is_empty() {
                    commands.spawn((
                        Mesh3d(meshes.add(mesh.mesh)),
                        MeshMaterial3d(material.clone()),
                        Transform::from_translation(Vec3::new(
                            (pos.x * CHUNK_SIZE as i32) as f32,
                            (pos.y * CHUNK_SIZE as i32) as f32,
                            (pos.z * CHUNK_SIZE as i32) as f32,
                        )),
                    ));
                    mesh_count += 1;
                }
                preview.chunks.insert(pos, chunk);
            }
        }
    }

    info!(
        "atlas {}px for {} blocks, {mesh_count} preview chunk meshes, spawn {spawn:?}",
        atlas.size,
        content.registry.blocks.len()
    );

    commands.insert_resource(atlas);
}

/// Spawns the camera, the sun and the ambient light.
fn setup_camera(mut commands: Commands, content: Res<Content>, cli: Res<Cli>) {
    let generator = WorldGen::new(cli.seed, content.registry.clone());
    let spawn = generator.spawn_point();

    let eye = spawn + Vec3::new(-14.0, 9.0, 18.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(eye).looking_at(spawn + Vec3::new(0.0, -2.0, 0.0), Vec3::Y),
    ));

    // Sunlight: the surface camp must be clearly lit.
    commands.spawn((
        DirectionalLight { illuminance: 12_000.0, shadow_maps_enabled: true, ..default() },
        Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::new(-0.4, -1.0, -0.3), Vec3::Y),
    ));
    // AmbientLight is a component in Bevy 0.20, not a resource.
    commands.spawn(AmbientLight {
        color: Color::srgb(0.55, 0.62, 0.75),
        brightness: 220.0,
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
        debug!("{:.0} fps", stats.fps);
        stats.frames = 0;
        stats.seconds = 0.0;
    }
}

/// Closes the app after `--exit-after` seconds so smoke tests can run headless-ish.
fn enforce_run_limit(cli: Res<Cli>, time: Res<Time>, mut exit: MessageWriter<AppExit>) {
    if cli.exit_after > 0 && time.elapsed_secs() >= cli.exit_after as f32 {
        info!("client run limit reached; exiting");
        exit.write(AppExit::Success);
    }
}
