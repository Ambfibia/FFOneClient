use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct PrimaryTextureContract {
    pub(super) path_id: i64,
    pub(super) route: &'static str,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) png_bytes: u64,
    pub(super) png_sha256: &'static str,
}

#[test]
fn all_new_user_equip_textures_are_exact_primary_png_publications() {
    assert_eq!(PRIMARY_MAIN_ARCHIVE_BYTES, 7_000_415);
    assert_eq!(
        PRIMARY_MAIN_ARCHIVE_SHA256,
        "59788201962b6a1737b114486c361fe74eef69f507d1d125ca3171377eec602f"
    );
    assert_eq!(PRIMARY_TEXTURES.len(), 41);
    assert_eq!(
        PRIMARY_TEXTURES
            .iter()
            .map(|texture| texture.path_id)
            .collect::<BTreeSet<_>>()
            .len(),
        PRIMARY_TEXTURES.len(),
        "primary PathIDs must be unique"
    );

    let asset_root = workspace_root().join("assets/game");
    for texture in PRIMARY_TEXTURES {
        let path = asset_root.join(texture.route);
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "read PathID {} {}: {error}",
                texture.path_id,
                path.display()
            )
        });
        assert_eq!(
            bytes.len() as u64,
            texture.png_bytes,
            "PathID {} byte count: {}",
            texture.path_id,
            path.display()
        );
        assert_eq!(
            sha256_lower(&bytes),
            texture.png_sha256,
            "PathID {} SHA-256: {}",
            texture.path_id,
            path.display()
        );
        assert_eq!(
            image::image_dimensions(&path).unwrap_or_else(|error| panic!(
                "decode PathID {} {}: {error}",
                texture.path_id,
                path.display()
            )),
            (texture.width, texture.height),
            "PathID {} dimensions: {}",
            texture.path_id,
            path.display()
        );
    }
}
