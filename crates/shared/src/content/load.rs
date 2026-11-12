//! Loading content from `assets/data/*.ron` and hashing it for the handshake.

use std::path::{Path, PathBuf};

use super::defs::ContentSet;
use super::registry::{ContentError, ErrorKind, Registry};

/// The content files, in the order they are hashed.
pub const CONTENT_FILES: [&str; 8] = [
    "blocks.ron",
    "items.ron",
    "machines.ron",
    "recipes.ron",
    "tech_tree.ron",
    "layers.ron",
    "fuels.ron",
    "hazards.ron",
];

/// Raw text of each content file plus the loaded [`ContentSet`].
#[derive(Debug, Clone)]
pub struct LoadedContent {
    /// The parsed content.
    pub set: ContentSet,
    /// A stable hash over every file's contents. Sent during the handshake.
    pub hash: u64,
    /// Directory the files came from.
    pub dir: PathBuf,
}

/// Loads and validates all content from a directory.
///
/// # Errors
/// Returns a list of every validation problem found, each with a readable message.
pub fn load_dir(dir: impl AsRef<Path>) -> Result<(Registry, LoadedContent), Vec<ContentError>> {
    let dir = dir.as_ref();
    let mut texts = Vec::new();
    for name in CONTENT_FILES {
        let path = dir.join(name);
        match std::fs::read_to_string(&path) {
            Ok(text) => texts.push((name, text)),
            Err(e) => {
                return Err(vec![ContentError {
                    kind: ErrorKind::Malformed,
                    message: format!("cannot read {}: {e}", path.display()),
                }]);
            }
        }
    }
    let set = parse_set(&texts)?;
    let hash = hash_texts(&texts);
    let registry = Registry::build(set.clone())?;
    Ok((registry, LoadedContent { set, hash, dir: dir.to_path_buf() }))
}

/// Parses the file texts into a [`ContentSet`] without resolving references.
///
/// # Errors
/// Returns a parse error naming the offending file.
pub fn parse_set(texts: &[(&str, String)]) -> Result<ContentSet, Vec<ContentError>> {
    let mut set = ContentSet::default();
    for (name, text) in texts {
        let err = |e: ron::error::SpannedError| ContentError {
            kind: ErrorKind::Malformed,
            message: format!("{name}: {e}"),
        };
        match *name {
            "blocks.ron" => set.blocks = ron::from_str(text).map_err(|e| vec![err(e)])?,
            "items.ron" => set.items = ron::from_str(text).map_err(|e| vec![err(e)])?,
            "machines.ron" => set.machines = ron::from_str(text).map_err(|e| vec![err(e)])?,
            "recipes.ron" => set.recipes = ron::from_str(text).map_err(|e| vec![err(e)])?,
            "tech_tree.ron" => set.tech = ron::from_str(text).map_err(|e| vec![err(e)])?,
            "layers.ron" => set.layers = ron::from_str(text).map_err(|e| vec![err(e)])?,
            "fuels.ron" => set.fuels = ron::from_str(text).map_err(|e| vec![err(e)])?,
            "hazards.ron" => set.hazards = ron::from_str(text).map_err(|e| vec![err(e)])?,
            other => {
                return Err(vec![ContentError {
                    kind: ErrorKind::Malformed,
                    message: format!("unexpected content file `{other}`"),
                }]);
            }
        }
    }
    Ok(set)
}

/// FNV-1a hash over the concatenated file contents, order-stable.
#[must_use]
pub fn hash_texts(texts: &[(&str, String)]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for (name, text) in texts {
        for byte in name.bytes().chain(text.bytes()) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
    }
    hash
}

/// Returns the content directory next to the executable, or the workspace `assets/data`.
///
/// This makes `cargo run` work from anywhere in the workspace and keeps a shipped build
/// self-contained.
#[must_use]
pub fn default_content_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("DEEPCORE_ASSETS") {
        return PathBuf::from(dir).join("data");
    }
    let cwd = PathBuf::from("assets/data");
    if cwd.is_dir() {
        return cwd;
    }
    // Walk up from the current directory looking for assets/data (works from crate dirs).
    let mut probe = std::env::current_dir().unwrap_or_default();
    for _ in 0..4 {
        let candidate = probe.join("assets/data");
        if candidate.is_dir() {
            return candidate;
        }
        if !probe.pop() {
            break;
        }
    }
    PathBuf::from("assets/data")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_and_order_sensitive() {
        let a = vec![("a.ron", "hello".to_string())];
        let b = vec![("a.ron", "hello".to_string())];
        let c = vec![("a.ron", "hellp".to_string())];
        assert_eq!(hash_texts(&a), hash_texts(&b));
        assert_ne!(hash_texts(&a), hash_texts(&c));
    }

    #[test]
    fn malformed_ron_is_reported_with_the_file_name() {
        let texts = vec![
            ("blocks.ron", "not ron at all".to_string()),
            ("items.ron", "[]".to_string()),
            ("machines.ron", "[]".to_string()),
            ("recipes.ron", "[]".to_string()),
            ("tech_tree.ron", "[]".to_string()),
            ("layers.ron", "[]".to_string()),
            ("fuels.ron", "[]".to_string()),
            ("hazards.ron", "[]".to_string()),
        ];
        let errs = parse_set(&texts).unwrap_err();
        assert!(errs[0].message.contains("blocks.ron"));
    }
}
