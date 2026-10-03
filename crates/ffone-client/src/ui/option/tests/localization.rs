use super::*;

#[test]
fn language_fields_fit_new_social_section_without_moving_footer_or_controls() {
    assert_eq!(
        OPTION_TRANSLATION_RECT,
        OptionUiRect::new(238.0, 79.0, 175.0, 25.0)
    );
    assert_eq!(
        OPTION_VOICE_LANGUAGE_RECT,
        OptionUiRect::new(238.0, 127.0, 175.0, 25.0)
    );
    assert!(
        OPTION_VOICE_LANGUAGE_RECT.x + OPTION_VOICE_LANGUAGE_RECT.width <= OPTION_APPLY_RECT.x
    );
    assert_eq!(
        OPTION_KEYMAP_VIEW_RECT,
        OptionUiRect::new(25.0, 230.0, 900.0, 260.0)
    );
}

#[test]
fn social_language_fields_switch_independently_and_block_other_pages() {
    let (localization, language) = Localization::open(&asset_root(), "en").unwrap();
    let mut app = App::new();
    app.insert_resource(localization)
        .insert_resource(language)
        .insert_resource(VoiceLanguage {
            requested: "en".into(),
            effective: "en".into(),
        })
        .init_resource::<OptionUiModel>()
        .init_resource::<OptionUiOutbox>()
        .add_systems(Update, handle_dropdown_interactions);
    {
        let mut model = app.world_mut().resource_mut::<OptionUiModel>();
        model.visible = true;
        model.selected_tab = OptionTab::Social;
        model.open_dropdown(OptionDropdownKind::Translation);
    }
    let text_choice = app
        .world_mut()
        .spawn((
            Interaction::Pressed,
            OptionDropdownChoice::Translation("ru".into()),
        ))
        .id();
    app.update();
    assert_eq!(app.world().resource::<Language>().effective, "ru");
    assert_eq!(app.world().resource::<VoiceLanguage>().effective, "en");
    app.world_mut().despawn(text_choice);
    app.world_mut()
        .resource_mut::<OptionUiModel>()
        .open_dropdown(OptionDropdownKind::Voice);
    let voice_choice = app
        .world_mut()
        .spawn((
            Interaction::Pressed,
            OptionDropdownChoice::Voice("ru".into()),
        ))
        .id();
    app.update();
    assert_eq!(app.world().resource::<Language>().effective, "ru");
    assert_eq!(app.world().resource::<VoiceLanguage>().effective, "ru");
    assert!(!app.world().resource::<OptionUiModel>().modal.voice_dropdown);
    app.world_mut().despawn(voice_choice);
    app.world_mut().resource_mut::<OptionUiModel>().selected_tab = OptionTab::Controls;
    app.world_mut().spawn((
        Interaction::Pressed,
        OptionDropdownChoice::Translation("en".into()),
    ));
    app.update();
    assert_eq!(app.world().resource::<Language>().effective, "ru");
}
