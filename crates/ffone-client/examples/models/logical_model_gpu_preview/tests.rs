use super::*;

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn cli_accepts_canonical_candidate_paths_and_limits() {
    let parsed = PreviewConfig::parse(strings(&[
        "--asset-root",
        "target/logical-models-candidate",
        "--model",
        "models/npc/npc_dexter.glb",
        "--screenshot",
        "target/gpu/npc_dexter.png",
        "--animation",
        "Run",
        "--frames",
        "1200",
        "--timeout",
        "60.5",
    ]))
    .expect("valid CLI");
    let ParseOutcome::Run(config) = parsed else {
        panic!("expected runnable config");
    };
    assert_eq!(config.model, "models/npc/npc_dexter.glb");
    assert_eq!(config.animation.as_deref(), Some("Run"));
    assert_eq!(config.max_frames, 1200);
    assert_eq!(config.timeout, Duration::from_secs_f64(60.5));
}

#[test]
fn cli_rejects_bundle_absolute_and_parent_model_paths() {
    for model in [
        "../npc_dexter.glb",
        "D:/legacy/npc_dexter.glb",
        "models/npc/npc_dexter.unity3d",
    ] {
        let error = PreviewConfig::parse(strings(&[
            "--asset-root",
            "candidate",
            "--model",
            model,
            "--screenshot",
            "shot.png",
        ]))
        .expect_err("unsafe or non-GLB model path must fail");
        assert!(
            error.contains("relative") || error.contains("escape") || error.contains(".glb"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn cli_requires_all_three_io_paths() {
    let error = PreviewConfig::parse(strings(&["--asset-root", "candidate"]))
        .expect_err("missing model must fail");
    assert_eq!(error, "--model is required");
}

#[test]
fn explicit_material_true_names_override_generic_npc_roles_exactly() {
    use ffone_client::legacy_model_material::LegacyShaderKind;

    const MAIN: &str = "dt_etc_downtownbus_a_00-01 - default-dt_etc_downtownbus_a_01.dds";
    const SUB: &str = "dt_etc_downtownbus_a_00-02 - default-dt_etc_downtownbus_a_02.dds";

    assert_eq!(
        preview_npc_texture_role(
            MAIN,
            LegacyShaderKind::AlphaBlendNormalGlow,
            true,
            true,
            Some(MAIN),
            Some(SUB),
        ),
        Some(LegacyNpcTextureRole::Main)
    );
    assert_eq!(
        preview_npc_texture_role(
            SUB,
            LegacyShaderKind::AlphaBlendNormalGlow,
            true,
            true,
            Some(MAIN),
            Some(SUB),
        ),
        Some(LegacyNpcTextureRole::Sub)
    );
    assert_eq!(
        preview_npc_texture_role(
            "dt_etc_downtownbus_a_00-19 - default-car_for08.dds",
            LegacyShaderKind::SrcAlphaAdditiveBackface,
            true,
            true,
            Some(MAIN),
            Some(SUB),
        ),
        None
    );
    assert_eq!(
        preview_npc_texture_role(
            "main",
            LegacyShaderKind::OpaqueNormal,
            true,
            true,
            Some(MAIN),
            Some(SUB),
        ),
        None,
        "an explicit exact mapping must suppress the generic role for that slot"
    );
}

#[test]
fn cli_accepts_explicit_npc_runtime_policy_and_report() {
    let parsed = PreviewConfig::parse(strings(&[
        "--asset-root",
        "candidate",
        "--model",
        "models/npc/mob/npc_arturo/npc_smallturo.glb",
        "--screenshot",
        "target/runtime/npc_smallturo.png",
        "--animation-name",
        "stand1",
        "--character-kind",
        "npc",
        "--true-root",
        "npc_smallturo",
        "--npc-scale",
        "1.0",
        "--report",
        "target/runtime/npc_smallturo.json",
    ]))
    .expect("valid NPC runtime acceptance CLI");
    let ParseOutcome::Run(config) = parsed else {
        panic!("expected runnable config");
    };
    assert_eq!(
        config.character,
        Some(CharacterRuntimeConfig {
            kind: CharacterKind::Npc,
            true_root: "npc_smallturo".to_owned(),
            npc_scale: Some(1.0),
        })
    );
    assert_eq!(
        config.report,
        Some(PathBuf::from("target/runtime/npc_smallturo.json"))
    );
}

#[test]
fn cli_rejects_incomplete_or_cross_kind_character_policies() {
    let common = [
        "--asset-root",
        "candidate",
        "--model",
        "model.glb",
        "--screenshot",
        "shot.png",
    ];
    for extra in [
        vec!["--character-kind", "npc", "--true-root", "npc_smallturo"],
        vec![
            "--character-kind",
            "nano",
            "--true-root",
            "nano_coco",
            "--npc-scale",
            "1.35",
        ],
        vec!["--true-root", "nano_coco"],
    ] {
        let mut args = common.to_vec();
        args.extend(extra);
        PreviewConfig::parse(strings(&args))
            .expect_err("incomplete or cross-kind policy must fail closed");
    }
}

#[test]
fn runtime_policy_replaces_origin_once_and_preserves_only_the_typed_scale() {
    let authored = Transform::from_xyz(-0.740_397, 2.0, -3.0)
        .with_rotation(Quat::from_rotation_x(0.25))
        .with_scale(Vec3::splat(1.35));
    let visual = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI));

    let npc = CharacterRuntimeConfig {
        kind: CharacterKind::Npc,
        true_root: "npc_smallturo".to_owned(),
        npc_scale: Some(1.15),
    }
    .policy()
    .try_resolve_root(authored)
    .expect("valid NPC policy");
    assert_eq!(npc.translation, Vec3::ZERO);
    assert_eq!(npc.rotation, Quat::IDENTITY);
    assert_eq!(npc.scale, Vec3::splat(1.15));
    assert!(is_exact_character_runtime_chain(
        Transform::IDENTITY,
        visual,
        npc
    ));

    let nano = CharacterRuntimeConfig {
        kind: CharacterKind::Nano,
        true_root: "nano_coco".to_owned(),
        npc_scale: None,
    }
    .policy()
    .try_resolve_root(authored)
    .expect("valid Nano policy");
    assert_eq!(nano.translation, Vec3::ZERO);
    assert_eq!(nano.rotation, Quat::IDENTITY);
    assert_eq!(nano.scale, authored.scale);
    assert!(is_exact_character_runtime_chain(
        Transform::IDENTITY,
        visual,
        nano
    ));

    let player = CharacterRuntimeConfig {
        kind: CharacterKind::Player,
        true_root: "assembled_player".to_owned(),
        npc_scale: None,
    }
    .policy()
    .try_resolve_root(authored)
    .expect("valid Player policy");
    assert_eq!(player.translation, Vec3::ZERO);
    assert_eq!(player.rotation, Quat::IDENTITY);
    assert_eq!(player.scale, Vec3::ONE);
    assert!(is_exact_character_runtime_chain(
        Transform::IDENTITY,
        visual,
        player
    ));
}

#[test]
fn runtime_chain_rejects_a_second_turn_or_residual_imported_origin() {
    let visual = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI));
    let valid_root = Transform::from_scale(Vec3::splat(1.3));
    assert!(!is_exact_character_runtime_chain(
        Transform::IDENTITY,
        visual,
        valid_root.with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
    ));
    assert!(!is_exact_character_runtime_chain(
        Transform::IDENTITY,
        visual,
        valid_root.with_translation(Vec3::new(0.25, 0.0, 0.0))
    ));
}

#[test]
fn transformed_bounds_keep_rotation_scale_and_translation() {
    let aabb = Aabb::from_min_max(Vec3::new(-1.0, -2.0, -0.5), Vec3::new(1.0, 2.0, 0.5));
    let transform = GlobalTransform::from(
        Transform::from_xyz(4.0, -3.0, 2.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::new(2.0, 1.0, 3.0)),
    );
    let bounds = transformed_aabb(&aabb, &transform).expect("finite bounds");
    assert!(bounds.min.abs_diff_eq(Vec3::new(2.0, -5.0, 0.5), 1.0e-5));
    assert!(bounds.max.abs_diff_eq(Vec3::new(6.0, -1.0, 3.5), 1.0e-5));
}

#[test]
fn skinned_bounds_follow_bevy_joint_matrices_not_the_mesh_node_transform() {
    let point = skinned_vertex_world(
        [0.5, 2.0, 3.0],
        [0, 1, 0, 0],
        [0.25, 0.75, 0.0, 0.0],
        &[
            Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            Mat4::from_translation(Vec3::new(-2.0, 4.0, 0.0)),
        ],
    )
    .expect("valid weighted vertex");
    assert!(point.abs_diff_eq(Vec3::new(-0.5, 5.0, 3.0), 1.0e-6));

    // A glTF skinned mesh node can carry an authored transform which is
    // already represented by its inverse bind matrices. Bevy's shader
    // uses jointGlobal * inverseBind directly, so applying that node
    // transform again while framing the camera would be a double transform.
    let misleading_mesh_node = Transform::from_translation(Vec3::new(-5.960_093_5, 0.0, 0.0));
    assert!(!point.abs_diff_eq(misleading_mesh_node.transform_point(point), 1.0e-6));
}

#[test]
fn skinned_bounds_reject_invalid_joint_references() {
    let error = skinned_vertex_world(
        [0.0, 0.0, 0.0],
        [1, 0, 0, 0],
        [1.0, 0.0, 0.0, 0.0],
        &[Mat4::IDENTITY],
    )
    .expect_err("out-of-range joint must fail closed");
    assert!(error.contains("out of bounds"));
}

#[test]
fn bounds_union_and_camera_frame_are_finite() {
    let mut bounds = Bounds3::from_point(Vec3::new(-4.0, 1.0, 2.0)).unwrap();
    bounds.include_point(Vec3::new(8.0, 11.0, 5.0));
    bounds.include_point(Vec3::new(0.0, -3.0, -9.0));
    assert_eq!(bounds.min, Vec3::new(-4.0, -3.0, -9.0));
    assert_eq!(bounds.max, Vec3::new(8.0, 11.0, 5.0));

    let frame =
        CameraFrame::for_bounds(bounds, PreviewCameraView::Primary).expect("camera frame");
    assert!(frame.eye.is_finite());
    assert_eq!(frame.target, bounds.center());
    assert!(frame.near > 0.0);
    assert!(frame.far > frame.near);
}

#[test]
fn clear_frame_retry_policy_preserves_reverses_or_cycles_the_exact_camera_view() {
    assert_eq!(BlankCameraRetry::parse("same"), Ok(BlankCameraRetry::Same));
    assert_eq!(
        BlankCameraRetry::parse("opposite"),
        Ok(BlankCameraRetry::Opposite)
    );
    assert_eq!(
        BlankCameraRetry::parse("all-axes"),
        Ok(BlankCameraRetry::AllAxes)
    );
    assert!(BlankCameraRetry::parse("fallback").is_err());
    assert_eq!(
        PreviewCameraView::Primary.opposite(),
        PreviewCameraView::Reverse
    );
    assert_eq!(
        PreviewCameraView::Reverse.opposite(),
        PreviewCameraView::Primary
    );
    assert!(
        PreviewCameraView::Primary
            .direction()
            .abs_diff_eq(-PreviewCameraView::Reverse.direction(), 1.0e-6)
    );
    assert_eq!(
        PreviewCameraView::Reverse.next_diagnostic(),
        PreviewCameraView::PositiveX
    );
    assert_eq!(
        PreviewCameraView::PositiveX.opposite(),
        PreviewCameraView::NegativeX
    );
    assert_eq!(
        PreviewCameraView::NegativeZ.next_diagnostic(),
        PreviewCameraView::Primary
    );

    let bounds = Bounds3 {
        min: Vec3::new(-0.25, 0.25, -0.32),
        max: Vec3::new(0.25, 1.12, 0.02),
    };
    let primary =
        CameraFrame::for_bounds(bounds, PreviewCameraView::Primary).expect("primary frame");
    let reverse =
        CameraFrame::for_bounds(bounds, PreviewCameraView::Reverse).expect("reverse frame");
    assert_eq!(primary.target, reverse.target);
    assert_eq!(primary.near, reverse.near);
    assert_eq!(primary.far, reverse.far);
    assert!(
        (primary.eye - primary.target).abs_diff_eq(-(reverse.eye - reverse.target), 1.0e-6)
    );
}

#[test]
fn gate_uses_first_reached_limit() {
    assert_eq!(
        gate_limit(2, Duration::from_secs(10), 3, Duration::from_secs(10)),
        Some("wall-clock timeout")
    );
    assert_eq!(
        gate_limit(3, Duration::from_secs(9), 3, Duration::from_secs(10)),
        Some("frame limit reached")
    );
    assert_eq!(
        gate_limit(2, Duration::from_secs(9), 3, Duration::from_secs(10)),
        None
    );
}

#[test]
fn xdt_texture_without_a_matching_material_is_a_primary_unity_noop() {
    assert_eq!(texture_override_status(None, 0), "not_requested");
    assert_eq!(
        texture_override_status(Some("textures/requested.png"), 0),
        "no_matching_material_unity_noop"
    );
    assert_eq!(
        texture_override_status(Some("textures/requested.png"), 2),
        "bound"
    );
}

#[test]
fn render_error_filter_is_narrow_but_catches_shader_failures() {
    assert!(is_render_error(
        &Level::ERROR,
        "bevy_render::render_resource::pipeline_cache",
        "failed to process shader"
    ));
    assert!(is_render_error(
        &Level::WARN,
        "naga::valid",
        "WGSL shader validation error"
    ));
    assert!(!is_render_error(
        &Level::WARN,
        "bevy_asset",
        "ordinary asset warning"
    ));
}

#[test]
fn acceptance_error_filter_rejects_missing_gltf_texture_labels() {
    assert!(is_render_error(
        &Level::ERROR,
        "bevy_asset::server",
        "GLB does not contain Texture0"
    ));
    assert!(is_render_error(
        &Level::ERROR,
        "bevy_gltf::loader",
        "missing labeled asset Image2"
    ));
    assert!(is_render_error(
        &Level::ERROR,
        "unrelated_loader",
        "asset load failed for a required PNG"
    ));
}

#[test]
fn screenshot_gate_rejects_clear_only_pixels() {
    let rgb = [14_u8, 19, 27].repeat(40 * 40);
    let error =
        screenshot_foreground(&rgb, 40, 40, &[], None).expect_err("blank frame must fail");
    assert!(error.contains("only the clear background"));
}

#[test]
fn screenshot_gate_accepts_a_visible_center_silhouette() {
    let mut rgb = [14_u8, 19, 27].repeat(40 * 40);
    for y in 10..30 {
        for x in 14..26 {
            let offset = (y * 40 + x) * 3;
            rgb[offset..offset + 3].copy_from_slice(&[220, 80, 35]);
        }
    }
    let (foreground, coverage) =
        screenshot_foreground(&rgb, 40, 40, &[], None).expect("silhouette must pass");
    assert_eq!(foreground, 240);
    assert!((coverage - 0.15).abs() < f64::EPSILON);
}

#[test]
fn screenshot_gate_accepts_a_sparse_isolated_material_with_explicit_floor() {
    let mut rgb = [14_u8, 19, 27].repeat(40 * 40);
    for index in 0..16 {
        let offset = ((12 + index / 8) * 40 + 16 + index % 8) * 3;
        rgb[offset..offset + 3].copy_from_slice(&[210, 12, 24]);
    }
    assert!(screenshot_foreground(&rgb, 40, 40, &[], None).is_err());
    let (foreground, coverage) = screenshot_foreground(&rgb, 40, 40, &[], Some(16))
        .expect("an explicitly isolated tiny material must remain GPU-testable");
    assert_eq!(foreground, 16);
    assert!((coverage - 0.01).abs() < f64::EPSILON);
}

#[test]
fn screenshot_gate_retries_an_outline_only_silhouette() {
    let mut rgb = [14_u8, 19, 27].repeat(40 * 40);
    for y in 10..30 {
        for x in 14..26 {
            let offset = (y * 40 + x) * 3;
            rgb[offset..offset + 3].copy_from_slice(&[0, 0, 0]);
        }
    }
    let error = screenshot_foreground(&rgb, 40, 40, &[[0, 0, 0]], None)
        .expect_err("an unoccluded outline shell must be retried");
    assert!(error.contains("only the source outline pass"));
}

#[test]
fn screenshot_gate_accepts_surface_pixels_inside_an_outline() {
    let mut rgb = [14_u8, 19, 27].repeat(40 * 40);
    for y in 10..30 {
        for x in 14..26 {
            let offset = (y * 40 + x) * 3;
            let color = if x == 14 || x == 25 || y == 10 || y == 29 {
                [0, 0, 0]
            } else {
                [220, 80, 35]
            };
            rgb[offset..offset + 3].copy_from_slice(&color);
        }
    }
    let (foreground, coverage) = screenshot_foreground(&rgb, 40, 40, &[[0, 0, 0]], None)
        .expect("a resolved surface plus outline must pass");
    assert_eq!(foreground, 240);
    assert!((coverage - 0.15).abs() < f64::EPSILON);
}
