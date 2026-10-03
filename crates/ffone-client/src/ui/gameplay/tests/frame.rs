use super::*;

#[test]
fn combat_frame_hides_for_live_tutorial_events_and_returns_during_active_combat() {
    use crate::avatar_action::LegacyAvatarActionContext;
    use crate::ui::gameplay::combat_frame::{CombatFrame, bind_combat_frame};
    let mut app = App::new();
    app.init_resource::<Time<Real>>()
        .insert_resource(GameplayUiModel {
            visible: true,
            ..default()
        })
        .add_systems(Update, bind_combat_frame);
    let player = app
        .world_mut()
        .spawn(LegacyAvatarActionContext::default())
        .id();
    let frame = app
        .world_mut()
        .spawn((CombatFrame, Node::default(), ImageNode::default()))
        .id();
    for (combat, event, expected) in [
        (false, false, Display::None),
        (true, false, Display::Flex),
        (true, true, Display::None),
        (true, false, Display::Flex),
        (false, false, Display::None),
    ] {
        let mut context = app
            .world_mut()
            .get_mut::<LegacyAvatarActionContext>(player)
            .unwrap();
        context.combat_condition = combat;
        context.tutorial_event = event;
        app.update();
        assert_eq!(app.world().get::<Node>(frame).unwrap().display, expected);
    }
}

#[test]
fn combat_frame_uses_the_clean_one_hertz_realtime_alpha_pulse() {
    assert!((combat_frame_alpha(0.0) - 0.5).abs() <= f32::EPSILON);
    assert!((combat_frame_alpha(0.25) - 1.0).abs() <= f32::EPSILON);
    assert!(combat_frame_alpha(0.75).abs() <= f32::EPSILON);
    assert!((combat_frame_alpha(1.0) - 0.5).abs() <= 0.000_001);
    assert_eq!(combat_frame_alpha(f32::NAN), 0.0);
}

#[test]
fn combat_frame_texture_matches_primary_gameframe_path_id_384() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .join(COMBAT_FRAME_PATH);
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert_eq!(bytes.len() as u64, COMBAT_FRAME_BYTES);
    assert_eq!(
        format!("{:x}", sha2::Sha256::digest(bytes)),
        COMBAT_FRAME_SHA256
    );
}

#[test]
fn nano_skill_target_frame_matches_the_primary_print_name_path_id_513_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .join(NANO_SKILL_TARGET_ICON_PATH);
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert_eq!(bytes.len() as u64, NANO_SKILL_TARGET_ICON_BYTES);
    assert_eq!(
        format!("{:x}", sha2::Sha256::digest(bytes)),
        NANO_SKILL_TARGET_ICON_SHA256
    );
    assert_eq!(NANO_SKILL_TARGET_ICON_SIZE, Vec2::new(74.0, 80.0));
    assert_ne!(
        NANO_SKILL_TARGET_ICON_PATH, "ui/en/gameplay/shared/skill_target_01_dds.png",
        "Effects.resourceFile's similarly named VFX texture is not PrintName.SkillIcon"
    );
}
