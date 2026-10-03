//! Translate only exact server command replies; other server text is preserved.
use super::*;

pub(super) fn command_reply(text: &str) -> Option<LocalizedText> {
    let (key, fallback) = match text {
        "Available commands" => ("ui.chat.command.available", "Available commands"),
        "/redeem: No code specified" => (
            "ui.chat.command.redeem.missing",
            "/redeem: No code specified",
        ),
        "Usage: /redeem <code>" => ("ui.chat.command.redeem.usage", "Usage: /redeem <code>"),
        "/redeem: Code too long" => ("ui.chat.command.redeem.long", "/redeem: Code too long"),
        "/redeem: Unknown code" => ("ui.chat.command.redeem.unknown", "/redeem: Unknown code"),
        "/redeem: You have already redeemed this code item" => (
            "ui.chat.command.redeem.repeat",
            "/redeem: You have already redeemed this code item",
        ),
        "/redeem: Not enough space in inventory" => (
            "ui.chat.command.redeem.full",
            "/redeem: Not enough space in inventory",
        ),
        "/redeem: Finish trading before redeeming a code" => (
            "ui.chat.command.redeem.trade",
            "/redeem: Finish trading before redeeming a code",
        ),
        "/redeem: Could not save code rewards; try again later" => (
            "ui.chat.command.redeem.failed",
            "/redeem: Could not save code rewards; try again later",
        ),
        "You have redeemed code items" => (
            "ui.chat.command.redeem.success",
            "You have redeemed code items",
        ),
        "/about: Show information about the server" => (
            "ui.chat.command.help.about",
            "/about: Show information about the server",
        ),
        "/level: Change your character's level" => (
            "ui.chat.command.help.level",
            "/level: Change your character's level",
        ),
        "/levelx: Change your character's level" => (
            "ui.chat.command.help.levelx",
            "/levelx: Change your character's level",
        ),
        "/whois: Describe the nearest NPC" => (
            "ui.chat.command.help.whois",
            "/whois: Describe the nearest NPC",
        ),
        "/ban_a: Ban an account" => ("ui.chat.command.help.ban_a", "/ban_a: Ban an account"),
        "/ban_i: Ban a player and their account" => (
            "ui.chat.command.help.ban_i",
            "/ban_i: Ban a player and their account",
        ),
        "/unban: Unban an account" => ("ui.chat.command.help.unban", "/unban: Unban an account"),
        "/followme: Make the nearest NPC start following you" => (
            "ui.chat.command.help.followme",
            "/followme: Make the nearest NPC start following you",
        ),
        "/unfollowme: Stop the nearest NPC from following you" => (
            "ui.chat.command.help.unfollowme",
            "/unfollowme: Stop the nearest NPC from following you",
        ),
        "/changeai: Change the AI script of an NPC" => (
            "ui.chat.command.help.changeai",
            "/changeai: Change the AI script of an NPC",
        ),
        "/queryai: Query the AI script of an NPC" => (
            "ui.chat.command.help.queryai",
            "/queryai: Query the AI script of an NPC",
        ),
        "/reload: Reload all Lua scripts" => (
            "ui.chat.command.help.reload",
            "/reload: Reload all Lua scripts",
        ),
        "/perms: View or change a player's permissions level" => (
            "ui.chat.command.help.perms",
            "/perms: View or change a player's permissions level",
        ),
        "/ping: View your current ping to the server" => (
            "ui.chat.command.help.ping",
            "/ping: View your current ping to the server",
        ),
        "/refresh: Reinsert the player into the current chunk" => (
            "ui.chat.command.help.refresh",
            "/refresh: Reinsert the player into the current chunk",
        ),
        "/registerall: Register all transportation locations" => (
            "ui.chat.command.help.registerall",
            "/registerall: Register all transportation locations",
        ),
        "/unregisterall: Unregister all transportation locations" => (
            "ui.chat.command.help.unregisterall",
            "/unregisterall: Unregister all transportation locations",
        ),
        "/help: Show this help message" => {
            ("ui.chat.command.help.help", "/help: Show this help message")
        }
        "/redeem: Redeem a code item" => {
            ("ui.chat.command.help.redeem", "/redeem: Redeem a code item")
        }
        _ => return None,
    };
    Some(LocalizedText::new(key, fallback))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_reply_keys_resolve_in_both_actual_bundles() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
        let en: serde_json::Value = serde_json::from_slice(&std::fs::read(root.join("localization/en.json")).unwrap()).unwrap();
        let (localization, language) = Localization::open(&root, "ru").unwrap();
        let mut checked = 0;
        for (key, value) in en["entries"].as_object().unwrap() {
            if key.starts_with("ui.chat.command.help.") || key.starts_with("ui.chat.command.redeem.") || key == "ui.chat.command.available" {
                let english = value.as_str().unwrap();
                let reply = command_reply(english).expect("server reply has a semantic key");
                assert_ne!(localization.text(&language, &reply), english, "{key}");
                checked += 1;
            }
        }
        assert_eq!(checked, 29);
        assert!(command_reply("Player says: Available commands").is_none());
    }
}
