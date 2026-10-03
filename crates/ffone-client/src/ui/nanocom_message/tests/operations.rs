use super::*;

pub(super) fn passive(request_id: u64) -> NanocomMessageRequest {
    NanocomMessageRequest::type_9_numbuh_two(request_id, "Numbuh Two", "Passive")
}

pub(super) fn buddy(request_id: u64) -> NanocomMessageRequest {
    NanocomMessageRequest::buddy_invite(request_id, format!("Buddy {request_id}"))
}

pub(super) fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.000_1,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn spawned_tree_localizes_every_text_and_preserves_source_style_metrics() {
    let asset_root = tempdir().unwrap().keep();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(NanocomMessageUiPlugin);
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.update();
    {
        let mut model = app.world_mut().resource_mut::<NanocomMessageUiModel>();
        model.enqueue_buddy_invite(13, "Dexter");
        model.set_expanded(true);
        while model.pop_sound().is_some() {}
    }
    app.update();

    let world = app.world_mut();
    let mut all_texts = world.query::<(
        (&TextFont, &LineHeight),
        Option<&LocalizedText>,
        Option<&NanocomGuiStyleRole>,
    )>();
    let all_texts = all_texts
        .iter(world)
        .map(|(font, localized, style)| (font.clone(), localized.cloned(), style.copied()))
        .collect::<Vec<_>>();
    assert_eq!(all_texts.len(), 7);
    assert!(all_texts.iter().all(|(_, localized, style)| {
        localized.as_ref().is_some_and(|text| !text.key.is_empty()) && style.is_some()
    }));

    let mut elements = world.query::<(
        &NanocomMessageElement,
        &NanocomGuiStyleRole,
        (&TextFont, &LineHeight),
        &LocalizedText,
    )>();
    let elements = elements
        .iter(world)
        .map(|(element, style, font, localized)| {
            (*element, *style, font.clone(), localized.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(elements.len(), 5);
    for (element, style, font, localized) in elements {
        match element {
            NanocomMessageElement::CompactTitle => {
                assert_eq!(style, NanocomGuiStyleRole::BigFont14);
                assert_eq!(
                    font.0.font_size.eval(Vec2::ZERO, 16.0),
                    NANOCOM_JEFFE_14_FONT_SIZE
                );
                assert_eq!((*font.1), LineHeight::Px(NANOCOM_JEFFE_14_LINE_HEIGHT));
                assert_eq!(localized.key, NANOCOM_COMPACT_TITLE_LOCALIZATION_KEY);
            }
            NanocomMessageElement::CompactBody => {
                assert_eq!(style, NanocomGuiStyleRole::MessageText);
                assert_eq!(
                    font.0.font_size.eval(Vec2::ZERO, 16.0),
                    NANOCOM_CHALET_SMALL_FONT_SIZE
                );
                assert_eq!((*font.1), LineHeight::Px(NANOCOM_CHALET_SMALL_LINE_HEIGHT));
                assert_eq!(localized.key, NANOCOM_COMPACT_BODY_LOCALIZATION_KEY);
            }
            NanocomMessageElement::ExpandedTitle => {
                assert_eq!(style, NanocomGuiStyleRole::MessageTitle);
                assert_eq!(
                    font.0.font_size.eval(Vec2::ZERO, 16.0),
                    NANOCOM_MESSAGE_TITLE_FONT_SIZE
                );
                assert_eq!((*font.1), LineHeight::Px(NANOCOM_MESSAGE_TITLE_LINE_HEIGHT));
                assert_eq!(localized.key, NANOCOM_EXPANDED_TITLE_LOCALIZATION_KEY);
            }
            NanocomMessageElement::ExpandedBody => {
                assert_eq!(style, NanocomGuiStyleRole::CenterBox2);
                assert_eq!(
                    font.0.font_size.eval(Vec2::ZERO, 16.0),
                    NANOCOM_CHALET_SMALL_FONT_SIZE
                );
                assert_eq!(localized.key, NANOCOM_INVITATION_LOCALIZATION_KEY);
            }
            NanocomMessageElement::ExpandedExpiration => {
                assert_eq!(style, NanocomGuiStyleRole::CenterBox2);
                assert_eq!(
                    font.0.font_size.eval(Vec2::ZERO, 16.0),
                    NANOCOM_CHALET_SMALL_FONT_SIZE
                );
                assert_eq!(localized.key, NANOCOM_EXPIRATION_LOCALIZATION_KEY);
            }
            _ => unreachable!("only text elements participate in this query"),
        }
    }

    let mut labels = world.query::<(
        &NanocomMessageButtonLabel,
        &NanocomGuiStyleRole,
        (&TextFont, &LineHeight),
        &LocalizedText,
    )>();
    let labels = labels
        .iter(world)
        .map(|(label, style, font, localized)| {
            (*label, *style, font.clone(), localized.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(labels.len(), 2);
    for (label, style, font, localized) in labels {
        assert_eq!(
            font.0.font_size.eval(Vec2::ZERO, 16.0),
            NANOCOM_JEFFE_14_FONT_SIZE
        );
        assert_eq!((*font.1), LineHeight::Px(NANOCOM_JEFFE_14_LINE_HEIGHT));
        assert_eq!(
            localized.key,
            match label.0 {
                NanocomMessageChoice::Accept => NANOCOM_ACCEPT_LOCALIZATION_KEY,
                NanocomMessageChoice::Decline => NANOCOM_DECLINE_LOCALIZATION_KEY,
            }
        );
        assert_eq!(
            style,
            match label.0 {
                NanocomMessageChoice::Accept => NanocomGuiStyleRole::Button,
                NanocomMessageChoice::Decline => NanocomGuiStyleRole::RedButton,
            }
        );
    }

    let mut buttons = world.query::<(&NanocomMessageButton, &Node)>();
    for (button, node) in buttons.iter(world) {
        assert_eq!(node.padding.left, px(10));
        assert_eq!(node.padding.right, px(6));
        assert_eq!(node.padding.bottom, px(6));
        assert_eq!(
            node.padding.top,
            px(match button.choice {
                NanocomMessageChoice::Accept => 3,
                NanocomMessageChoice::Decline => 4,
            })
        );
    }
}

#[test]
fn hidden_modal_icon_inherits_parent_and_cannot_escape_compact_panel() {
    let asset_root = tempdir().unwrap().keep();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(NanocomMessageUiPlugin);
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.update();
    {
        let mut model = app.world_mut().resource_mut::<NanocomMessageUiModel>();
        model.enqueue_type_9_numbuh_two(9, "Numbuh Two", "Come in, cadet.");
        while model.pop_sound().is_some() {}
    }
    app.update();

    let world = app.world_mut();
    let mut elements = world.query::<(&NanocomMessageElement, &Visibility)>();
    let visibility = elements
        .iter(world)
        .map(|(element, visibility)| (*element, *visibility))
        .collect::<Vec<_>>();
    let visibility_of = |target| {
        visibility
            .iter()
            .find_map(|(element, visibility)| (*element == target).then_some(visibility))
    };
    assert_eq!(
        visibility_of(NanocomMessageElement::CompactIcon),
        Some(&Visibility::Inherited)
    );
    assert_eq!(
        visibility_of(NanocomMessageElement::ExpandedRoot),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        visibility_of(NanocomMessageElement::ExpandedIcon),
        Some(&Visibility::Inherited),
        "a Visible child escapes the hidden modal root and renders at screen center"
    );
}

#[test]
fn presentation_fails_closed_for_unreached_clean_message_types() {
    for kind in [
        NanocomMessageKind::TradeInvite,
        NanocomMessageKind::ClubInvite,
        NanocomMessageKind::Skill,
    ] {
        assert!(!kind.is_reached_presentation());
    }
    for kind in [
        NanocomMessageKind::Npc,
        NanocomMessageKind::Nano,
        NanocomMessageKind::BuddyInvite,
        NanocomMessageKind::GroupInvite,
    ] {
        assert!(kind.is_reached_presentation());
    }

    let mut model = NanocomMessageUiModel::default();
    model.enqueue(NanocomMessageRequest {
        request_id: 12,
        kind: NanocomMessageKind::TradeInvite,
        title: "Trade Invitation".to_owned(),
        body: "Unreached producer".to_owned(),
        compact_frame_path: NANOCOM_BUDDY_FRAME_PATH.to_owned(),
        compact_icon_path: Some(NANOCOM_BUDDY_ICON_PATH.to_owned()),
        lifetime_seconds: NANOCOM_BUDDY_LIFETIME_SECONDS,
        localized_title: None,
        localized_body: None,
        voice_true_name: None,
        buddy_name: None,
    });
    model.set_expanded(true);
    assert!(!model.compact_visible());
    assert!(!model.expanded_visible());

    model.clear();
    model.enqueue(NanocomMessageRequest::group_invite(14, "Remote Player"));
    model.set_expanded(true);
    assert!(model.compact_visible());
    assert!(model.expanded_visible());
}

#[test]
fn interactive_priority_is_stable_fifo_within_both_classes() {
    let mut model = NanocomMessageUiModel::default();
    model.enqueue(passive(1));
    model.enqueue(passive(2));
    model.enqueue(buddy(3));
    model.enqueue(buddy(4));
    model.enqueue(passive(5));

    let ids = model
        .queued()
        .iter()
        .map(|queued| queued.request.request_id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![3, 4, 1, 2, 5]);
    assert_eq!(model.reveal_parameter(), 1.0);
}

#[test]
fn type_9_adapter_remains_passive_and_uses_ten_second_lifetime() {
    let mut model = NanocomMessageUiModel::default();
    model.enqueue_type_9_numbuh_two(9, "Numbuh Two", "Come in, cadet.");
    let active = model.active().expect("type-9 head");
    assert_eq!(active.request.kind, NanocomMessageKind::Npc);
    assert_eq!(active.remaining_seconds, 10.0);
    assert!(!active.request.kind.is_interactive());
    assert!(!model.expanded_visible());
    assert!(model.choose(NanocomMessageChoice::Accept).is_none());
}

#[test]
fn reveal_uses_exact_squared_sine_and_top_right_pivot() {
    let viewport = Vec2::new(1_264.0, 681.0);
    let hidden = nanocom_compact_layout(viewport, 1.0, 1.0);
    assert_close(hidden.painted.x, 1_142.0);
    assert_close(hidden.painted.width, 0.0);

    let revealed = nanocom_compact_layout(viewport, 0.0, 1.0);
    assert_close(revealed.painted.x, 770.0);
    assert_close(revealed.painted.width, 372.0);
    assert_close(
        viewport.x - (revealed.painted.x + revealed.painted.width),
        NANOCOM_REVEALED_RIGHT_MARGIN,
    );

    let scaled = nanocom_compact_layout(viewport, 0.0, 2.0);
    assert_close(scaled.painted.x, 276.0);
    assert_close(scaled.painted.width, 744.0);
    assert_close(scaled.node.x, 462.0);
    assert_close(scaled.node.y, 61.0);
}
