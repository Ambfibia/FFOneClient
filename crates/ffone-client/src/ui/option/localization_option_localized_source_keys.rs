use super::*;

pub const OPTION_TRANSLATION_RECT: OptionUiRect = OptionUiRect::new(238.0, 79.0, 175.0, 25.0);

pub const OPTION_VOICE_LANGUAGE_RECT: OptionUiRect = OptionUiRect::new(238.0, 127.0, 175.0, 25.0);

pub const OPTION_LANGUAGE_HEADER_RECT: OptionUiRect = OptionUiRect::new(10.0, 20.0, 450.0, 45.0);

pub const OPTION_LANGUAGE_BODY_RECT: OptionUiRect = OptionUiRect::new(10.0, 50.0, 450.0, 150.0);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionLanguageExtensionRoot;

pub(super) const OPTION_LOCALIZED_SOURCE_KEYS: &[(&str, &str)] = &[
    ("GRAPHICS & SOUND", "ui.option.tab.graphics"),
    ("GAME UI", "ui.option.tab.game_ui"),
    ("SOCIAL", "ui.option.tab.social"),
    ("CONTROLS", "ui.option.tab.controls"),
    ("APPLY CHANGE", "ui.option.apply"),
    ("SAVE AND EXIT", "ui.option.save_exit"),
    ("GRAPHICS", "ui.option.graphics.title"),
    ("SOUND", "ui.option.sound.title"),
    ("DEFAULT GRAPHICS", "ui.option.graphics.defaults"),
    ("DEFAULT SOUND", "ui.option.sound.defaults"),
    ("SCREEN MODE:", "ui.option.graphics.screen_mode"),
    ("DETAIL LEVEL:", "ui.option.graphics.detail_level"),
    ("VISIBILITY:", "ui.option.graphics.visibility"),
    (
        "PARTICLE EFFECT DENSITY:",
        "ui.option.graphics.particle_density",
    ),
    ("SHADOWS:", "ui.option.graphics.shadows"),
    ("TEXTURE QUALITY:", "ui.option.graphics.texture_quality"),
    ("CUSTOM DETAIL LEVEL", "ui.option.graphics.custom_detail"),
    ("Near", "ui.option.common.near"),
    ("Far", "ui.option.common.far"),
    ("Low", "ui.option.common.low"),
    ("Medium", "ui.option.common.medium"),
    ("High", "ui.option.common.high"),
    ("On", "ui.option.common.on"),
    ("Off", "ui.option.common.off"),
    ("Yes", "ui.option.common.yes"),
    ("No", "ui.option.common.no"),
    ("YES", "ui.option.common.yes_upper"),
    ("NO", "ui.option.common.no_upper"),
    ("TOON OUTLINE", "ui.option.graphics.toon_outline"),
    ("GLOW", "ui.option.graphics.glow"),
    (
        "ANISOTROPIC FILTERING",
        "ui.option.graphics.anisotropic_filtering",
    ),
    ("SOFT VEGETATION", "ui.option.graphics.soft_vegetation"),
    ("OBJECT FADING", "ui.option.graphics.object_fading"),
    ("MASTER VOLUME", "ui.option.sound.master"),
    ("SOUND EFFECT VOLUME", "ui.option.sound.effects"),
    ("VOICE VOLUME", "ui.option.sound.voice"),
    ("AMBIENT VOLUME", "ui.option.sound.ambient"),
    ("MUSIC VOLUME", "ui.option.sound.music"),
    ("TOON OUTLINE:", "ui.option.graphics.toon_outline_label"),
    ("GLOW:", "ui.option.graphics.glow_label"),
    (
        "ANISOTROPIC FILTERING:",
        "ui.option.graphics.anisotropic_filtering_label",
    ),
    (
        "SOFT VEGETATION:",
        "ui.option.graphics.soft_vegetation_label",
    ),
    ("OBJECT FADING:", "ui.option.graphics.object_fading_label"),
    ("MASTER VOLUME:", "ui.option.sound.master_label"),
    ("SOUND EFFECT VOLUME:", "ui.option.sound.effects_label"),
    ("VOICE VOLUME:", "ui.option.sound.voice_label"),
    ("AMBIENT VOLUME:", "ui.option.sound.ambient_label"),
    ("MUSIC VOLUME:", "ui.option.sound.music_label"),
    ("CUSTOM VOLUME", "ui.option.sound.custom_volume"),
    ("Browser", "ui.option.graphics.browser"),
    (
        "BestQuality",
        "ui.option.graphics.detail.best_quality_value",
    ),
    (
        "GoodQuality",
        "ui.option.graphics.detail.good_quality_value",
    ),
    ("Balanced", "ui.option.graphics.detail.balanced_value"),
    (
        "GoodPerformance",
        "ui.option.graphics.detail.good_performance_value",
    ),
    (
        "BestPerformance",
        "ui.option.graphics.detail.best_performance_value",
    ),
    ("Custom", "ui.option.graphics.detail.custom_value"),
    ("BEST QUALITY", "ui.option.graphics.detail.best_quality"),
    ("GOOD QUALITY", "ui.option.graphics.detail.good_quality"),
    ("BALANCED", "ui.option.graphics.detail.balanced"),
    (
        "GOOD PERFORMANCE",
        "ui.option.graphics.detail.good_performance",
    ),
    (
        "BEST PERFORMANCE",
        "ui.option.graphics.detail.best_performance",
    ),
    ("PlayerOnly", "ui.option.graphics.shadow.player_only_value"),
    (
        "AllCharacters",
        "ui.option.graphics.shadow.all_characters_value",
    ),
    ("Own Player Only", "ui.option.graphics.shadow.player_only"),
    ("All Characters", "ui.option.graphics.shadow.all_characters"),
    ("DISPLAY ELEMENTS", "ui.option.game_ui.display_elements"),
    ("CHAT TEXT COLORS", "ui.option.game_ui.chat_text_colors"),
    (
        "DEFAULT DISPLAY ELEMENTS",
        "ui.option.game_ui.default_display_elements",
    ),
    (
        "DEFAULT TEXT COLORS",
        "ui.option.game_ui.default_text_colors",
    ),
    ("FLOATING DISPLAY", "ui.option.game_ui.floating_display"),
    ("MY NAME:", "ui.option.game_ui.my_name"),
    (
        "OTHER PLAYER NAMES:",
        "ui.option.game_ui.other_player_names",
    ),
    (
        "GROUP MEMBER NAMES:",
        "ui.option.game_ui.group_member_names",
    ),
    ("NPC NAMES:", "ui.option.game_ui.npc_names"),
    ("MONSTER NAMES:", "ui.option.game_ui.monster_names"),
    ("DAMAGE NUMBERS:", "ui.option.game_ui.damage_numbers"),
    ("BALLOON CHAT:", "ui.option.game_ui.balloon_chat"),
    ("SCALE UI ELEMENTS:", "ui.option.game_ui.scale_ui_elements"),
    ("OLD CHAT:", "ui.option.game_ui.old_chat"),
    ("COMPUTRESS HINTS:", "ui.option.game_ui.computress_hints"),
    (
        "NPC MESSAGES IN CHAT WINDOW:",
        "ui.option.game_ui.npc_messages",
    ),
    (
        "COMBAT IN CHAT WINDOW:",
        "ui.option.game_ui.combat_messages",
    ),
    ("ANIMATED NANOCOM:", "ui.option.game_ui.animated_nanocom"),
    ("GENERAL CHAT", "ui.option.game_ui.general_chat"),
    ("GROUP CHAT", "ui.option.game_ui.group_chat"),
    ("BUDDY CHAT", "ui.option.game_ui.buddy_chat"),
    ("INPUT CONTROLS", "ui.option.controls.input_controls"),
    ("KEY MAPPING", "ui.option.controls.key_mapping"),
    (
        "DEFAULT INPUT CONTROLS",
        "ui.option.controls.default_input_controls",
    ),
    (
        "DEFAULT KEY MAPPING",
        "ui.option.controls.default_key_mapping",
    ),
    (
        "INVERT CAMERA Y-AXIS:",
        "ui.option.controls.invert_camera_y",
    ),
    (
        "CAMERA SENSITIVITY:",
        "ui.option.controls.camera_sensitivity",
    ),
    ("PAD INVERT Y-AXIS:", "ui.option.controls.pad_invert_y"),
    ("PAD SENSITIVITY:", "ui.option.controls.pad_sensitivity"),
    ("Selected Gamepad :", "ui.option.controls.selected_gamepad"),
    ("XBox 360", "ui.option.controls.pad.xbox360"),
    ("RumblePad 2", "ui.option.controls.pad.rumblepad2"),
    ("PS2", "ui.option.controls.pad.playstation2"),
    ("SETTING 1", "ui.option.controls.setting_1"),
    ("SETTING 2", "ui.option.controls.setting_2"),
    ("SETTING 3", "ui.option.controls.setting_3"),
    ("MOVEMENT", "ui.option.controls.group.movement"),
    ("INTERFACE", "ui.option.controls.group.interface"),
    ("CAMERA", "ui.option.controls.group.camera"),
    ("COMBAT", "ui.option.controls.group.combat"),
    ("FORWARD", "ui.option.controls.action.forward"),
    ("BACKWARD", "ui.option.controls.action.backward"),
    ("SIDESTEP LEFT", "ui.option.controls.action.sidestep_left"),
    ("SIDESTEP RIGHT", "ui.option.controls.action.sidestep_right"),
    ("ESCAPE", "ui.option.controls.action.escape"),
    ("JUMP", "ui.option.controls.action.jump"),
    ("ATTACK/USE TARGET", "ui.option.controls.action.attack"),
    ("NANO POWER", "ui.option.controls.action.nano_power"),
    ("SUMMON NANO #1", "ui.option.controls.action.nano_1"),
    ("SUMMON NANO #2", "ui.option.controls.action.nano_2"),
    ("SUMMON NANO #3", "ui.option.controls.action.nano_3"),
    ("SWITCH WEAPON", "ui.option.controls.action.switch_weapon"),
    ("CHARGE WEAPON", "ui.option.controls.action.charge_weapon"),
    ("NANO BOOST", "ui.option.controls.action.nano_boost"),
    ("MY STUFF", "ui.option.controls.action.inventory"),
    ("NANO COM", "ui.option.controls.action.nanocom"),
    ("JOURNAL", "ui.option.controls.action.journal"),
    ("E-MAIL", "ui.option.controls.action.email"),
    ("CONTACTS", "ui.option.controls.action.contacts"),
    ("HELP", "ui.option.controls.action.help"),
    ("AUTORUN", "ui.option.controls.action.autorun"),
    ("MENU", "ui.option.controls.action.menu"),
    ("MAP", "ui.option.controls.action.map"),
    ("OPTIONS", "ui.option.controls.action.options"),
    ("TURN LEFT", "ui.option.controls.action.turn_left"),
    ("TURN RIGHT", "ui.option.controls.action.turn_right"),
    ("FREE CAMERA", "ui.option.controls.action.free_camera"),
    ("SEND CHAT", "ui.option.controls.action.send_chat"),
    ("ZOOM IN", "ui.option.controls.action.zoom_in"),
    ("ZOOM OUT", "ui.option.controls.action.zoom_out"),
    ("CAMERA UP", "ui.option.controls.action.camera_up"),
    ("CAMERA DOWN", "ui.option.controls.action.camera_down"),
    ("CAMERA LEFT", "ui.option.controls.action.camera_left"),
    ("CAMERA RIGHT", "ui.option.controls.action.camera_right"),
    ("SKILL #1", "ui.option.controls.action.skill_1"),
    ("SKILL #2", "ui.option.controls.action.skill_2"),
    ("SKILL #3", "ui.option.controls.action.skill_3"),
    ("VEHICLE", "ui.option.controls.action.vehicle"),
    ("NONE", "ui.option.controls.action.none"),
    ("ALLOW REQUESTS", "ui.option.social.allow_requests"),
    ("BLOCKED PLAYERS", "ui.option.social.blocked_players"),
    (
        "Select a name below to remove it from your Blocked Players list. Reminder: Unblocked players can send you requests and chat messages.",
        "ui.option.social.blocked_help",
    ),
    ("DEFAULT REQUESTS", "ui.option.social.default_requests"),
    ("GROUP REQUESTS:", "ui.option.social.group_requests"),
    ("BUDDY REQUESTS:", "ui.option.social.buddy_requests"),
    ("TRADE REQUESTS:", "ui.option.social.trade_requests"),
    ("UNIGNORE", "ui.option.social.unignore"),
    ("OK", "ui.option.common.ok"),
    ("GAME LANGUAGE", "ui.option.language.title"),
    ("TEXT LANGUAGE:", "ui.option.language.text_label"),
    ("VOICE LANGUAGE:", "ui.option.language.voice_label"),
    (
        "Text and voice can be changed independently.",
        "ui.option.language.independent_hint",
    ),
    ("ENGLISH", "ui.option.language.english"),
    ("RUSSIAN", "ui.option.language.russian"),
    (
        "NOTE!\nThis option is experimental.\nYou must restart your game for this option to take effect.",
        "ui.option.popup.restart_required",
    ),
    (
        "THAT KEY OR BUTTON IS ALREADY ASSIGNED.\nCHOOSE A DIFFERENT SETTING.",
        "ui.option.popup.duplicate_binding",
    ),
];

pub(super) fn option_localized_text(source: &str) -> Option<LocalizedText> {
    OPTION_LOCALIZED_SOURCE_KEYS
        .iter()
        .find_map(|(candidate, key)| {
            (*candidate == source).then(|| LocalizedText::new(*key, source))
        })
}

pub(super) fn dropdown_value_localized(
    model: &OptionUiModel,
    kind: OptionDropdownKind,
    localization: Option<&Localization>,
    language: Option<&Language>,
    voice_language: Option<&VoiceLanguage>,
) -> LocalizedText {
    match kind {
        OptionDropdownKind::Resolution => {
            if model.draft_options.graphics.windowed {
                option_localized_text("Browser").expect("Browser has an Option localization key")
            } else {
                option_passthrough_text(format!(
                    "{}X{}",
                    model.draft_options.graphics.width, model.draft_options.graphics.height
                ))
            }
        }
        OptionDropdownKind::Detail => option_localized_text(
            model.draft_options.graphics.detail.label(),
        )
        .unwrap_or_else(|| option_passthrough_text(model.draft_options.graphics.detail.label())),
        OptionDropdownKind::Shadow => option_localized_text(
            model.draft_options.graphics.shadow.label(),
        )
        .unwrap_or_else(|| option_passthrough_text(model.draft_options.graphics.shadow.label())),
        OptionDropdownKind::Texture => option_localized_text(
            model.draft_options.graphics.texture.label(),
        )
        .unwrap_or_else(|| option_passthrough_text(model.draft_options.graphics.texture.label())),
        OptionDropdownKind::Pad => option_localized_text(model.draft_input.pad_profile.label())
            .expect("every clean gamepad profile has an Option localization key"),
        OptionDropdownKind::Translation => {
            let selected = resolve_option_source(
                localization,
                language,
                &locale_source(language.map(|value| value.effective.as_str())),
            );
            LocalizedText::new("ui.option.language.selected_value", "{language}")
                .with_arg("language", selected)
        }
        OptionDropdownKind::Voice => {
            let selected = resolve_option_source(
                localization,
                language,
                &locale_source(voice_language.map(|value| value.effective.as_str())),
            );
            LocalizedText::new("ui.option.language.selected_value", "{language}")
                .with_arg("language", selected)
        }
    }
}

pub(super) fn locale_source(locale: Option<&str>) -> String {
    locale.map_or_else(|| "ENGLISH".to_owned(), locale_label)
}

pub(super) fn locale_label(locale: &str) -> String {
    match locale {
        "en" => "ENGLISH".to_owned(),
        "ru" => "RUSSIAN".to_owned(),
        other => other.to_ascii_uppercase(),
    }
}
