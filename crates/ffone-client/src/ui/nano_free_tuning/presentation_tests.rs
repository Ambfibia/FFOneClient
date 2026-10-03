use crate::nano_free_tuning_ui::*;
use bevy::asset::AssetPlugin;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use tempfile::tempdir;

fn test_app() -> App {
    let asset_root = tempdir().unwrap().keep();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(NanoFreeTuningUiPlugin);
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.update();
    app
}

fn sample_content() -> NanoFreeTuningContent {
    NanoFreeTuningContent {
        nano_id: 1,
        nano_style: 0,
        nano_name: "Buttercup".to_owned(),
        powers: std::array::from_fn(|index| NanoFreeTuningPower {
            tune_id: index as i16 + 1,
            skill_id: index as i16 + 11,
            icon_path: format!("icons/power-{index}.png"),
            name: format!("POWER {index}"),
            power_type: format!("TYPE {index}"),
            description: format!("DESCRIPTION {index}"),
        }),
    }
}

#[test]
fn result_playback_gets_its_full_duration_after_delayed_binding_and_next_award_resets() {
    let mut model = NanoFreeTuningModel::default();
    let context = NanoFreeTuningOpenContext {
        player_id: 7,
        killed_fusion: false,
        content: sample_content(),
        world: NanoFreeTuningWorldSnapshot::default(),
    };
    model.open(context.clone()).unwrap();
    model.phase = NanoFreeTuningPhase::ResultSkill;
    model.phase_elapsed = 0.0;
    model.advance(4.0).unwrap();
    model.set_result_animation_duration(1.4).unwrap();
    model.advance(1.0).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::ResultSkill);
    // Reporting the same bound clip again must not restart its clock.
    model.set_result_animation_duration(1.4).unwrap();
    model.advance(0.5).unwrap();
    assert_eq!(model.phase(), NanoFreeTuningPhase::ResultHide);
    model.cancel().unwrap();
    model.open(context).unwrap();
    model
        .advance(NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS + 0.01)
        .unwrap();
    assert!(model.controls_enabled());
    assert!(model.result_animation_duration.is_none());
    assert!(!model.intents().iter().any(|intent| matches!(
        intent,
        NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::HidePreviewNano)
            | NanoFreeTuningIntent::World(NanoFreeTuningWorldIntent::DestroyPreviewNano)
    )));
}

#[test]
fn button_states_and_release_commit_use_the_spawned_controls() {
    let mut app = test_app();
    {
        let mut model = app.world_mut().resource_mut::<NanoFreeTuningModel>();
        model
            .open(NanoFreeTuningOpenContext {
                player_id: 7,
                killed_fusion: false,
                content: sample_content(),
                world: NanoFreeTuningWorldSnapshot::default(),
            })
            .unwrap();
        model
            .advance(NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS + 0.01)
            .unwrap();
    }
    app.update();
    let buttons: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<NanoFreeTuningPowerButton>>()
        .iter(app.world())
        .collect();
    assert_eq!(buttons.len(), 3);
    let assets = app
        .world()
        .resource::<NanoFreeTuningPresentationAssets>()
        .clone();
    for button in buttons {
        let power_index = app
            .world()
            .get::<NanoFreeTuningPowerButton>(button)
            .unwrap()
            .power_index;
        let label = app.world().get::<Children>(button).unwrap()[0];
        assert!(app.world().get::<Pickable>(button).is_some());
        assert_eq!(
            app.world().get::<FocusPolicy>(label),
            Some(&FocusPolicy::Pass)
        );
        assert_eq!(
            app.world().get::<ImageNode>(button).unwrap().image,
            assets.select_normal
        );
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Hovered;
        app.world_mut()
            .get_mut::<RelativeCursorPosition>(button)
            .unwrap()
            .cursor_over = true;
        app.update();
        assert_eq!(
            app.world().get::<ImageNode>(button).unwrap().image,
            assets.select_hover
        );
        assert_eq!(
            app.world().get::<TextColor>(label).unwrap().0,
            NANO_FREE_TUNING_BUTTON_HOVER_TEXT
        );
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(
            app.world().get::<ImageNode>(button).unwrap().image,
            assets.select_normal
        );
        assert!(
            app.world_mut()
                .resource_mut::<NanoFreeTuningUiCommandOutbox>()
                .pop()
                .is_none()
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Hovered;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<NanoFreeTuningUiCommandOutbox>()
                .pop(),
            Some(NanoFreeTuningUiCommand::SelectAndConfirm { power_index })
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
        app.update();
        assert!(
            app.world_mut()
                .resource_mut::<NanoFreeTuningUiCommandOutbox>()
                .pop()
                .is_none()
        );
    }
}

#[test]
fn drag_outside_and_disable_during_press_cancel_without_replay() {
    let mut app = test_app();
    {
        let mut model = app.world_mut().resource_mut::<NanoFreeTuningModel>();
        model
            .open(NanoFreeTuningOpenContext {
                player_id: 7,
                killed_fusion: false,
                content: sample_content(),
                world: NanoFreeTuningWorldSnapshot::default(),
            })
            .unwrap();
        model
            .advance(NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS + 0.01)
            .unwrap();
    }
    let button = app
        .world_mut()
        .query_filtered::<Entity, With<NanoFreeTuningPowerButton>>()
        .iter(app.world())
        .next()
        .unwrap();
    for disable in [false, true] {
        app.world_mut()
            .resource_mut::<NanoFreeTuningModel>()
            .gui_enabled = true;
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
        app.world_mut()
            .get_mut::<RelativeCursorPosition>(button)
            .unwrap()
            .cursor_over = true;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        if disable {
            app.world_mut()
                .resource_mut::<NanoFreeTuningModel>()
                .gui_enabled = false;
            app.update();
            assert_eq!(
                app.world().get::<Pickable>(button).unwrap(),
                &Pickable::IGNORE
            );
            assert_eq!(
                app.world().get::<ImageNode>(button).unwrap().color.alpha(),
                NANO_FREE_TUNING_BUTTON_DISABLED_ALPHA
            );
            app.world_mut()
                .resource_mut::<NanoFreeTuningModel>()
                .gui_enabled = true;
        } else {
            app.world_mut()
                .get_mut::<RelativeCursorPosition>(button)
                .unwrap()
                .cursor_over = false;
        }
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        assert!(
            app.world_mut()
                .resource_mut::<NanoFreeTuningUiCommandOutbox>()
                .pop()
                .is_none()
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
    }
}

#[test]
fn creation_calls_animation_without_adding_a_summon_style_effect() {
    let mut model = NanoFreeTuningModel::default();
    let mut content = sample_content();
    content.nano_style = 2;
    model
        .open(NanoFreeTuningOpenContext {
            player_id: 7,
            killed_fusion: true,
            content,
            world: NanoFreeTuningWorldSnapshot::default(),
        })
        .unwrap();
    model.mark_effect_ready().unwrap();
    model
        .advance(NANO_FREE_TUNING_EFFECT_DELAY_SECONDS + 0.01)
        .unwrap();
    model
        .advance(NANO_FREE_TUNING_PROJECTILE_DELAY_SECONDS + 0.01)
        .unwrap();
    model
        .advance(NANO_FREE_TUNING_REVEAL_DELAY_SECONDS + 0.01)
        .unwrap();

    assert!(model.intents().iter().any(|intent| {
        matches!(
            intent,
            NanoFreeTuningIntent::Cinematic(NanoFreeTuningCinematicIntent::CallPreviewNano)
        )
    }));
}

#[test]
fn gameplay_phase_reentry_keeps_one_visible_selection_root() {
    let mut app = test_app();
    let root_count = |world: &mut World| {
        let mut query = world.query_filtered::<Entity, With<NanoFreeTuningPresentationRoot>>();
        query.iter(world).count()
    };
    assert_eq!(root_count(app.world_mut()), 1);

    app.world_mut()
        .resource_mut::<NextState<crate::ui_startup::NativeUiStartupPhase>>()
        .set(crate::ui_startup::NativeUiStartupPhase::Deferred);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<crate::ui_startup::NativeUiStartupPhase>>()
        .set(crate::ui_startup::NativeUiStartupPhase::Gameplay);
    app.update();
    assert_eq!(root_count(app.world_mut()), 1);

    app.world_mut()
        .resource_mut::<NanoFreeTuningModel>()
        .open(NanoFreeTuningOpenContext {
            player_id: 7,
            killed_fusion: false,
            content: sample_content(),
            world: NanoFreeTuningWorldSnapshot::default(),
        })
        .unwrap();
    {
        let mut model = app.world_mut().resource_mut::<NanoFreeTuningModel>();
        model.mark_effect_ready().unwrap();
        model
            .advance(NANO_FREE_TUNING_EFFECT_DELAY_SECONDS + 0.01)
            .unwrap();
        model
            .advance(NANO_FREE_TUNING_PROJECTILE_DELAY_SECONDS + 0.01)
            .unwrap();
        model
            .advance(NANO_FREE_TUNING_REVEAL_DELAY_SECONDS + 0.01)
            .unwrap();
        model
            .advance(NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS + 0.01)
            .unwrap();
    }
    app.update();

    let world = app.world_mut();
    let mut roots = world.query_filtered::<&Node, With<NanoFreeTuningPresentationRoot>>();
    assert_eq!(roots.single(world).unwrap().display, Display::Flex);
}

#[test]
fn every_real_text_is_key_first_and_carries_exact_source_gui_style() {
    let mut app = test_app();
    let world = app.world_mut();
    let text_entities = {
        let mut query = world.query_filtered::<Entity, With<Text>>();
        query.iter(world).collect::<Vec<_>>()
    };
    assert_eq!(text_entities.len(), 16);
    for entity in text_entities {
        let localized = world
            .get::<LocalizedText>(entity)
            .unwrap_or_else(|| panic!("Text is not key-first: {entity:?}"));
        let style = *world
            .get::<NanoFreeTuningUiTextStyle>(entity)
            .unwrap_or_else(|| panic!("Text has no clean GUIStyle: {entity:?}"));
        let font = world.get::<TextFont>(entity).unwrap();
        let transform = world
            .get::<UiTransform>(entity)
            .unwrap_or_else(|| panic!("Text has no replacement-font offset: {entity:?}"));
        assert!(!localized.key.trim().is_empty());
        assert_ne!(localized.key, "ui.content.passthrough");
        assert_eq!(style.source_skin_path_id(), NANO_FREE_TUNING_SKIN_PATH_ID);
        assert_eq!(style.replacement_font_path(), NANO_FREE_TUNING_FONT_PATH);
        assert_eq!(font.font_size, style.font_size().into());
        assert_eq!(
            *world.get::<LineHeight>(entity).unwrap(),
            LineHeight::Px(style.line_height())
        );
        assert_eq!(
            transform.translation,
            Val2::px(0.0, style.replacement_y_offset())
        );
    }

    let exact_styles = [
        (
            NanoFreeTuningUiTextStyle::BigBlueMiddleLeft,
            "bigblue",
            1_012,
            3,
            [0.0; 4],
            true,
            0,
            [0.8, 1.0, 1.0, 1.0],
            2,
        ),
        (
            NanoFreeTuningUiTextStyle::BigYellowMiddleLeft,
            "bigyellow",
            1_012,
            3,
            [0.0; 4],
            true,
            0,
            [1.0, 1.0, 0.0, 1.0],
            1,
        ),
        (
            NanoFreeTuningUiTextStyle::TransparentLightBlueUpperLeft,
            "TransparentLightBlue",
            903,
            0,
            [0.0; 4],
            false,
            0,
            [0.8, 1.0, 1.0, 1.0],
            4,
        ),
        (
            NanoFreeTuningUiTextStyle::TransparentYellowSmallUpperLeft,
            "TransparentYellowSmall",
            948,
            0,
            [0.0; 4],
            false,
            0,
            [1.0, 1.0, 0.0, 1.0],
            3,
        ),
        (
            NanoFreeTuningUiTextStyle::TransparentBlueUpperLeft,
            "TransparentBlue",
            948,
            0,
            [0.0; 4],
            true,
            0,
            [0.0, 1.0, 1.0, 1.0],
            3,
        ),
        (
            NanoFreeTuningUiTextStyle::ButtonMiddleCenter,
            "button",
            903,
            4,
            [6.0, 6.0, 3.0, 3.0],
            false,
            1,
            [0.8, 1.0, 1.0, 1.0],
            3,
        ),
    ];
    for (style, name, font_path_id, alignment, padding, wrap, clipping, color, count) in
        exact_styles
    {
        assert_eq!(style.source_style_name(), name);
        assert_eq!(style.source_font_path_id(), font_path_id);
        assert_eq!(style.legacy_alignment(), alignment);
        assert_eq!(style.padding(), padding);
        assert_eq!(style.content_offset(), [0.0, 0.0]);
        assert_eq!(style.word_wrap(), wrap);
        assert_eq!(style.legacy_text_clipping(), clipping);
        assert_eq!(style.text_color(), color);
        assert_eq!(style.replacement_y_offset(), 0.0);
        let mut query = world.query::<&NanoFreeTuningUiTextStyle>();
        assert_eq!(
            query.iter(world).filter(|actual| **actual == style).count(),
            count
        );
    }

    let mut buttons = world.query::<(&NanoFreeTuningPowerButton, &Node)>();
    for (_, node) in buttons.iter(world) {
        assert_eq!(node.padding, UiRect::new(px(6), px(6), px(3), px(3)));
        assert_eq!(node.overflow, Overflow::clip());
        assert_eq!(node.justify_content, JustifyContent::Center);
        assert_eq!(node.align_items, AlignItems::Center);
    }

    let mut icon_slots = world.query::<(&NanoFreeTuningIconLabelStyle, &Node)>();
    assert_eq!(icon_slots.iter(world).count(), 3);
    for (_, node) in icon_slots.iter(world) {
        assert_eq!(
            node.padding,
            UiRect::new(px(0), px(0), px(3), px(3)),
            "default GUI.skin.label image content padding"
        );
        assert_eq!(node.overflow, Overflow::clip());
    }
    assert_eq!(NanoFreeTuningIconLabelStyle::SOURCE_STYLE_NAME, "label");
    assert_eq!(NanoFreeTuningIconLabelStyle::SOURCE_SKIN_PATH_ID, 1_379);
    assert_eq!(NanoFreeTuningIconLabelStyle::SOURCE_FONT_PATH_ID, 903);
    assert_eq!(NanoFreeTuningIconLabelStyle::LEGACY_ALIGNMENT, 0);
    assert_eq!(NanoFreeTuningIconLabelStyle::CONTENT_OFFSET, [0.0, 0.0]);
    assert!(NanoFreeTuningIconLabelStyle::WORD_WRAP);
    assert_eq!(NanoFreeTuningIconLabelStyle::LEGACY_TEXT_CLIPPING, 1);
}

#[test]
fn table_authored_copy_uses_semantic_nano_and_tune_keys() {
    let mut app = test_app();
    app.world_mut()
        .resource_mut::<NanoFreeTuningModel>()
        .open(NanoFreeTuningOpenContext {
            player_id: 7,
            killed_fusion: false,
            content: sample_content(),
            world: NanoFreeTuningWorldSnapshot {
                player: NanoFreeTuningTransform {
                    position: Vec3::ZERO,
                    rotation: Quat::IDENTITY,
                },
                first_defeated_fusion: None,
            },
        })
        .unwrap();
    app.update();

    let world = app.world_mut();
    let mut texts = world.query::<(&NanoFreeTuningText, &LocalizedText)>();
    for (role, localized) in texts.iter(world) {
        match role.0 {
            NanoFreeTuningTextRole::AcquiredPrefix => {
                assert_eq!(localized.key, NANO_FREE_TUNING_ACQUIRED_LOCALIZATION_KEY);
                assert_eq!(localized.fallback, "YOU HAVE ACQUIRED ");
                assert!(localized.args.is_empty());
            }
            NanoFreeTuningTextRole::Bang => {
                assert_eq!(localized.key, NANO_FREE_TUNING_BANG_LOCALIZATION_KEY);
                assert_eq!(localized.fallback, " !");
                assert!(localized.args.is_empty());
            }
            NanoFreeTuningTextRole::NanoName => {
                assert_eq!(localized.key, "content.nano.1.acquired_name");
                assert_eq!(localized.fallback, " Buttercup ");
                assert!(localized.args.is_empty());
            }
            NanoFreeTuningTextRole::PowerName(index)
            | NanoFreeTuningTextRole::PowerType(index)
            | NanoFreeTuningTextRole::PowerDescription(index) => {
                let field = match role.0 {
                    NanoFreeTuningTextRole::PowerName(_) => "name",
                    NanoFreeTuningTextRole::PowerType(_) => "type_label",
                    _ => "description",
                };
                assert_eq!(
                    localized.key,
                    format!(
                        "content.nano_tune.{}.{field}",
                        sample_content().powers[index].tune_id
                    )
                );
                assert!(localized.args.is_empty());
            }
        }
    }
}

fn template_args(template: &str) -> BTreeSet<String> {
    let mut output = BTreeSet::new();
    let mut remaining = template;
    while let Some(open) = remaining.find('{') {
        remaining = &remaining[open + 1..];
        let close = remaining
            .find('}')
            .expect("closed localization placeholder");
        output.insert(remaining[..close].to_owned());
        remaining = &remaining[close + 1..];
    }
    output
}

#[test]
fn production_bundles_publish_every_reached_key_and_exact_clean_whitespace() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut bundles = BTreeMap::new();
    for locale in ["en", "ru"] {
        let runtime = fs::read(
            root.join("assets/game/localization")
                .join(format!("{locale}.json")),
        )
        .unwrap();
        let value: Value = serde_json::from_slice(&runtime).unwrap();
        bundles.insert(locale, value["entries"].as_object().unwrap().clone());
    }
    assert_eq!(
        bundles["en"].keys().collect::<BTreeSet<_>>(),
        bundles["ru"].keys().collect::<BTreeSet<_>>()
    );
    let exact = [
        (
            NANO_FREE_TUNING_ACQUIRED_LOCALIZATION_KEY,
            "YOU HAVE ACQUIRED ",
            "ВЫ ПОЛУЧИЛИ ",
        ),
        (
            NANO_FREE_TUNING_NANO_NAME_LOCALIZATION_KEY,
            " {name} ",
            " {name} ",
        ),
        (NANO_FREE_TUNING_BANG_LOCALIZATION_KEY, " !", " !"),
        (
            NANO_FREE_TUNING_TITLE_LOCALIZATION_KEY,
            "SELECT A POWER",
            "ВЫБЕРИТЕ СИЛУ",
        ),
        (
            NANO_FREE_TUNING_POWER_NAME_LOCALIZATION_KEY,
            "{name}",
            "{name}",
        ),
        (
            NANO_FREE_TUNING_POWER_TYPE_LOCALIZATION_KEY,
            "{type}",
            "{type}",
        ),
        (
            NANO_FREE_TUNING_POWER_DESCRIPTION_LOCALIZATION_KEY,
            "{description}",
            "{description}",
        ),
        (
            NANO_FREE_TUNING_SELECT_LOCALIZATION_KEY,
            "SELECT",
            "ВЫБРАТЬ",
        ),
    ];
    for (key, en, ru) in exact {
        assert_eq!(bundles["en"][key].as_str(), Some(en), "EN {key}");
        assert_eq!(bundles["ru"][key].as_str(), Some(ru), "RU {key}");
        assert_eq!(
            template_args(en),
            template_args(ru),
            "placeholder mismatch {key}"
        );
    }
}

#[test]
fn production_source_has_no_generic_passthrough_or_raw_text_mutation() {
    let source = concat!(
        include_str!("state.rs"),
        "\n",
        include_str!("containers.rs"),
        "\n",
        include_str!("assets.rs"),
        "\n",
        include_str!("codec.rs"),
        "\n",
        include_str!("commands.rs"),
        "\n",
        include_str!("constants.rs"),
        "\n",
        include_str!("audio.rs"),
        "\n",
        include_str!("layout.rs"),
        "\n",
        include_str!("output.rs"),
        "\n",
        include_str!("localization_nano_free_tuning_localized_text.rs"),
        "\n",
        include_str!("interaction.rs"),
        "\n",
        include_str!("types.rs"),
        "\n",
        include_str!("validation.rs"),
        "\n",
        include_str!("models.rs"),
        "\n",
        include_str!("operations.rs"),
        "\n",
        include_str!("view.rs"),
        "\n",
        include_str!("systems.rs"),
        "\n",
        include_str!("mod.rs")
    );
    let production = source.split_once("#[cfg(test)]").unwrap().0;
    assert!(!production.contains("ui.content.passthrough"));
    // TextColor changes state styling, not localized copy. Match the
    // component identifier instead of rejecting its TextColor prefix too.
    assert!(production.split("&mut Text").skip(1).all(|tail| {
        tail.chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    }));
    assert!(!production.contains("Mut<Text>"));
}
