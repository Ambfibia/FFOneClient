use super::*;

pub(super) fn project_asset(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .join(path.split('/').collect::<PathBuf>())
}
