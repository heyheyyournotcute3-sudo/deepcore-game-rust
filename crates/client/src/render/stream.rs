//! Chunk rendering: keeps a mesh per loaded chunk, regenerating only chunks that changed, and
//! streams chunks in around the player.
//!
//! Meshing itself is CPU work that belongs off the main thread (M2 moves it to
//! `AsyncComputeTaskPool`); the M1 preview world is small enough to mesh inline.

use bevy::prelude::*;

use deepcore_shared::voxel::{BlockId, ChunkPos};
use deepcore_shared::worldgen::WorldGen;

use crate::Content;
use crate::player::LocalPlayer;
use crate::render::mesh::{ChunkNeighbourhood, build_mesh};
use crate::world::{ClientWorld, chunk_origin};

/// Chunks whose meshes are stale and must be rebuilt.
#[derive(Resource, Default)]
pub struct DirtyChunks(pub Vec<ChunkPos>);

/// One mesh entity per rendered chunk.
#[derive(Resource, Default)]
pub struct ChunkMeshes(pub std::collections::HashMap<ChunkPos, Entity>);

/// Render distance in chunks.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ViewConfig {
    /// Horizontal radius in chunks.
    pub radius: i32,
    /// Vertical radius in chunks.
    pub vertical: i32,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self { radius: 6, vertical: 4 }
    }
}

/// The world material.
#[derive(Resource, Clone)]
pub struct WorldMaterial(pub Handle<StandardMaterial>);

/// Registers chunk rendering.
pub struct ChunkRenderPlugin;

impl Plugin for ChunkRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DirtyChunks>()
            .init_resource::<ChunkMeshes>()
            .init_resource::<ViewConfig>()
            .add_systems(Startup, setup_material)
            .add_systems(Update, (stream_chunks, rebuild_dirty_chunks).chain());
    }
}

/// Creates the shared terrain material from the atlas.
fn setup_material(
    mut commands: Commands,
    content: Res<Content>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let atlas = crate::render::build_atlas(&content.registry, &mut images);
    // Double-sided: the player sees the inside of tunnels, and greedy meshing emits one quad
    // per visible face.
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(atlas.image.clone()),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        cull_mode: None,
        ..default()
    });
    info!("atlas {}px for {} blocks", atlas.size, content.registry.blocks.len());
    commands.insert_resource(WorldMaterial(material));
    commands.insert_resource(atlas);
}

/// Generates chunks around the player and marks newly available chunks for meshing.
///
/// Streaming is incremental and bounded per frame: only the chunks nearest the player are
/// generated, a few per frame, so the main thread keeps rendering the HUD and handling input
/// while the view distance fills in.
fn stream_chunks(
    content: Res<Content>,
    view: Res<ViewConfig>,
    seed: Res<crate::Seed>,
    mut world: ResMut<ClientWorld>,
    mut dirty: ResMut<DirtyChunks>,
    player: Single<&LocalPlayer>,
    mut generated: Local<u32>,
    mut cursor: Local<usize>,
) {
    /// Chunks generated per frame.
    const BUDGET: usize = 2;

    let centre =
        ChunkPos::from_block(deepcore_shared::voxel::BlockPos::from_world(player.body.position));
    let generator = WorldGen::new(seed.0, content.registry.clone());

    // Build the wanted set once per frame; it is small (a few hundred entries).
    let mut wanted: Vec<(i32, ChunkPos)> = Vec::new();
    for dy in -view.vertical..=view.vertical {
        for dz in -view.radius..=view.radius {
            for dx in -view.radius..=view.radius {
                let pos = ChunkPos::new(centre.x + dx, centre.y + dy, centre.z + dz);
                if pos.in_bounds() && world.chunk(pos).is_none() {
                    wanted.push((pos.distance_squared(centre), pos));
                }
            }
        }
    }
    if wanted.is_empty() {
        *cursor = 0;
        return;
    }
    wanted.sort_unstable();
    *cursor = 0;

    let mut new_chunks = 0;
    for (_, pos) in wanted.iter().take(BUDGET) {
        if world.generate(&generator, *pos) {
            new_chunks += 1;
            dirty.0.push(*pos);
            for neighbour in pos.neighbours() {
                dirty.0.push(neighbour);
            }
        }
    }
    if new_chunks > 0 {
        *generated += new_chunks;
        debug!(
            "streamed {new_chunks} chunks ({} total, {} loaded, {} pending)",
            *generated,
            world.len(),
            wanted.len()
        );
    }
}

/// Rebuilds the meshes of chunks that changed, and of their neighbours when faces open up.
fn rebuild_dirty_chunks(
    mut commands: Commands,
    content: Res<Content>,
    world: Res<ClientWorld>,
    material: Res<WorldMaterial>,
    mut dirty: ResMut<DirtyChunks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_meshes: ResMut<ChunkMeshes>,
    atlas: Res<crate::render::atlas::Atlas>,
) {
    if dirty.0.is_empty() {
        return;
    }
    let transparent: Vec<bool> =
        content.registry.blocks.iter().map(|b| b.def.transparent).collect();
    let is_transparent = move |b: BlockId| transparent.get(b.0 as usize).copied().unwrap_or(false);
    let atlas_ref = atlas.clone();
    let uv_of = move |b: BlockId| atlas_ref.uv_for(b);

    let mut done: Vec<ChunkPos> = Vec::new();
    for pos in dirty.0.drain(..) {
        if done.contains(&pos) {
            continue;
        }
        done.push(pos);
        let Some(chunk) = world.chunk(pos) else {
            continue;
        };
        let neighbours = [
            world.chunk(pos.offset_dir(deepcore_shared::voxel::Dir::PosX)),
            world.chunk(pos.offset_dir(deepcore_shared::voxel::Dir::NegX)),
            world.chunk(pos.offset_dir(deepcore_shared::voxel::Dir::PosY)),
            world.chunk(pos.offset_dir(deepcore_shared::voxel::Dir::NegY)),
            world.chunk(pos.offset_dir(deepcore_shared::voxel::Dir::PosZ)),
            world.chunk(pos.offset_dir(deepcore_shared::voxel::Dir::NegZ)),
        ];
        let view = ChunkNeighbourhood { chunk, pos, neighbours };
        let built = build_mesh(&view, &is_transparent, &uv_of);

        let existing = chunk_meshes.0.get(&pos).copied();
        if built.quads.is_empty() {
            // An empty chunk needs no entity; drop any stale mesh.
            if let Some(entity) = existing {
                commands.entity(entity).despawn();
                chunk_meshes.0.remove(&pos);
            }
            continue;
        }
        if let Some(entity) = existing {
            // Swap in the new mesh; the old mesh asset is dropped with the component.
            commands.entity(entity).insert((
                Mesh3d(meshes.add(built.mesh)),
                MeshMaterial3d(material.0.clone()),
                Transform::from_translation(chunk_origin(pos)),
            ));
        } else {
            let entity = commands
                .spawn((
                    Mesh3d(meshes.add(built.mesh)),
                    MeshMaterial3d(material.0.clone()),
                    Transform::from_translation(chunk_origin(pos)),
                ))
                .id();
            chunk_meshes.0.insert(pos, entity);
        }
    }
}
