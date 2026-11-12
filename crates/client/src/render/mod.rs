//! Client rendering: texture atlas, chunk meshing and the world view.

pub mod atlas;
pub mod mesh;

pub use atlas::build_atlas;

pub mod stream;

pub use stream::{ChunkRenderPlugin, DirtyChunks, ViewConfig};
