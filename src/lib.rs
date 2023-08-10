use std::path::PathBuf;

pub mod remark;
pub mod render;
pub mod utils;

pub const DEFAULT_KIND_FILTER: &[&str] = &["FastISelFailure", "NeverInline", "SpillReloadCopies"];

/// Directory containing Rust sources
pub struct RustcSourceRoot(pub PathBuf);

impl RustcSourceRoot {
    pub fn from_sysroot(path: PathBuf) -> anyhow::Result<Self> {
        let src_dir = path.join("lib").join("rustlib").join("src").join("rust");
        if src_dir.is_dir() {
            Ok(Self(src_dir))
        } else {
            Err(anyhow::anyhow!("Path {} does not exist", src_dir.display()))
        }
    }
}
