use std::{fs, path::Path};

use bevy::asset::AssetPlugin;
use sha2::{Digest, Sha256};

use crate::login_ui::*;

fn spawned_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_systems(Startup, spawn_login_ui);
    app.update();
    app
}

#[test]
fn server_browser_renders_above_background_without_moving_centered_login() {
    let mut app = spawned_app();
    app.insert_resource(OptionUiModel::default())
        .init_resource::<LoginBrowser>()
        .init_resource::<browser_view::BrowserPopover>()
        .init_resource::<LoginUiModel>()
        .add_systems(Update, (update_login_layout, browser_view::bind_browser).chain());
    app.world_mut().spawn((Window {
        resolution: (1280, 720).into(),
        ..default()
    }, PrimaryWindow));
    app.world_mut().resource_mut::<LoginUiModel>().visible = true;
    app.update();
    let mut panels = app.world_mut().query_filtered::<&Node, With<LoginPanel>>();
    let panel = panels.single(app.world()).unwrap();
    assert_eq!(panel.left, px((1280.0 - LOGIN_PANEL_SIZE.x) / 2.0));
    assert_eq!(panel.top, px((720.0 - (LOGIN_PANEL_SIZE.y + 24.0)) / 2.0));
    let mut browsers = app.world_mut().query_filtered::<(&Node, &UiTransform, &ZIndex), With<browser_view::BrowserPanel>>();
    let (browser, transform, z) = browsers.single(app.world()).unwrap();
    assert!(z.0 > 0, "full-screen backgrounds must not cover the server browser");
    let Val::Px(left) = browser.left else { panic!("absolute browser position"); };
    let actual_left = left + 140.0 * (1.0 - transform.scale.x);
    assert!(actual_left > 640.0 + LOGIN_PANEL_SIZE.x / 2.0);
    assert!(actual_left + 280.0 * transform.scale.x <= 1280.0);
    app.world_mut().resource_mut::<LoginUiModel>().saved_account = Some("saved-player".into());
    app.update();
    let mut panels = app.world_mut().query_filtered::<&Node, With<LoginPanel>>();
    let panel = panels.single(app.world()).unwrap();
    assert_eq!(panel.height, px(LOGIN_PANEL_SIZE.y + 24.0));
    assert_eq!(panel.top, px((720.0 - (LOGIN_PANEL_SIZE.y + 24.0)) / 2.0));
    let mut submit = app.world_mut().query_filtered::<&Node, With<LoginSubmitButton>>();
    assert_eq!(submit.single(app.world()).unwrap().top, px(159.0));
    app.world_mut().resource_mut::<LoginUiModel>().saved_account = None;
    app.update();
    assert_eq!(submit.single(app.world()).unwrap().top, px(159.0), "editing must not shift the actions");
}

#[test]
fn every_login_text_entity_has_semantic_ownership() {
    let mut app = spawned_app();

    let mut texts = app
        .world_mut()
        .query::<(&Text, Option<&LocalizedText>, Option<&LoginTextStyle0104>)>();
    let rows = texts.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(rows.len(), 8);
    assert!(
        rows.iter()
            .all(|(_, localized, style)| localized.is_some() && style.is_some())
    );
}

#[test]
fn static_login_keys_and_clean_casing_are_exact() {
    let mut app = spawned_app();
    let mut query = app.world_mut().query::<&LocalizedText>();
    let mut rows = query
        .iter(app.world())
        .map(|localized| (localized.key.as_str(), localized.fallback.as_str()))
        .collect::<Vec<_>>();
    rows.sort_unstable();
    assert_eq!(
        rows,
        vec![
            (
                "status.login.credentials",
                "Enter your account name and password."
            ),
            ("ui.content.passthrough", "{text}"),
            ("ui.content.passthrough", "{text}"),
            ("ui.login.language_value", "LANGUAGE: {language}"),
            ("ui.login.password", "Password :"),
            ("ui.login.register", "How Do I Register?"),
            ("ui.login.submit", "Log In"),
            ("ui.login.username", "Username :"),
        ]
    );
}

#[test]
fn login_language_button_switches_supported_text_locale_before_authentication() {
    let mut app = spawned_app();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, language) = Localization::open(&root, "ru").unwrap();
    app.insert_resource(localization);
    app.insert_resource(language);
    app.insert_resource(crate::localization::VoiceLanguage {
        requested: "en".to_owned(),
        effective: "en".to_owned(),
    });
    app.insert_resource(OptionUiModel::default());
    app.insert_resource(LoginUiModel {
        visible: true,
        ..default()
    });
    app.add_systems(Update, handle_login_language_interaction);
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<LoginLanguageButton>>();
    let button = query.single(app.world()).unwrap();
    app.world_mut()
        .entity_mut(button)
        .insert(Interaction::Pressed);
    app.update();
    assert_eq!(app.world().resource::<Language>().effective, "en");
    app.world_mut().entity_mut(button).insert(Interaction::None);
    app.update();
    app.world_mut()
        .entity_mut(button)
        .insert(Interaction::Pressed);
    app.update();
    assert_eq!(app.world().resource::<Language>().effective, "ru");
    assert_eq!(
        app.world().resource::<crate::localization::VoiceLanguage>().effective,
        "en"
    );
}

#[test]
fn every_text_uses_the_serialized_style_font_and_line_height() {
    let mut app = spawned_app();
    let assets = app.world().resource::<LoginUiAssets>().clone();
    let mut query = app
        .world_mut()
        .query::<(&LoginTextStyle0104, (&TextFont, &LineHeight), &TextLayout)>();
    let mut counts = [0_u8; 4];
    for (style, font, layout) in query.iter(app.world()) {
        let (expected_handle, expected_size, expected_height, expected_index) = match style {
            LoginTextStyle0104::Label => (
                &assets.font,
                LOGIN_JEFFE_FONT_SIZE,
                LOGIN_JEFFE_LINE_HEIGHT,
                0,
            ),
            LoginTextStyle0104::Button => (
                &assets.font,
                LOGIN_JEFFE_FONT_SIZE,
                LOGIN_JEFFE_LINE_HEIGHT,
                1,
            ),
            LoginTextStyle0104::TextField => (
                &assets.text_field_font,
                LOGIN_CHALET_FONT_SIZE,
                LOGIN_CHALET_LINE_HEIGHT,
                2,
            ),
            LoginTextStyle0104::StatusAdapter => {
                (&assets.font, 11.0, LOGIN_JEFFE_LINE_HEIGHT, 3)
            }
        };
        counts[expected_index] += 1;
        assert_eq!(
            &font.0.font,
            &bevy::text::FontSource::Handle(expected_handle.clone())
        );
        assert_eq!(font.0.font_size.eval(Vec2::ZERO, 16.0), expected_size);
        assert_eq!((*font.1), LineHeight::Px(expected_height));
        assert_eq!(font.0.font_smoothing, FontSmoothing::AntiAliased);
        if *style == LoginTextStyle0104::Button {
            assert_eq!(layout.justify, Justify::Center);
            assert_eq!(layout.linebreak, LineBreak::NoWrap);
        }
    }
    assert_eq!(counts, [2, 3, 2, 1]);
}

#[test]
fn every_text_has_its_style_local_replacement_font_y_offset() {
    let mut app = spawned_app();
    let mut query = app
        .world_mut()
        .query::<(&LoginTextStyle0104, &UiTransform)>();
    let mut count = 0;
    for (style, transform) in query.iter(app.world()) {
        count += 1;
        assert_eq!(
            transform.translation,
            Val2::px(0.0, style.y_offset()),
            "{style:?}"
        );
    }
    assert_eq!(count, 8);
    assert_eq!(LOGIN_LABEL_Y_OFFSET, 0.0);
    assert_eq!(LOGIN_BUTTON_Y_OFFSET, 0.0);
    assert_eq!(LOGIN_TEXT_FIELD_Y_OFFSET, 0.0);
    assert_eq!(LOGIN_STATUS_ADAPTER_Y_OFFSET, 0.0);
}

#[test]
fn exact_gui_style_padding_is_attached_to_controls() {
    let mut app = spawned_app();
    let mut labels = app
        .world_mut()
        .query_filtered::<(&Node, &LoginTextStyle0104), With<LoginTextStyle0104>>();
    let styled_nodes = labels
        .iter(app.world())
        .filter(|(node, _)| node.position_type == PositionType::Absolute)
        .collect::<Vec<_>>();
    assert_eq!(styled_nodes.len(), 3);
    for (node, style) in styled_nodes {
        if *style == LoginTextStyle0104::Label {
            assert_eq!(node.padding, LOGIN_LABEL_PADDING);
        }
    }

    let mut fields = app.world_mut().query::<(
        &Node,
        Option<&LoginUsernameField>,
        Option<&LoginPasswordField>,
        Option<&LoginSubmitButton>,
        Option<&LoginCommunityButton>,
        Option<&LoginRegisterButton>,
    )>();
    for (node, username, password, submit, community, register) in fields.iter(app.world()) {
        if username.is_some() || password.is_some() {
            assert_eq!(node.padding, LOGIN_TEXT_FIELD_PADDING);
        } else if submit.is_some() || community.is_some() || register.is_some() {
            assert_eq!(node.padding, LOGIN_BUTTON_PADDING);
        }
    }
}

#[test]
fn panel_fields_and_buttons_use_exact_serialized_nine_slice_borders() {
    let mut app = spawned_app();
    let mut query = app.world_mut().query::<(
        &ImageNode,
        Option<&LoginPanel>,
        Option<&LoginUsernameField>,
        Option<&LoginPasswordField>,
        Option<&LoginStyledButton>,
        Option<&LoginLanguageButton>,
    )>();
    let mut panel_count = 0;
    let mut field_count = 0;
    let mut button_count = 0;
    for (image, panel, username, password, button, language_button) in query.iter(app.world()) {
        let NodeImageMode::Sliced(slicer) = &image.image_mode else {
            continue;
        };
        if panel.is_some() {
            panel_count += 1;
            assert_eq!(slicer.border, LOGIN_PANEL_BORDER);
        } else if username.is_some() || password.is_some() {
            field_count += 1;
            assert_eq!(slicer.border, LOGIN_TEXT_FIELD_BORDER);
            assert_eq!(image.visual_box, bevy::ui::VisualBox::BorderBox);
        } else if button.is_some() || language_button.is_some() {
            button_count += 1;
            assert_eq!(slicer.border, LOGIN_BUTTON_BORDER);
            assert_eq!(image.visual_box, bevy::ui::VisualBox::BorderBox);
        }
        assert_eq!(slicer.center_scale_mode, SliceScaleMode::Stretch);
        assert_eq!(slicer.sides_scale_mode, SliceScaleMode::Stretch);
    }
    assert_eq!((panel_count, field_count, button_count), (1, 2, 3));
}

#[test]
fn root_is_exact_black_and_clips_scale_and_crop() {
    let mut app = spawned_app();
    let mut query = app
        .world_mut()
        .query_filtered::<(&Node, &BackgroundColor), With<LoginRoot>>();
    let (node, color) = query.single(app.world()).expect("login root");
    assert_eq!(color.0, Color::BLACK);
    assert_eq!(node.overflow, Overflow::clip());
    assert_eq!(node.width, percent(100));
    assert_eq!(node.height, percent(100));
}

#[test]
fn clean_draw_order_is_fallback_loaded_panel() {
    let mut app = spawned_app();
    let mut roots = app
        .world_mut()
        .query_filtered::<&Children, With<LoginRoot>>();
    let children = roots
        .single(app.world())
        .expect("root children")
        .iter()
        // The browser owns a higher local ZIndex; preserve the background/form order.
        .filter(|entity| app.world().get::<browser_view::BrowserPanel>(*entity).is_none())
        .collect::<Vec<_>>();
    assert_eq!(children.len(), 3);
    assert!(
        app.world()
            .get::<LoginFallbackBackground>(children[0])
            .is_some()
    );
    assert!(
        app.world()
            .get::<LoginLoadedBackground>(children[1])
            .is_some()
    );
    assert!(app.world().get::<LoginPanel>(children[2]).is_some());
}

#[test]
fn source_audit_has_one_key_first_text_constructor_and_no_direct_text_binding() {
    let source = concat!(
        include_str!("constants.rs"),
        "\n",
        include_str!("state.rs"),
        "\n",
        include_str!("containers.rs"),
        "\n",
        include_str!("assets.rs"),
        "\n",
        include_str!("textures.rs"),
        "\n",
        include_str!("interaction.rs"),
        "\n",
        include_str!("types.rs"),
        "\n",
        include_str!("layout.rs"),
        "\n",
        include_str!("operations.rs"),
        "\n",
        include_str!("frame.rs"),
        "\n",
        include_str!("models.rs"),
        "\n",
        include_str!("commands.rs"),
        "\n",
        include_str!("view.rs"),
        "\n",
        include_str!("systems.rs"),
        "\n",
        include_str!("mod.rs")
    );
    let text_constructor = ["Text", "::new("].concat();
    let mutable_text_query = ["&mut ", "Text,"].concat();
    let settings_key = ["ui.login", ".settings"].concat();
    let settings_caption = ["SET", "TINGS"].concat();
    let localization_ordering = [".before(LocalizationSet", "::Apply)"].concat();
    assert_eq!(
        source
            .lines()
            .filter(|line| line.trim_start().starts_with(&text_constructor))
            .count(),
        1
    );
    assert!(!source.contains(&mutable_text_query));
    assert!(!source.contains(&settings_key));
    assert!(!source.contains(&settings_caption));
    assert!(source.contains(&localization_ordering));
}

#[test]
fn bind_login_ui_system_parameters_are_disjoint() {
    let mut world = World::new();
    let mut system = IntoSystem::into_system(bind_login_ui);

    system.initialize(&mut world);
}

#[test]
fn background_branch_truth_table_matches_clean_early_returns() {
    assert_eq!(
        login_background_mode(LoginSurface::AutoLogin, false),
        LoginBackgroundMode0104::Black
    );
    assert_eq!(
        login_background_mode(LoginSurface::AutoLogin, true),
        LoginBackgroundMode0104::Black
    );
    assert_eq!(
        login_background_mode(LoginSurface::WarpShard, true),
        LoginBackgroundMode0104::FallbackScaleToFit
    );
    for surface in [
        LoginSurface::Manual,
        LoginSurface::WaitingForWebAuthentication,
        LoginSurface::WebAuthentication,
    ] {
        assert_eq!(
            login_background_mode(surface, false),
            LoginBackgroundMode0104::FallbackScaleToFit
        );
        assert_eq!(
            login_background_mode(surface, true),
            LoginBackgroundMode0104::LoadedScaleAndCrop
        );
    }
}

#[test]
fn dynamic_background_assignment_is_forbidden_in_clean_web_warp_and_auto_branches() {
    assert!(login_can_assign_loaded_background(LoginSurface::Manual));
    assert!(login_can_assign_loaded_background(
        LoginSurface::WaitingForWebAuthentication
    ));
    assert!(!login_can_assign_loaded_background(
        LoginSurface::WebAuthentication
    ));
    assert!(!login_can_assign_loaded_background(LoginSurface::WarpShard));
    assert!(!login_can_assign_loaded_background(LoginSurface::AutoLogin));
}

#[test]
fn ready_loaded_background_is_assigned_after_the_clean_fallback_draw() {
    let (first_mode, assigned_after_first_draw) =
        login_background_frame(LoginSurface::Manual, true, false, true);
    assert_eq!(first_mode, LoginBackgroundMode0104::FallbackScaleToFit);
    assert!(assigned_after_first_draw);

    let (second_mode, assigned_after_second_draw) =
        login_background_frame(LoginSurface::Manual, true, assigned_after_first_draw, true);
    assert_eq!(second_mode, LoginBackgroundMode0104::LoadedScaleAndCrop);
    assert!(assigned_after_second_draw);

    let (_, hidden_assignment) =
        login_background_frame(LoginSurface::Manual, false, false, true);
    assert!(!hidden_assignment);
    for surface in [
        LoginSurface::WebAuthentication,
        LoginSurface::WarpShard,
        LoginSurface::AutoLogin,
    ] {
        let (_, assigned) = login_background_frame(surface, true, false, true);
        assert!(!assigned, "{surface:?}");
    }
}

#[test]
fn clean_ui_scale_formula_keeps_small_windows_at_one() {
    assert_eq!(login_ui_scale(681.0, true, false), 1.0);
    assert_eq!(login_ui_scale(768.0, true, false), 1.05);
    assert_eq!(login_ui_scale(1_536.0, true, false), 2.1);
    assert_eq!(login_ui_scale(1_536.0, false, false), 1.0);
    assert_eq!(login_ui_scale(1_536.0, true, true), 1.0);
}

#[test]
fn exact_clean_source_identity_and_object_ownership_are_stable() {
    assert_eq!(LOGIN_SOURCE_BUILD, "retrobution-20260613");
    assert_eq!(LOGIN_SOURCE_MAIN_ARCHIVE, "main.unity3d");
    assert_eq!(LOGIN_SOURCE_MAIN_ARCHIVE_BYTES, 7_000_415);
    assert_eq!(LOGIN_SOURCE_RUNTIME, "fusion-2.x.x");
    assert_eq!(LOGIN_SOURCE_SERIALIZED_FILE, "sharedassets0.assets");
    assert_eq!(LOGIN_SOURCE_GAME_OBJECT_PATH_ID, 1_350);
    assert_eq!(LOGIN_COMPONENT_PATH_ID, 1_469);
    assert_eq!(LOGIN_COMPONENT_SCRIPT_PATH_ID, 1_068);
    assert_eq!(LOGIN_MODE_COMPONENT_PATH_ID, 1_471);
    assert_eq!(LOGIN_MODE_SCRIPT_PATH_ID, 1_026);
    assert_eq!(LOGIN_SKIN_PATH_ID, 1_374);
    assert_eq!(LOGIN_BLACK_TEXTURE_PATH_ID, 270);
    assert_eq!(LOGIN_BLACK_TEXTURE_SIZE, UVec2::new(8, 8));
    assert_eq!(LOGIN_BLACK_TEXTURE_SOURCE_BYTES, 32);
    assert_eq!(
        LOGIN_BLACK_TEXTURE_SOURCE_SHA256,
        "66687AADF862BD776C8FC18B8E9F8E20089714856EE233B3902A591D0D5F2925"
    );
    assert_eq!(
        LOGIN_BLACK_TEXTURE_DECODED_RGBA_SHA256,
        "2C893CD47B6757133987906E00C55E6DCCE3C567B9DF9E7F91FFECD3604CC6A6"
    );
    assert_eq!(LOGIN_FALLBACK_BACKGROUND_PATH_ID, 369);
    assert_eq!(LOGIN_PANEL_TEXTURE_PATH_ID, 101);
    assert_eq!(LOGIN_BUTTON_NORMAL_TEXTURE_PATH_ID, 411);
    assert_eq!(LOGIN_BUTTON_HOVER_TEXTURE_PATH_ID, 320);
    assert_eq!(LOGIN_BUTTON_ACTIVE_TEXTURE_PATH_ID, 567);
    assert_eq!(LOGIN_TEXT_FIELD_TEXTURE_PATH_ID, 337);
    assert_eq!(LOGIN_LOADED_BACKGROUND_TEXTURE_PATH_ID, 35);
    assert_eq!(LOGIN_SOURCE_CREATION_ARCHIVE_BYTES, 8_974_798);
    assert_eq!(
        LOGIN_SOURCE_CREATION_ARCHIVE_SHA256,
        "78785925E716027DE4BEC897C402C352BB411B7D627DE1783E6FBDA8BF34E59E"
    );
    assert_eq!(
        LOGIN_SOURCE_CREATION_ASSET,
        "CustomAssetBundle-bd5f53480423447d7bcaed95cb2a96c8"
    );
    assert_eq!(LOGIN_JEFFE_SOURCE_FONT_PATH_ID, 1_012);
    assert_eq!(LOGIN_CHALET_SOURCE_FONT_PATH_ID, 1_115);
    assert_eq!(
        LOGIN_SOURCE_MAIN_ARCHIVE_SHA256,
        "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F"
    );
    assert_eq!(
        LOGIN_SOURCE_UNITY_ENGINE_SHA256,
        "90EF121A97F954D35A50FB27F7DBCF99EAFFEAFCBCF938EAE9910C29F3745E0C"
    );
}

#[test]
fn published_login_images_and_replacement_fonts_match_the_pinned_contracts() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for spec in LOGIN_IMAGE_SPECS {
        let path = asset_root.join(spec.path);
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!("cannot read {} for {}: {error}", path.display(), spec.role)
        });
        assert_eq!(bytes.len() as u64, spec.bytes, "{}", spec.role);
        assert_eq!(
            format!("{:X}", Sha256::digest(&bytes)),
            spec.sha256,
            "{}",
            spec.role
        );
        assert_eq!(
            image::image_dimensions(&path).unwrap(),
            (spec.width, spec.height),
            "{}",
            spec.role
        );
    }
    for spec in LOGIN_REPLACEMENT_FONT_SPECS {
        let path = asset_root.join(spec.path);
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!("cannot read {} for {}: {error}", path.display(), spec.role)
        });
        assert_eq!(bytes.len() as u64, spec.bytes, "{}", spec.role);
        assert_eq!(
            format!("{:X}", Sha256::digest(&bytes)),
            spec.sha256,
            "{}",
            spec.role
        );
    }
}

#[test]
fn button_and_text_field_state_colours_match_serialized_gui_style_states() {
    assert_eq!(
        login_text_field_color(Interaction::None, false),
        Color::srgb(0.901_960_85, 0.901_960_85, 0.901_960_85)
    );
    assert_eq!(
        login_text_field_color(Interaction::Hovered, false),
        Color::srgb(0.9, 0.9, 0.9)
    );
    assert_eq!(
        login_text_field_color(Interaction::Pressed, false),
        Color::WHITE
    );
    assert_eq!(
        login_text_field_color(Interaction::None, true),
        Color::WHITE
    );
    assert_eq!(
        login_button_text_color(Interaction::None),
        Color::srgb(0.9, 0.9, 0.9)
    );
    assert_eq!(login_button_text_color(Interaction::Hovered), Color::WHITE);
    assert_eq!(login_button_text_color(Interaction::Pressed), Color::WHITE);
}

#[test]
fn select_all_replaces_only_the_focused_login_field() {
    let mut model = LoginUiModel {
        username: "старый".into(),
        password: "secret".into(),
        ..default()
    };
    let mut outbox = LoginUiOutbox::default();
    apply_login_edit_key(
        &mut model,
        &mut outbox,
        KeyCode::KeyA,
        Some("ф"),
        true,
        false,
    );
    assert_eq!(model.username_edit.range(), 0..6);
    assert_eq!(model.username, "старый");
    apply_login_key(&mut model, &mut outbox, KeyCode::KeyN, Some("новый"));
    assert_eq!(model.username, "новый");
    assert_eq!(model.password, "secret");
    apply_login_key(&mut model, &mut outbox, KeyCode::Tab, None);
    apply_login_edit_key(
        &mut model,
        &mut outbox,
        KeyCode::KeyA,
        Some("a"),
        true,
        false,
    );
    apply_login_key(&mut model, &mut outbox, KeyCode::Delete, None);
    assert!(model.password.is_empty());
    assert_eq!(model.username, "новый");
    assert!(outbox.requests.is_empty());
}

#[test]
fn initial_clean_focus_is_username_and_tab_cycles_named_controls() {
    let mut model = LoginUiModel::default();
    let mut outbox = LoginUiOutbox::default();
    assert_eq!(model.focused, LoginField::Username);
    apply_login_key(&mut model, &mut outbox, KeyCode::Tab, None);
    assert_eq!(model.focused, LoginField::Password);
    apply_login_key(&mut model, &mut outbox, KeyCode::Tab, None);
    assert_eq!(model.focused, LoginField::Username);
}

#[test]
fn keyboard_text_targets_only_the_focused_control() {
    let mut model = LoginUiModel::default();
    let mut outbox = LoginUiOutbox::default();
    apply_login_key(&mut model, &mut outbox, KeyCode::KeyA, Some("Alice"));
    apply_login_key(&mut model, &mut outbox, KeyCode::Tab, None);
    apply_login_key(&mut model, &mut outbox, KeyCode::KeyN, Some("nano"));
    assert_eq!(model.username, "Alice");
    assert_eq!(model.password, "nano");
}

#[test]
fn username_field_removes_newlines_and_caps_at_clean_max_length() {
    let mut model = LoginUiModel::default();
    let mut outbox = LoginUiOutbox::default();
    apply_login_key(
        &mut model,
        &mut outbox,
        KeyCode::KeyA,
        Some("abcd\nefgh\r0123456789012345678901234567890123456789"),
    );
    assert!(!model.username.contains('\n'));
    assert!(!model.username.contains('\r'));
    assert_eq!(model.username.chars().count(), 32);
}

#[test]
fn password_field_caps_at_clean_max_length_and_trims_every_edit() {
    let mut model = LoginUiModel {
        focused: LoginField::Password,
        ..default()
    };
    let mut outbox = LoginUiOutbox::default();
    apply_login_key(
        &mut model,
        &mut outbox,
        KeyCode::KeyA,
        Some("  secret  0123456789012345678901234567890123456789"),
    );
    assert!(!model.password.starts_with(' '));
    assert!(!model.password.ends_with(' '));
    assert_eq!(model.password.chars().count(), 32);
}

#[test]
fn backspace_mutates_only_the_focused_control() {
    let mut model = LoginUiModel {
        username: "User".into(),
        password: "pass".into(),
        focused: LoginField::Password,
        ..default()
    };
    let mut outbox = LoginUiOutbox::default();
    apply_login_key(&mut model, &mut outbox, KeyCode::Backspace, None);
    assert_eq!(model.username, "User");
    assert_eq!(model.password, "pas");
}

#[test]
fn enter_and_numpad_enter_submit_the_same_exact_request() {
    for key in [KeyCode::Enter, KeyCode::NumpadEnter] {
        let mut model = LoginUiModel {
            username: " Dexter ".into(),
            password: " nano ".into(),
            ..default()
        };
        let mut outbox = LoginUiOutbox::default();
        apply_login_key(&mut model, &mut outbox, key, None);
        assert_eq!(
            outbox.drain().collect::<Vec<_>>(),
            vec![LoginRequest {
                username: "Dexter".into(),
                password: "nano".into(),
            }]
        );
    }
}

#[test]
fn pending_login_rejects_duplicate_enter_submission() {
    let mut model = LoginUiModel {
        username: "Dexter".into(),
        password: "nano".into(),
        ..default()
    };
    let mut outbox = LoginUiOutbox::default();
    apply_login_key(&mut model, &mut outbox, KeyCode::Enter, None);
    apply_login_key(&mut model, &mut outbox, KeyCode::Enter, None);
    assert_eq!(outbox.drain().count(), 1);
}

#[test]
fn empty_login_is_rejected_locally() {
    let mut model = LoginUiModel::default();
    let mut outbox = LoginUiOutbox::default();
    queue_login(&mut model, &mut outbox);
    assert!(outbox.drain().next().is_none());
    assert_eq!(model.focused, LoginField::Username);
    assert!(!model.busy);
    assert!(model.status.is_empty());
}

#[test]
fn valid_login_is_queued() {
    let mut model = LoginUiModel {
        username: "Ambfibia".to_owned(),
        password: "secret".to_owned(),
        ..default()
    };
    let mut outbox = LoginUiOutbox::default();
    queue_login(&mut model, &mut outbox);
    let request = outbox.drain().next().expect("request");
    assert_eq!(request.username, "Ambfibia");
    assert_eq!(request.password, "secret");
    assert!(model.busy);
}

#[test]
fn submit_matches_legacy_username_cleanup_and_password_trimming() {
    let mut model = LoginUiModel {
        username: "  Amb\nfibia  ".to_owned(),
        password: "  secret  ".to_owned(),
        ..default()
    };
    let mut outbox = LoginUiOutbox::default();

    queue_login(&mut model, &mut outbox);

    let request = outbox.drain().next().expect("manual login request");
    assert_eq!(request.username, "Ambfibia");
    assert_eq!(request.password, "secret");
    assert_eq!(model.username, "Ambfibia");
    assert_eq!(model.password, "secret");
}

#[test]
fn all_legacy_non_manual_surfaces_fail_closed_for_form_input() {
    let mut model = LoginUiModel {
        visible: true,
        ..default()
    };
    assert!(model.accepts_manual_input());

    for surface in [
        LoginSurface::WaitingForWebAuthentication,
        LoginSurface::WebAuthentication,
        LoginSurface::AutoLogin,
        LoginSurface::WarpShard,
    ] {
        model.surface = surface;
        assert!(!model.accepts_manual_input(), "{surface:?}");
    }

    model.surface = LoginSurface::Manual;
    model.system_popup_active = true;
    assert!(!model.accepts_manual_input());
    model.system_popup_active = false;
    model.busy = true;
    assert!(!model.accepts_manual_input());
}

#[test]
fn retained_link_effects_are_exact_and_do_not_invent_registration_navigation() {
    let mut effects = LoginUiEffectOutbox::default();
    effects.effects.push_back(LoginUiEffect::OpenCommunity {
        url: LOGIN_COMMUNITY_URL,
    });
    effects
        .effects
        .push_back(LoginUiEffect::ShowRegistrationInstructions {
            message: LOGIN_REGISTRATION_INSTRUCTIONS,
        });

    assert_eq!(
        effects.drain().collect::<Vec<_>>(),
        vec![
            LoginUiEffect::OpenCommunity {
                url: "http://www.forums.fusionfalluniverse.com",
            },
            LoginUiEffect::ShowRegistrationInstructions {
                message: "To Register:\nChoose an Username and Password into the appropriate boxes and press Log In. Make sure to remember these as there is no account recovery!",
            },
        ]
    );
}

#[test]
fn login_fields_native_action_rectangles_and_scale_and_crop_are_stable() {
    assert_eq!(
        LOGIN_BUTTON_PATH,
        "ui/en/server-selection/ff-button-normal.png"
    );
    assert_eq!(
        LOGIN_TEXT_FIELD_PATH,
        "ui/en/launcher/login/ff-textfield-normal.png"
    );
    assert_eq!(LOGIN_TEXT_FIELD_FONT_PATH, "fonts/chaletbook-regular.ttf");
    assert_eq!(LOGIN_PANEL_SIZE, Vec2::new(370.0, 275.0));
    assert_eq!(
        LOGIN_USERNAME_FIELD_RECT,
        UiSourceRect::new(45.0, 40.0, 280.0, 25.0)
    );
    assert_eq!(
        LOGIN_PASSWORD_FIELD_RECT,
        UiSourceRect::new(45.0, 100.0, 280.0, 25.0)
    );
    assert_eq!(
        LOGIN_SUBMIT_RECT,
        UiSourceRect::new(45.0, 135.0, 280.0, 35.0)
    );
    assert_eq!(LOGIN_LANGUAGE_RECT, UiSourceRect::new(45.0, 178.0, 280.0, 35.0));
    assert_eq!(LOGIN_BUTTON_PADDING.left, LOGIN_BUTTON_PADDING.right);
    assert_eq!(LOGIN_BUTTON_PADDING.top, LOGIN_BUTTON_PADDING.bottom);
    assert_eq!(LOGIN_GLAYOUT_TOP, 200.0);
    assert_eq!(LOGIN_GLAYOUT_EN_COMMUNITY_ADVANCE, 187.0);
    assert_eq!(LOGIN_GLAYOUT_EN_REGISTER_ADVANCE, 182.0);
    assert_eq!(LOGIN_GLAYOUT_EN_MIN_WIDTH, 203.0);
    assert_eq!(LOGIN_GLAYOUT_EN_X, 83.0);
    assert_eq!(LOGIN_GLAYOUT_BUTTON_HEIGHT, 25.0);
    assert_eq!(LOGIN_GLAYOUT_BUTTON_GAP, 9.0);
    assert_eq!(LOGIN_GLAYOUT_REGISTER_Y, 234.0);

    let viewport = Vec2::new(1264.0, 681.0);
    let fit = LoginBackgroundMode0104::FallbackScaleToFit
        .resolved_rect(viewport)
        .expect("fit rectangle");
    assert!((fit.origin.x - 0.0).abs() < 0.001);
    assert!((fit.origin.y - 7.383_331_3).abs() < 0.001);
    assert!((fit.size.x - 1264.0).abs() < 0.001);
    assert!((fit.size.y - 666.233_34).abs() < 0.001);
    let crop = LoginBackgroundMode0104::LoadedScaleAndCrop
        .resolved_rect(viewport)
        .expect("crop rectangle");
    assert!((crop.origin.x - 0.0).abs() < 0.001);
    assert!((crop.origin.y + 15.0).abs() < 0.001);
    assert!((crop.size.x - 1264.0).abs() < 0.001);
    assert!((crop.size.y - 711.0).abs() < 0.001);
    let panel_origin = (viewport - LOGIN_PANEL_SIZE) * 0.5;
    assert_eq!(panel_origin, Vec2::new(447.0, 203.0));
}

#[test]
fn registration_matches_native_action_column_without_discord() {
    let mut app = spawned_app();
    let mut areas = app
        .world_mut()
        .query_filtered::<&Node, With<LoginGLayoutArea>>();
    let area = areas.single(app.world()).expect("GUILayout area");
    assert_eq!(area.position_type, PositionType::Absolute);
    assert_eq!(area.left, px(0));
    assert_eq!(area.top, px(221.0));
    assert_eq!(area.width, px(LOGIN_PANEL_SIZE.x));
    assert_eq!(area.justify_content, JustifyContent::Default);

    let mut columns = app
        .world_mut()
        .query_filtered::<&Node, With<LoginGLayoutColumn>>();
    let column = columns.single(app.world()).expect("GUILayout column");
    assert_eq!(column.min_width, px(280.0));
    assert_eq!(column.width, px(280.0));
    assert_eq!(column.flex_grow, 0.0);
    assert_eq!(column.flex_direction, FlexDirection::Column);
    assert_eq!(column.align_items, AlignItems::Stretch);
    assert_eq!(column.row_gap, px(LOGIN_GLAYOUT_BUTTON_GAP));

    let mut spaces = app
        .world_mut()
        .query_filtered::<&Node, With<LoginGLayoutFlexibleSpace>>();
    let spaces = spaces.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(spaces.len(), 2);
    let mut grow = spaces.iter().map(|node| node.flex_grow).collect::<Vec<_>>();
    grow.sort_by(f32::total_cmp);
    assert_eq!(
        grow,
        [LOGIN_GLAYOUT_LEADING_FLEX, LOGIN_GLAYOUT_TRAILING_FLEX]
    );
    assert!(spaces.iter().all(|node| node.flex_basis == px(0)));

    let mut buttons = app.world_mut().query::<(
        &Node,
        Option<&LoginCommunityButton>,
        Option<&LoginRegisterButton>,
    )>();
    let lower = buttons
        .iter(app.world())
        .filter(|(_, community, register)| community.is_some() || register.is_some())
        .collect::<Vec<_>>();
    assert_eq!(lower.len(), 1);
    assert!(lower[0].1.is_none(), "Discord must not be spawned");
    assert!(lower.iter().all(|(node, _, _)| {
        node.width == Val::Auto && node.height == px(35.0)
    }));
}

#[test]
fn saved_account_requires_explicit_matching_selection_before_submit() {
    let mut model = LoginUiModel { username: "player".into(), ..default() };
    let mut outbox = LoginUiOutbox::default();
    queue_login(&mut model, &mut outbox);
    assert!(outbox.requests.is_empty());
    model.saved_account = Some("other".into());
    queue_login(&mut model, &mut outbox);
    assert!(outbox.requests.is_empty());
    model.saved_account = Some("player".into());
    queue_login(&mut model, &mut outbox);
    assert_eq!(outbox.requests.len(), 1);
    assert!(model.busy);
    assert!(outbox.requests[0].password.is_empty());
}

#[test]
fn account_list_selects_without_login_and_edit_restores_credentials() {
    let mut app = spawned_app();
    let mut browser = LoginBrowser::default();
    browser.selected = 0;
    browser.accounts = vec![browser::SavedAccount { server: browser.servers[0].key().into(), username: "saved-player".into() }];
    app.insert_resource(browser).insert_resource(OptionUiModel::default())
        .insert_resource(LoginUiModel { visible: true, ..default() })
        .init_resource::<account_view::AccountForm>()
        .init_resource::<LoginUiOutbox>()
        .add_systems(Update, (account_view::interactions, account_view::bind).chain());
    app.update();
    assert_eq!(app.world().resource::<LoginUiModel>().saved_account.as_deref(), Some("saved-player"));
    let select = app.world_mut().spawn((Interaction::Pressed, account_view::Action::Select("saved-player".into()))).id();
    app.update();
    assert!(!app.world().resource::<LoginUiModel>().busy);
    assert!(app.world().resource::<LoginUiOutbox>().requests.is_empty());
    let mut menus = app.world_mut().query_filtered::<&Node, With<account_view::AccountMenu>>();
    let menu = menus.single(app.world()).unwrap();
    assert_eq!(menu.height, percent(100));
    assert_eq!(menu.overflow, Overflow::scroll_y());
    app.world_mut().entity_mut(select).despawn();
    app.world_mut().spawn((Interaction::Pressed, account_view::Action::Edit("saved-player".into())));
    app.update();
    let model = app.world().resource::<LoginUiModel>();
    assert!(model.saved_account.is_none());
    assert_eq!(model.username, "saved-player");
    assert!(model.password.is_empty());
    assert_eq!(model.focused, LoginField::Password);
    let mut fields = app.world_mut().query_filtered::<&Node, With<LoginPasswordField>>();
    assert_eq!(fields.single(app.world()).unwrap().display, Display::Flex);
    let mut labels = app.world_mut().query_filtered::<&Node, With<account_view::CredentialLabel>>();
    assert!(labels.iter(app.world()).all(|n| n.display == Display::Flex));
    app.world_mut().spawn((Interaction::Pressed, account_view::Action::Back));
    app.update();
    assert_eq!(app.world().resource::<LoginUiModel>().saved_account.as_deref(), Some("saved-player"));
}

#[test]
fn browser_hover_rebinds_slice_border_with_the_texture() {
    let mut app = spawned_app();
    app.init_resource::<LoginBrowser>().add_systems(Update, browser_view::button_states);
    let entity = {
        let assets = app.world().resource::<LoginUiAssets>();
        let image = browser_view::field_image(assets);
        app.world_mut().spawn((Interaction::Hovered, browser_view::Action::OpenAddServer, image)).id()
    };
    app.update();
    let image = app.world().get::<ImageNode>(entity).unwrap();
    let NodeImageMode::Sliced(slicer) = &image.image_mode else { panic!("sliced button"); };
    assert_eq!(slicer.border, LOGIN_BUTTON_BORDER);
    app.world_mut().entity_mut(entity).insert(Interaction::None);
    app.update();
    let image = app.world().get::<ImageNode>(entity).unwrap();
    let NodeImageMode::Sliced(slicer) = &image.image_mode else { panic!("sliced field"); };
    assert_eq!(slicer.border, LOGIN_TEXT_FIELD_BORDER);
}

#[test]
fn server_address_keeps_input_skin_and_add_uses_login_button_states() {
    let mut app = spawned_app();
    app.init_resource::<LoginBrowser>().add_systems(Update, browser_view::button_states);
    let assets = app.world().resource::<LoginUiAssets>().clone();
    let input = app.world_mut().spawn((Interaction::Hovered, browser_view::Action::ServerInput,
        browser_view::field_image(&assets))).id();
    let add = app.world_mut().spawn((Interaction::None, browser_view::Action::AddServer,
        browser_view::field_image(&assets))).id();
    app.update();
    assert_eq!(app.world().get::<ImageNode>(input).unwrap().image, assets.text_field);
    for (state, expected) in [(Interaction::None, assets.button),
        (Interaction::Hovered, assets.button_over), (Interaction::Pressed, assets.button_active)] {
        app.world_mut().entity_mut(add).insert(state);
        app.update();
        let image = app.world().get::<ImageNode>(add).unwrap();
        assert_eq!(image.image, expected);
        let NodeImageMode::Sliced(slicer) = &image.image_mode else {panic!("button slice");};
        assert_eq!(slicer.border, LOGIN_BUTTON_BORDER);
        assert_eq!(app.world().get::<ImageNode>(input).unwrap().image, assets.text_field);
    }
}
