//! Procedural block texture atlas.
//!
//! No downloaded art: every block's tile is generated from its `color` and `pattern` fields in
//! `blocks.ron`. Tiles are 16×16 and packed into a square atlas, one row per 16 blocks.

use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use deepcore_shared::content::{Pattern, Registry};
use deepcore_shared::voxel::BlockId;

/// Size of one tile in pixels.
pub const TILE: u32 = 16;
/// Tiles per atlas row.
pub const ATLAS_COLUMNS: u32 = 16;

/// The generated atlas and the UV rectangle of each block.
#[derive(Resource, Clone)]
pub struct Atlas {
    /// The atlas image, ready for `StandardMaterial::base_color_texture`.
    pub image: Handle<Image>,
    /// Per block: `(u_min, v_min, u_size, v_size)` in `0..1`.
    pub uvs: Vec<[f32; 4]>,
    /// Atlas size in pixels.
    pub size: u32,
}

impl Atlas {
    /// UV rectangle for a block, falling back to the whole texture for unknown ids.
    #[must_use]
    pub fn uv_for(&self, block: BlockId) -> [f32; 4] {
        self.uvs.get(block.0 as usize).copied().unwrap_or([0.0, 0.0, 1.0, 1.0])
    }
}

/// Generates the atlas image for every block in the registry.
#[must_use]
pub fn build_atlas(registry: &Registry, images: &mut Assets<Image>) -> Atlas {
    let block_count = registry.blocks.len().max(1) as u32;
    let rows = block_count.div_ceil(ATLAS_COLUMNS).max(1);
    let size = ATLAS_COLUMNS * TILE;
    let height = rows * TILE;
    let mut pixels = vec![0u8; (size * height * 4) as usize];

    let mut uvs = Vec::with_capacity(registry.blocks.len());
    for (i, block) in registry.blocks.iter().enumerate() {
        let col = i as u32 % ATLAS_COLUMNS;
        let row = i as u32 / ATLAS_COLUMNS;
        let tile = render_tile(&block.def.pattern, block.def.color, block.def.emissive, i as u64);
        for ty in 0..TILE {
            for tx in 0..TILE {
                let px = col * TILE + tx;
                let py = row * TILE + ty;
                let src = ((ty * TILE + tx) * 4) as usize;
                let dst = ((py * size + px) * 4) as usize;
                pixels[dst..dst + 4].copy_from_slice(&tile[src..src + 4]);
            }
        }
        // Bevy images use a top-left UV origin, so row 0 is v = 0. Do not flip.
        let u0 = (col * TILE) as f32 / size as f32;
        let v0 = (row * TILE) as f32 / height as f32;
        uvs.push([u0, v0, TILE as f32 / size as f32, TILE as f32 / height as f32]);
    }

    let mut image = Image::new(
        Extent3d { width: size, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
        mag_filter: bevy::image::ImageFilterMode::Nearest,
        min_filter: bevy::image::ImageFilterMode::Nearest,
        address_mode_u: bevy::image::ImageAddressMode::ClampToEdge,
        address_mode_v: bevy::image::ImageAddressMode::ClampToEdge,
        ..Default::default()
    });

    Atlas { image: images.add(image), uvs, size }
}

/// Renders one 16×16 tile in RGBA8.
fn render_tile(pattern: &Pattern, color: (f32, f32, f32), emissive: f32, seed: u64) -> Vec<u8> {
    let mut out = vec![0u8; (TILE * TILE * 4) as usize];
    let base = [color.0, color.1, color.2];
    for y in 0..TILE {
        for x in 0..TILE {
            let n = hash_noise(seed, x, y);
            let mut rgb = base;
            match pattern {
                Pattern::Flat => {}
                Pattern::Speckle => {
                    let k = 0.88 + n * 0.24;
                    rgb = [base[0] * k, base[1] * k, base[2] * k];
                }
                Pattern::Blotch => {
                    // Coarse blotches from a low-frequency hash of the quarter tile.
                    let q = hash_noise(seed ^ 0x51, x / 4, y / 4);
                    let k = 0.80 + q * 0.45;
                    rgb = [base[0] * k, base[1] * k, base[2] * k];
                }
                Pattern::OreFleck(fleck) => {
                    // Bright flecks: about a fifth of the pixels, clustered in pairs.
                    if n > 0.78 {
                        rgb = [fleck.0, fleck.1, fleck.2];
                    } else {
                        let k = 0.85 + n * 0.3;
                        rgb = [base[0] * k, base[1] * k, base[2] * k];
                    }
                }
                Pattern::Strata => {
                    let band = ((y / 3) as f32 * 0.13 + n * 0.08) - 0.2;
                    rgb = [
                        (base[0] + band).clamp(0.0, 1.0),
                        (base[1] + band).clamp(0.0, 1.0),
                        (base[2] + band).clamp(0.0, 1.0),
                    ];
                }
                Pattern::Glow => {
                    let cx = (x as f32 - 7.5) / 8.0;
                    let cy = (y as f32 - 7.5) / 8.0;
                    let d = (cx * cx + cy * cy).sqrt().min(1.0);
                    let k = (1.0 - d) * 0.9 + 0.1 + n * 0.1;
                    rgb = [base[0] * k, base[1] * k, base[2] * k];
                }
            }
            // Emissive blocks are drawn brighter so they read as light sources.
            if emissive > 0.0 {
                let k = 1.0 + emissive * 0.9;
                rgb = [(rgb[0] * k).min(1.0), (rgb[1] * k).min(1.0), (rgb[2] * k).min(1.0)];
            }
            let i = ((y * TILE + x) * 4) as usize;
            out[i] = (rgb[0].clamp(0.0, 1.0) * 255.0) as u8;
            out[i + 1] = (rgb[1].clamp(0.0, 1.0) * 255.0) as u8;
            out[i + 2] = (rgb[2].clamp(0.0, 1.0) * 255.0) as u8;
            out[i + 3] = 255;
        }
    }
    out
}

/// Deterministic per-pixel noise in `0..1`.
fn hash_noise(seed: u64, x: u32, y: u32) -> f32 {
    let mut h = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(u64::from(x) << 32)
        .wrapping_add(u64::from(y));
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 29;
    (h & 0xFFFF) as f32 / 65535.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_has_opaque_pixels() {
        let tile = render_tile(&Pattern::Speckle, (0.5, 0.5, 0.5), 0.0, 1);
        assert_eq!(tile.len(), (TILE * TILE * 4) as usize);
        assert!(tile.chunks(4).all(|p| p[3] == 255), "all pixels opaque");
    }

    #[test]
    fn ore_flecks_differ_from_base() {
        let tile = render_tile(&Pattern::OreFleck((1.0, 0.0, 0.0)), (0.1, 0.1, 0.1), 0.0, 7);
        let bright = tile.chunks(4).filter(|p| p[0] > 200).count();
        assert!(bright > 0, "ore flecks must be visible");
    }

    #[test]
    fn noise_is_deterministic() {
        assert_eq!(hash_noise(3, 4, 5), hash_noise(3, 4, 5));
        assert_ne!(hash_noise(3, 4, 5), hash_noise(3, 5, 4));
    }
}
