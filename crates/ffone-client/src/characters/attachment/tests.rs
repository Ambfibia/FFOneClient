use crate::attachment::*;

#[test]
fn standard_slots_use_the_five_exact_socket_names() {
    assert!(
        LegacyPlayerAttachmentSlot::Hat
            .socket_path()
            .ends_with("Bip01 helmet01")
    );
    assert!(
        LegacyPlayerAttachmentSlot::Glasses
            .socket_path()
            .ends_with("Bip01 glass01")
    );
    assert!(
        LegacyPlayerAttachmentSlot::LeftPistol
            .socket_path()
            .ends_with("Bip01 Lweapon01")
    );
    assert_eq!(
        LegacyPlayerAttachmentSlot::RightPistol.socket_path(),
        LegacyPlayerAttachmentSlot::Zipline.socket_path()
    );
    assert!(
        LegacyPlayerAttachmentSlot::Back
            .socket_path()
            .ends_with("Bip01 back01")
    );
}

#[test]
fn shared_rig_socket_paths_include_the_exact_gender_root() {
    assert_eq!(
        player_attachment_socket_full_path(
            PlayerRigGender::Male,
            LegacyPlayerAttachmentSlot::RightPistol,
        ),
        "m/Bip01/Bip01 NonAccum/Bip01 Pelvis/Bip01 Spine/Bip01 Spine1/Bip01 Neck/Bip01 R Clavicle/Bip01 R UpperArm/Bip01 R Forearm/Bip01 R Hand/Bip01 Rweapon01"
    );
    assert_eq!(
        player_attachment_socket_full_path(
            PlayerRigGender::Female,
            LegacyPlayerAttachmentSlot::RightPistol,
        ),
        "w/Bip01/Bip01 NonAccum/Bip01 Pelvis/Bip01 Spine/Bip01 Spine1/Bip01 Neck/Bip01 R Clavicle/Bip01 R UpperArm/Bip01 R Forearm/Bip01 R Hand/Bip01 Rweapon01"
    );
}

#[test]
fn standard_attachment_has_no_origin_scale_or_character_yaw_added() {
    let placement = standard_player_attachment_placement();
    assert_eq!(placement.item_local.translation, Vec3::ZERO);
    assert_eq!(placement.item_local.scale, Vec3::ONE);
    assert_eq!(placement.socket_local_scale_override, Some(Vec3::ONE));
    assert!((placement.item_local.rotation * Vec3::Z).abs_diff_eq(Vec3::NEG_Y, 0.000_01));
    assert!(!(placement.item_local.rotation * Vec3::Z).abs_diff_eq(Vec3::NEG_Z, 0.000_01));
}

#[test]
fn npc_cosmetic_preserves_explicit_nonuniform_and_negative_scale() {
    let placement = scripted_npc_cosmetic_placement(
        Vec3::new(1.0, -0.05, 0.16),
        Vec3::new(102.0, 15.0, -7.0),
        Vec3::new(-2.55, -2.25, -2.65),
    )
    .unwrap();
    assert_eq!(
        placement.item_local.translation,
        Vec3::new(-1.0, -0.05, 0.16)
    );
    assert_eq!(placement.item_local.scale, Vec3::new(-2.55, -2.25, -2.65));
    assert_eq!(placement.socket_local_scale_override, None);
    assert_eq!(scripted_hidden_tail_scale(), Vec3::ZERO);
}

#[test]
fn npc_cosmetic_rotation_obeys_h_r_h_for_every_basis_vector() {
    let euler = Vec3::new(30.0, -45.0, 17.0);
    let unity = unity_quaternion_euler(euler);
    let native = scripted_npc_cosmetic_placement(Vec3::ZERO, euler, Vec3::ONE)
        .unwrap()
        .item_local
        .rotation;
    for basis in [Vec3::X, Vec3::Y, Vec3::Z] {
        assert!(
            (native * unity_to_native_vector(basis))
                .abs_diff_eq(unity_to_native_vector(unity * basis), 0.000_01)
        );
    }
}

#[test]
fn nonfinite_cosmetic_trs_is_rejected() {
    assert_eq!(
        scripted_npc_cosmetic_placement(Vec3::ZERO, Vec3::ZERO, Vec3::new(1.0, f32::NAN, 1.0)),
        Err(LegacyAttachmentTransformError::NonFinite)
    );
}
