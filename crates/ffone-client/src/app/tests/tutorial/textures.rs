use super::*;

#[test]
fn tutorial_skybox_applies_the_authored_free_zone_texture_transforms() {
    // Clean Tutorial.resourceFile owns sky_Left_f/pathId 36 and
    // sky_Right_f/pathId 311. The publishing boundary intentionally maps
    // those legacy identities to stable semantic runtime paths.
    assert_eq!(
        LEGACY_FUTURE_SKY_TEXTURES[2],
        "map/shared/environment/skyboxes/future/left.png"
    );
    assert_eq!(
        LEGACY_FUTURE_SKY_TEXTURES[3],
        "map/shared/environment/skyboxes/future/right.png"
    );
    for face_index in 0..4 {
        let transform = legacy_sky_uv_transform(LegacySkyZone::Future, face_index);
        assert!(
            transform
                .transform_point2(Vec2::ZERO)
                .abs_diff_eq(Vec2::ZERO, 0.000_001)
        );
        assert!(
            transform
                .transform_point2(Vec2::ONE)
                .abs_diff_eq(Vec2::new(1.0, 2.0), 0.000_001)
        );
    }
    let top = legacy_sky_uv_transform(LegacySkyZone::Future, 4);
    for (source, expected) in [
        (Vec2::ZERO, Vec2::new(1.0, 0.0)),
        (Vec2::X, Vec2::ONE),
        (Vec2::ONE, Vec2::Y),
        (Vec2::Y, Vec2::ZERO),
    ] {
        assert!(
            top.transform_point2(source)
                .abs_diff_eq(expected, 0.000_001),
            "future top UV {source:?} did not rotate to {expected:?}"
        );
    }
    assert_eq!(
        legacy_sky_uv_transform(LegacySkyZone::Past, 0),
        Affine2::IDENTITY
    );
}
