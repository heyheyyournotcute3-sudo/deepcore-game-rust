//! Greedy chunk meshing.
//!
//! For each of the six faces we sweep the chunk slice by slice and merge runs of identical
//! blocks into one quad, skipping faces that are hidden by a neighbour. Neighbour lookups fall
//! back to the chunk store so faces between chunks are culled correctly.
//!
//! The output is a plain `Mesh` with positions, normals, UVs and indices, built on a worker
//! thread by the caller.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;

use deepcore_shared::voxel::{BlockId, CHUNK_SIZE, Chunk, ChunkPos, Dir};

/// Largest number of blocks a greedy quad may span on one axis.
///
/// Quads sample one atlas tile across their whole area, so an unbounded merge would smear the
/// texture. Eight keeps large floors cheap while the tiling stays readable.
pub const MAX_MERGE: i32 = 8;

/// A block lookup that can see outside the chunk.
pub trait Neighbour {
    /// Block at local chunk coordinates, which may be outside `0..32`.
    fn block(&self, x: i32, y: i32, z: i32) -> BlockId;
}

/// Neighbour view over a single chunk; outside is air.
///
/// Used by the tests and by anything that needs a quick standalone mesh.
#[cfg(test)]
pub struct SingleChunk<'a> {
    /// The chunk being meshed.
    pub chunk: &'a Chunk,
}

#[cfg(test)]
impl Neighbour for SingleChunk<'_> {
    fn block(&self, x: i32, y: i32, z: i32) -> BlockId {
        if x < 0
            || y < 0
            || z < 0
            || x >= CHUNK_SIZE as i32
            || y >= CHUNK_SIZE as i32
            || z >= CHUNK_SIZE as i32
        {
            return BlockId::AIR;
        }
        self.chunk.get(x as usize, y as usize, z as usize)
    }
}

/// Neighbour view over a chunk and its six neighbours, for seamless face culling.
///
/// The client uses this once chunk streaming lands (M2); [`SingleChunk`] is enough for the
/// local preview world.
#[allow(dead_code)]
pub struct ChunkNeighbourhood<'a> {
    /// The chunk being meshed.
    pub chunk: &'a Chunk,
    /// The chunk position, used to index the neighbour list.
    pub pos: ChunkPos,
    /// Neighbours in [`Dir::ALL`] order.
    pub neighbours: [Option<&'a Chunk>; 6],
}

impl Neighbour for ChunkNeighbourhood<'_> {
    fn block(&self, x: i32, y: i32, z: i32) -> BlockId {
        let size = CHUNK_SIZE as i32;
        if (0..size).contains(&x) && (0..size).contains(&y) && (0..size).contains(&z) {
            return self.chunk.get(x as usize, y as usize, z as usize);
        }
        // Work out which neighbour the coordinate falls into.
        let step = |v: i32| -> i32 {
            let below = v < 0;
            let above = v >= size;
            i32::from(above) - i32::from(below)
        };
        let (nx, ny, nz) = (step(x), step(y), step(z));
        let dir = match (nx, ny, nz) {
            (1, 0, 0) => Dir::PosX,
            (-1, 0, 0) => Dir::NegX,
            (0, 1, 0) => Dir::PosY,
            (0, -1, 0) => Dir::NegY,
            (0, 0, 1) => Dir::PosZ,
            (0, 0, -1) => Dir::NegZ,
            _ => return BlockId::AIR, // Diagonal: treat as air so corners still render.
        };
        let index = Dir::ALL.iter().position(|d| *d == dir).unwrap_or(0);
        let Some(neighbour) = self.neighbours[index] else {
            return BlockId::AIR;
        };
        let lx = x.rem_euclid(size) as usize;
        let ly = y.rem_euclid(size) as usize;
        let lz = z.rem_euclid(size) as usize;
        let _ = self.pos;
        neighbour.get(lx, ly, lz)
    }
}

/// A mesh built from a chunk, with the block ids needed to look up atlas tiles.
pub struct ChunkMesh {
    /// Positions, normals, UVs and indices.
    pub mesh: Mesh,
    /// For every quad, the block that produced it (used for UV assignment).
    pub quads: Vec<BlockId>,
}

/// True when a block hides the face of the block next to it.
fn is_opaque(block: BlockId, is_transparent: &impl Fn(BlockId) -> bool) -> bool {
    !block.is_air() && !is_transparent(block)
}

/// Builds a greedy mesh for a chunk.
///
/// `is_transparent` must answer from the registry so water and glass do not hide their
/// neighbours, and `uv_of` maps a block to its atlas rectangle.
pub fn build_mesh<N: Neighbour>(
    view: &N,
    is_transparent: &impl Fn(BlockId) -> bool,
    uv_of: &impl Fn(BlockId) -> [f32; 4],
) -> ChunkMesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut quads: Vec<BlockId> = Vec::new();

    let size = CHUNK_SIZE as i32;

    for dir in Dir::ALL {
        // For each face direction, sweep along the face normal.
        for slice in 0..size {
            // Mask of (block, mergeable) for the two in-plane axes.
            let mut mask: Vec<Option<BlockId>> = vec![None; (size * size) as usize];
            for b in 0..size {
                for a in 0..size {
                    let (x, y, z) = face_coords(&dir, a, b, slice);
                    let block = view.block(x, y, z);
                    if block.is_air() {
                        continue;
                    }
                    let neighbour = view.block(x + dir.dx(), y + dir.dy(), z + dir.dz());
                    let hidden = is_opaque(neighbour, is_transparent)
                        || (neighbour == block && is_transparent(block));
                    if !hidden {
                        mask[(b * size + a) as usize] = Some(block);
                    }
                }
            }

            // Greedy merge the mask into rectangles.
            for b in 0..size {
                let mut a = 0;
                while a < size {
                    let Some(block) = mask[(b * size + a) as usize] else {
                        a += 1;
                        continue;
                    };
                    // Extend along a, up to MAX_MERGE so the single atlas tile sampled per
                    // quad is not stretched beyond recognition.
                    let mut width = 1;
                    while a + width < size
                        && width < MAX_MERGE
                        && mask[(b * size + a + width) as usize] == Some(block)
                    {
                        width += 1;
                    }
                    // Extend along b.
                    let mut height = 1;
                    while b + height < size && height < MAX_MERGE {
                        let row_ok = (0..width)
                            .all(|w| mask[((b + height) * size + a + w) as usize] == Some(block));
                        if !row_ok {
                            break;
                        }
                        height += 1;
                    }
                    push_quad(
                        &dir,
                        a,
                        b,
                        width,
                        height,
                        slice,
                        block,
                        uv_of,
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut indices,
                        &mut quads,
                    );
                    for w in 0..width {
                        for h in 0..height {
                            mask[((b + h) * size + a + w) as usize] = None;
                        }
                    }
                    a += width;
                }
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    ChunkMesh { mesh, quads }
}

/// Maps `(a, b, slice)` to local chunk coordinates for a face direction.
fn face_coords(dir: &Dir, a: i32, b: i32, slice: i32) -> (i32, i32, i32) {
    match dir {
        Dir::PosX | Dir::NegX => (slice, b, a),
        Dir::PosY | Dir::NegY => (a, slice, b),
        Dir::PosZ | Dir::NegZ => (a, b, slice),
    }
}

#[allow(clippy::too_many_arguments)]
fn push_quad(
    dir: &Dir,
    a: i32,
    b: i32,
    width: i32,
    height: i32,
    slice: i32,
    block: BlockId,
    uv_of: &impl Fn(BlockId) -> [f32; 4],
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    quads: &mut Vec<BlockId>,
) {
    let base = positions.len() as u32;
    let n = dir.normal();
    let [u0, v0, du, dv] = uv_of(block);
    let af = a as f32;
    let bf = b as f32;
    let wf = width as f32;
    let hf = height as f32;
    // Offset the face to the block boundary so faces do not z-fight with neighbours.
    let off = dir.normal().max(Vec3::ZERO);

    // The four corners in the face plane, in (a, b) space.
    let corners = [(af, bf), (af + wf, bf), (af + wf, bf + hf), (af, bf + hf)];
    let mut verts = [Vec3::ZERO; 4];
    for (i, (ca, cb)) in corners.iter().enumerate() {
        let local = match dir {
            Dir::PosX | Dir::NegX => Vec3::new(slice as f32, *cb, *ca),
            Dir::PosY | Dir::NegY => Vec3::new(*ca, slice as f32, *cb),
            Dir::PosZ | Dir::NegZ => Vec3::new(*ca, *cb, slice as f32),
        };
        verts[i] = local + off;
    }

    for v in verts {
        positions.push([v.x, v.y, v.z]);
        normals.push([n.x, n.y, n.z]);
    }
    // The quad samples exactly one atlas tile; UVs must stay inside that tile so neighbouring
    // tiles never bleed in. (Upgrade path: a custom material with fract() UVs, see PROGRESS.md.)
    uvs.push([u0, v0]);
    uvs.push([u0 + du, v0]);
    uvs.push([u0 + du, v0 + dv]);
    uvs.push([u0, v0 + dv]);

    // Winding: counter-clockwise when seen from outside.
    let flip = matches!(dir, &Dir::NegX | &Dir::NegY | &Dir::NegZ);
    if flip {
        indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
    } else {
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    quads.push(block);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uv(_: BlockId) -> [f32; 4] {
        [0.0, 0.0, 0.0625, 0.0625]
    }

    fn opaque(b: BlockId) -> bool {
        b.is_air()
    }

    #[test]
    fn empty_chunk_has_no_geometry() {
        let chunk = Chunk::new();
        let mesh = build_mesh(&SingleChunk { chunk: &chunk }, &opaque, &uv);
        assert_eq!(mesh.quads.len(), 0);
    }

    #[test]
    fn single_block_has_six_faces() {
        let mut chunk = Chunk::new();
        chunk.set(5, 5, 5, BlockId(1));
        let mesh = build_mesh(&SingleChunk { chunk: &chunk }, &opaque, &uv);
        assert_eq!(mesh.quads.len(), 6, "an isolated block shows all six faces");
    }

    #[test]
    fn adjacent_blocks_hide_shared_faces() {
        let mut chunk = Chunk::new();
        chunk.set(5, 5, 5, BlockId(1));
        chunk.set(6, 5, 5, BlockId(1));
        let mesh = build_mesh(&SingleChunk { chunk: &chunk }, &opaque, &uv);
        // The shared face is culled and the remaining faces merge pairwise, so 6 quads cover
        // a 2x1x1 slab exactly.
        assert_eq!(mesh.quads.len(), 6, "two touching blocks merge into a slab");
    }

    #[test]
    fn greedy_merges_a_flat_plane() {
        // A 32x32 plane of blocks at y = 0 should merge into very few quads.
        let mut chunk = Chunk::new();
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                chunk.set(x, 0, z, BlockId(1));
            }
        }
        let mesh = build_mesh(&SingleChunk { chunk: &chunk }, &opaque, &uv);
        // 32/MAX_MERGE = 4 quads per side, so 4*4 top + 4*4 bottom + 4 side strips * 4 = 48.
        assert_eq!(mesh.quads.len(), 48, "a full plane merges into MAX_MERGE-sized quads");
    }

    #[test]
    fn greedy_quads_cover_the_whole_face_without_gaps() {
        // Regression: a merged quad must span its full width/height, and merged quads must not
        // leave gaps. We verify total coverage of the top face analytically.
        let mut chunk = Chunk::new();
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                chunk.set(x, 0, z, BlockId(1));
            }
        }
        let mesh = build_mesh(&SingleChunk { chunk: &chunk }, &opaque, &uv);
        let positions =
            mesh.mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3().unwrap().to_vec();
        let mut area = 0.0f32;
        for quad in positions.chunks(4) {
            if quad.iter().all(|p| (p[1] - 1.0).abs() < 1e-5) {
                let w = (quad[1][0] - quad[0][0]).abs();
                let h = (quad[2][2] - quad[1][2]).abs();
                area += w * h;
            }
        }
        assert!((area - 1024.0).abs() < 1e-3, "top face coverage was {area}, expected 1024");
    }

    #[test]
    fn full_chunk_merges_to_a_small_quad_count() {
        let chunk = Chunk::filled(BlockId(1));
        let mesh = build_mesh(&SingleChunk { chunk: &chunk }, &opaque, &uv);
        assert!(mesh.quads.len() <= 100, "greedy meshing must collapse a full chunk");
        assert_eq!(mesh.quads.len(), 96, "6 faces x 4x4 quads at MAX_MERGE 8");
    }

    #[test]
    fn neighbourhood_culls_faces_between_chunks() {
        let mut a = Chunk::filled(BlockId(1));
        let b = Chunk::filled(BlockId(1));
        // Remove one block so the mesher has work.
        a.set(0, 0, 0, BlockId::AIR);
        let view = ChunkNeighbourhood {
            chunk: &a,
            pos: ChunkPos::new(0, 0, 0),
            neighbours: [Some(&b), None, None, None, None, None],
        };
        let mesh = build_mesh(&view, &opaque, &uv);
        // Chunk `a` is solid, so all six outer faces are emitted (merged into MAX_MERGE
        // quads: 96), and the +X face is *not* duplicated against the neighbour. The carved
        // hole adds three more faces.
        let solid = Chunk::filled(BlockId(1));
        let with_neighbour = {
            let view = ChunkNeighbourhood {
                chunk: &solid,
                pos: ChunkPos::new(0, 0, 0),
                neighbours: [Some(&b), None, None, None, None, None],
            };
            build_mesh(&view, &opaque, &uv).quads.len()
        };
        let alone = {
            let view = ChunkNeighbourhood {
                chunk: &solid,
                pos: ChunkPos::new(0, 0, 0),
                neighbours: [None; 6],
            };
            build_mesh(&view, &opaque, &uv).quads.len()
        };
        assert_eq!(alone, 96, "a solid chunk meshes to 96 quads");
        assert!(
            with_neighbour < alone,
            "a solid neighbour must remove faces: {with_neighbour} vs {alone}"
        );
        assert_eq!(
            with_neighbour + 16,
            alone,
            "the +X face is 16 merged quads and must be fully culled"
        );
        // The carved chunk still meshes, including its hole.
        assert!(mesh.quads.len() >= 90);
    }
}
