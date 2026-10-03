use super::*;
#[test]
fn coco_server_reply_selects_skill_or_buff_icon_and_localizes_both_languages() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = crate::assets::AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let (localization, en) = Localization::open(&root, "en").unwrap();
    let (_, ru) = Localization::open(&root, "ru").unwrap();
    let mut queue = RewardNotices::default();
    let reply = ShinyPickupSuccess0104 {
        skill_id: 183,
        cstb: 0,
    };
    let en_text = queue
        .receive_shiny(&reply, &content, &localization, &en)
        .unwrap();
    let ru_text = queue
        .receive_shiny(&reply, &content, &localization, &ru)
        .unwrap();
    assert_ne!(
        localization.text(&en, &en_text),
        localization.text(&ru, &ru_text)
    );
    assert!(!localization.text(&ru, &ru_text).contains('{'));
    assert!(root.join(&queue.shiny[0].icon).is_file());
    let buff = content
        .gameplay_skill_buffs()
        .find(|v| v.buff_id > 0)
        .unwrap();
    queue
        .receive_shiny(
            &ShinyPickupSuccess0104 {
                cstb: buff.buff_id,
                ..reply
            },
            &content,
            &localization,
            &en,
        )
        .unwrap();
    assert_eq!(
        queue.shiny.back().unwrap().icon,
        format!("icons/skills/skillicon_{:02}.png", buff.icon_number)
    );
    assert!(
        queue
            .receive_shiny(
                &ShinyPickupSuccess0104 {
                    skill_id: -1,
                    cstb: 0
                },
                &content,
                &localization,
                &en
            )
            .is_none()
    );
    assert_eq!(queue.shiny.len(), 3);
    queue.advance(4.01);
    assert_eq!(queue.shiny.len(), 2);
    assert_eq!(queue.shiny.front().unwrap().remaining, 2.0);
    queue.clear();
    assert!(queue.shiny.is_empty());
    let en_bundle: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("localization/en.json")).unwrap()).unwrap();
    let ru_bundle: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("localization/ru.json")).unwrap()).unwrap();
    assert_eq!(
        en_bundle["entries"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ru_bundle["entries"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>()
    );
    for key in [
        "ui.gameplay.reward.coco_title",
        "ui.gameplay.reward.coco_received",
    ] {
        assert!(
            en_bundle["entries"][key]
                .as_str()
                .unwrap()
                .contains("{bonus}")
        );
        assert!(
            ru_bundle["entries"][key]
                .as_str()
                .unwrap()
                .contains("{bonus}")
        );
    }
}

#[test]
fn coco_notice_starts_below_screen_settles_and_fades_at_source_times() {
    let size = Vec2::new(1024.0, 768.0);
    assert!((pose(2.0, size, 1.0, false).0.y - 768.0).abs() < 0.001);
    assert_eq!(pose(0.0, size, 1.0, false), (Vec2::new(423.0, 633.0), 1.0));
    assert_eq!(pose(-1.5, size, 1.0, true), (Vec2::new(423.0, 593.0), 0.5));
    assert_eq!(pose(-2.0, size, 1.0, false).1, 0.0);
}
