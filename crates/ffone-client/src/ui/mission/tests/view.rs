use super::*;

#[test]
fn spawn_spree_uses_exact_computress_journal_portrait() {
    const COMPUTRESS_PORTRAIT_BYTES: usize = 7_452;
    const COMPUTRESS_PORTRAIT_SHA256: &str =
        "C0FF729C8704392D237A0820449230EA43467423624B5C39CA8F47BA312E4C85";
    let root = asset_root();
    let locator = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let mission = content.journal_entry(451, "Sector V").unwrap();

    assert_eq!(mission.journal_npc_type, 2555);
    assert_eq!(
        content.gameplay_npc_portrait_icon_path(mission.journal_npc_type),
        Some("icons/entities/npc/npcicon_49.png")
    );
    let bytes = include_bytes!("../../../../../../assets/game/icons/entities/npc/npcicon_49.png");
    assert_eq!(bytes.len(), COMPUTRESS_PORTRAIT_BYTES);
    assert_eq!(
        format!("{:X}", Sha256::digest(bytes)),
        COMPUTRESS_PORTRAIT_SHA256
    );
}
