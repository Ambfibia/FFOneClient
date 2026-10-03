use super::*;

pub(super) fn asset_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .to_path_buf()
}
