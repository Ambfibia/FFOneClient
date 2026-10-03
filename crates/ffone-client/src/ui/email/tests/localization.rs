use super::*;

#[test]
fn production_email_tree_attaches_key_first_localization_to_every_text() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(EmailUiPlugin);
    app.update();

    let assets = app.world().resource::<EmailUiAssets>().clone();
    let world = app.world_mut();
    let mut texts = world.query::<(
        &Text,
        &LocalizedText,
        &EmailUiTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &UiTransform,
        &ChildOf,
    )>();
    let text_rows: Vec<_> = texts
        .iter(world)
        .map(
            |(text, localized, style, font, layout, transform, parent)| {
                (
                    text.0.clone(),
                    localized.key.clone(),
                    *style,
                    (font.0.clone(), *font.1),
                    layout.clone(),
                    *transform,
                    parent.parent(),
                )
            },
        )
        .collect();
    assert!(!text_rows.is_empty());
    assert_eq!(
        text_rows.len(),
        world.query::<&Text>().iter(world).count(),
        "an Email Text entity bypassed LocalizedText"
    );
    for expected_key in [
        "ui.email.send_mail",
        "ui.email.folder.guide",
        "ui.email.folder.player",
        "ui.email.folder.empty",
        "ui.email.previous_five",
        "ui.email.next_five",
        "ui.email.page.of",
        "ui.email.page.number",
        "ui.email.page.emails",
        "ui.email.column.from",
        "ui.email.column.subject",
        "ui.email.column.day",
        "ui.email.detail.from",
        "ui.email.detail.subject",
        "ui.email.detail.received",
        "ui.email.detail.no_selection",
        "ui.email.detail.taros",
        "ui.email.accept_all_items",
        "ui.email.accept_taros",
        "ui.email.remove_buddy",
        "ui.email.delete",
        "ui.email.reply",
        "ui.email.compose.title",
        "ui.email.compose.to",
        "ui.email.compose.subject",
        "ui.email.compose.attachments",
        "ui.email.compose.taros",
        "ui.email.compose.postage.label",
        "ui.email.compose.postage.amount",
        "ui.email.buddy_list",
        "ui.email.add_taros",
        "ui.email.cancel",
        "ui.email.send",
        "ui.email.calculator.value",
        "ui.email.calculator.amount_to_add",
        "ui.email.calculator.digit",
        "ui.email.calculator.clear",
        "ui.email.calculator.add",
    ] {
        assert!(
            text_rows.iter().any(|(_, key, ..)| key == expected_key),
            "missing semantic Email text role {expected_key}"
        );
    }
    for (text, key, style, font, layout, transform, parent) in text_rows {
        assert!(!key.is_empty(), "empty localization key for {text:?}");
        assert_eq!(
            font,
            style.font(&assets),
            "wrong source font metrics for {key}"
        );
        let expected_layout = style.layout();
        assert_eq!(
            layout.justify, expected_layout.justify,
            "wrong source alignment for {key}"
        );
        assert_eq!(
            layout.linebreak, expected_layout.linebreak,
            "wrong source wrapping for {key}"
        );
        assert_eq!(
            transform,
            UiTransform::from_translation(Val2::px(0.0, style.y_offset())),
            "wrong replacement-font Y calibration for {key}"
        );
        assert!(
            world.get::<Node>(parent).is_some(),
            "Email Text {key} is not owned by a source-Rect container"
        );
    }
    let mut fit_query = world.query::<(&LocalizedText, Option<&UiTextAutoFit>)>();
    let unbounded = fit_query
        .iter(world)
        .filter_map(|(localized, fit)| fit.is_none().then_some(localized.key.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        unbounded,
        std::collections::BTreeSet::from([
            "ui.email.detail.no_selection",
            "ui.email.compose.postage.amount",
            "ui.email.compose.postage.label",
        ]),
        "all bounded Email Text must preserve its source Rect via translation auto-fit"
    );
    let mut popup_query = world.query::<(&EmailPopup, &GlobalZIndex)>();
    let popup_depths = popup_query
        .iter(world)
        .map(|(popup, depth)| (*popup, depth.0))
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        popup_depths,
        std::collections::HashSet::from([
            (EmailPopup::BuddyList, EMAIL_UI_POPUP_Z_INDEX),
            (EmailPopup::AddTaros, EMAIL_UI_POPUP_Z_INDEX),
        ]),
        "clean depth-8 popup passes must render above the depth-9..11 panels"
    );
}

#[test]
fn source_audit_keeps_email_text_creation_behind_localized_helpers() {
    let source = concat!(
        include_str!("../constants.rs"),
        "\n",
        include_str!("../state.rs"),
        "\n",
        include_str!("../assets.rs"),
        "\n",
        include_str!("../interaction.rs"),
        "\n",
        include_str!("../frame.rs"),
        "\n",
        include_str!("../validation.rs"),
        "\n",
        include_str!("../input_resolve_email_escape_gate.rs"),
        "\n",
        include_str!("../layout.rs"),
        "\n",
        include_str!("../animation.rs"),
        "\n",
        include_str!("../types_email_ui_text_style.rs"),
        "\n",
        include_str!("../types_email_ui_plugin.rs"),
        "\n",
        include_str!("../commands.rs"),
        "\n",
        include_str!("../models.rs"),
        "\n",
        include_str!("../operations_flush_email_transport_outbox.rs"),
        "\n",
        include_str!("../operations_email_summary_sender_text.rs"),
        "\n",
        include_str!("../audio.rs"),
        "\n",
        include_str!("../view_email_list_panel.rs"),
        "\n",
        include_str!("../view_email_calculator_popup.rs"),
        "\n",
        include_str!("../systems.rs"),
        "\n",
        include_str!("../localization_email_localized_text.rs"),
        "\n",
        include_str!("../mod.rs")
    ).replace("\r\n", "\n");
    let text_constructor = ["Text", "::new("].concat();
    let text_constructors: Vec<_> = source
        .match_indices(&text_constructor)
        .filter(|(offset, _)| {
            *offset == 0
                || !source.as_bytes()[offset - 1].is_ascii_alphanumeric()
                    && source.as_bytes()[offset - 1] != b'_'
        })
        .collect();
    assert_eq!(
        text_constructors.len(),
        1,
        "new Email Text must go through the single localized/style-aware helper"
    );
    assert!(source.contains(".before(LocalizationSet::Apply)"));
    for (offset, _) in text_constructors {
        let tail = &source[offset..source.len().min(offset + 180)];
        assert!(
            tail.contains("localized"),
            "Text constructor at byte {offset} has no adjacent LocalizedText component"
        );
    }
    let binder = &source[source.find("fn sync_email_ui_model").unwrap()
        ..source.find("fn email_fallback_text").unwrap()];
    assert!(!binder.contains("&mut Text"));
    assert!(!binder.contains("text.0"));
    for forbidden in [
        ["Text", "::new(format!("].concat(),
        ["text", ".0 = email_text"].concat(),
        ["Text", "::new(model."].concat(),
    ] {
        assert!(
            !source.contains(&forbidden),
            "Email source bypasses keyed dynamic localization via {forbidden:?}"
        );
    }
}
