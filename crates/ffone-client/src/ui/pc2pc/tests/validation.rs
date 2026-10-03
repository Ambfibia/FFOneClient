use super::*;

#[test]
fn source_audit_keeps_pc2pc_text_creation_key_first() {
    let source = concat!(
        include_str!("../constants.rs"),
        "\n",
        include_str!("../containers.rs"),
        "\n",
        include_str!("../assets.rs"),
        "\n",
        include_str!("../interaction.rs"),
        "\n",
        include_str!("../layout.rs"),
        "\n",
        include_str!("../codec.rs"),
        "\n",
        include_str!("../types_pc2pc_offer_runtime0104.rs"),
        "\n",
        include_str!("../types_pc2pc_authoritative_snapshot0104.rs"),
        "\n",
        include_str!("../types_pc2pc_ui_model0104.rs"),
        "\n",
        include_str!("../types_pc2pc_text_style0104.rs"),
        "\n",
        include_str!("../projection.rs"),
        "\n",
        include_str!("../commands.rs"),
        "\n",
        include_str!("../validation.rs"),
        "\n",
        include_str!("../operations.rs"),
        "\n",
        include_str!("../output.rs"),
        "\n",
        include_str!("../state.rs"),
        "\n",
        include_str!("../view.rs"),
        "\n",
        include_str!("../localization_equipment_slot_localized_text.rs"),
        "\n",
        include_str!("../mod.rs")
    );
    let text_constructor = ["Text", "::new("].concat();
    let constructors = source
        .match_indices(&text_constructor)
        .filter(|(offset, _)| {
            *offset == 0
                || (!source.as_bytes()[offset - 1].is_ascii_alphanumeric()
                    && source.as_bytes()[offset - 1] != b'_')
        })
        .collect::<Vec<_>>();
    assert_eq!(
        constructors.len(),
        1,
        "new PC2PC Text must go through spawn_pc2pc_text_node"
    );
    let constructor_tail =
        &source[constructors[0].0..source.len().min(constructors[0].0 + 180)];
    assert!(constructor_tail.contains("localized"));
    assert!(source.contains(".before(LocalizationSet::Apply)"));

    let binder = &source[source.find("fn bind_pc2pc_ui").unwrap()
        ..source.find("fn presentation_icon").unwrap()];
    assert!(!binder.contains("&mut Text"));
    assert!(!binder.contains("text.0"));
    assert!(binder.contains("Option<&mut LocalizedText>"));
    for required in [
        "pc2pc_taros_amount_text(model.projection.local_offer_taros)",
        "pc2pc_remote_offer_text(model.projection.remote_name.to_uppercase())",
        "pc2pc_passthrough_text(model.projection.remote_name.clone())",
        "pc2pc_chat_log_text(&model.chat)",
    ] {
        assert!(
            binder.contains(required),
            "missing keyed binder {required:?}"
        );
    }
}
