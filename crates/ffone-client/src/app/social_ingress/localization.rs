use super::*;

pub(super) fn localized_player_chat_line(
    kind: ChatLineKind,
    key: &'static str,
    fallback: &'static str,
    sender: impl Into<String>,
    message: impl Into<String>,
) -> ChatLineUi {
    let sender = sender.into();
    let message = message.into();
    let text = match key {
        "ui.hud.chat.message.local" => format!("[Local] {sender}: {message}"),
        "ui.hud.chat.message.group" => format!("[Group] {sender}: {message}"),
        "ui.hud.chat.message.buddy" => format!("[Buddy] {sender}: {message}"),
        "ui.hud.chat.message.direct" => format!("{sender}: {message}"),
        _ => unreachable!("player chat helper owns a closed semantic-key set"),
    };
    ChatLineUi {
        text,
        kind,
        localized: Some(
            LocalizedText::new(key, fallback)
                .with_arg("sender", sender)
                .with_arg("message", message),
        ),
    }
}
