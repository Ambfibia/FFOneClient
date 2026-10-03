use super::*;

#[test]
fn source_audit_keeps_clean_item_and_buddy_controls_connected_to_handlers() {
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
    for required in [
        "Button,\n            EmailUiAttachmentSlot",
        "Button,\n                            EmailUiAttachmentSlot",
        "Button,\n                                        EmailUiInventorySlot",
        "EmailUiBuddyRow { row }",
        "select_email_buddy(buddy.row",
        "attach_email_inventory_item(inventory_slot, attachment_slot",
        "detach_email_inventory_item(attachment_slot",
        "accept_email_item(\n                    email_item_slot",
    ] {
        assert!(
            source.contains(required),
            "production Email UI lost a reachable clean interaction path: {required:?}"
        );
    }
    assert!(source.contains("first_free_email_attachment_slot(&model)"));
    assert!(source.contains("first_free_email_inventory_slot(&model)"));
    assert!(source.contains("email_item_slot: email_item_slot as i32 + 1"));
}
