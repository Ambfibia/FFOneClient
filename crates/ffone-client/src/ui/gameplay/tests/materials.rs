use super::*;

#[test]
fn chat_input_switches_semantic_copy_without_writing_an_english_render_value() {
    let chat = ChatUi::default();
    let transition = GameplayMenuTransition::default();
    assert_eq!(
        localized_chat_input(&chat, &transition),
        LocalizedText::new(
            "ui.hud.chat.open_hint",
            "Press ENTER to access chat and menus."
        )
    );

    let active = ChatUi {
        active: true,
        input: "Привет".to_owned(),
        ..default()
    };
    assert_eq!(
        localized_chat_input(&active, &transition),
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", "Привет")
    );
}
