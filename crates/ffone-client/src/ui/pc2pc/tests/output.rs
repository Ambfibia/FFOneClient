use super::*;

#[test]
fn dynamic_pc2pc_copy_uses_keyed_arguments_only() {
    let remote = pc2pc_remote_offer_text("MANDARK");
    assert_eq!(remote.key, "ui.pc2pc.offer.remote");
    assert_eq!(remote.fallback, "{name}'S OFFER");
    assert_eq!(remote.args["name"], "MANDARK");
    assert_eq!(pc2pc_fallback_text(&remote), "MANDARK'S OFFER");

    let taros = pc2pc_taros_amount_text(4_242);
    assert_eq!(taros.fallback, "{amount}");
    assert_eq!(taros.args["amount"], "4242");
    assert!(!taros.fallback.contains("4242"));

    let count = pc2pc_count_text("17");
    assert_eq!(count.key, "ui.inventory.item.count");
    assert_eq!(count.args["count"], "17");

    let chat = VecDeque::from([Pc2pcChatLine0104 {
        speaker_pc_id: REMOTE_ID,
        display_name: "Server-authored name".to_owned(),
        text: "Player-authored message".to_owned(),
        emote_code: 1,
    }]);
    let chat = pc2pc_chat_log_text(&chat);
    assert_eq!(chat.key, "ui.pc2pc.chat.log");
    assert_eq!(chat.fallback, "{log}");
    assert_eq!(
        chat.args["log"],
        "Server-authored name: Player-authored message"
    );
    assert!(!chat.fallback.contains("Server-authored"));
    assert!(!chat.fallback.contains("Player-authored"));

    let weapon = equipment_slot_localized_text(6);
    assert_eq!(weapon.key, "ui.inventory.slot.weapon");
    assert_eq!(weapon.fallback, "WEAPON {ordinal}");
    assert_eq!(weapon.args["ordinal"], "1");
    assert_eq!(
        Pc2pcButtonLabel::Accept.localized_text().key,
        "ui.pc2pc.button.accept"
    );
}
