use super::*;

#[must_use]
pub fn system_message_button_localized(spec: SystemMessageButtonSpec) -> LocalizedText {
    let key = match spec.label {
        "OKAY" => "ui.system_message.okay",
        "CANCEL" => "ui.common.cancel",
        "YES" => "ui.system_message.yes",
        "NO" => "ui.system_message.no",
        "EXIT CHARACTER CREATION" => "ui.system_message.exit_character_creation",
        "DELETE MISSION" => "ui.system_message.delete_mission",
        "SKIP" => "ui.system_message.skip",
        "CONTINUE" => "ui.common.continue",
        "LEAVE" => "ui.system_message.leave",
        "DELETE" => "ui.common.delete",
        "WARP" => "ui.system_message.warp",
        _ => "ui.content.passthrough",
    };
    if key == "ui.content.passthrough" {
        LocalizedText::new(key, "{text}").with_arg("text", spec.label)
    } else {
        LocalizedText::new(key, spec.label)
    }
}
