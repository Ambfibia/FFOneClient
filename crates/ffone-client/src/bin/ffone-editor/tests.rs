use super::*;

#[test]
fn preview_viewport_excludes_all_editor_chrome() {
    let (position, size) = preview_viewport_logical_rect(Vec2::new(1600.0, 940.0)).unwrap();
    assert_eq!(position, Vec2::new(348.0, 190.0));
    assert_eq!(size, Vec2::new(874.0, 594.0));

    let (minimum_position, minimum_size) =
        preview_viewport_logical_rect(Vec2::new(1180.0, 720.0)).unwrap();
    assert_eq!(minimum_position, position);
    assert_eq!(minimum_size, Vec2::new(454.0, 374.0));
}

#[test]
fn editor_game_images_use_nine_slice_contracts() {
    let image = sliced_image(Handle::default(), OPTION_BIG_LABEL_BORDER);
    let NodeImageMode::Sliced(slicer) = image.image_mode else {
        panic!("editor game texture must not use Stretch");
    };
    assert_eq!(slicer.border, OPTION_BIG_LABEL_BORDER);
    assert_eq!(slicer.center_scale_mode, SliceScaleMode::Stretch);
    assert_eq!(slicer.sides_scale_mode, SliceScaleMode::Stretch);
    assert_eq!(slicer.max_corner_scale, 1.0);
}

fn project_assets() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game")
}

#[test]
fn production_catalog_resolves_xdt_npcs_and_nanos() {
    let locator = AssetLocator::open(project_assets()).unwrap();
    let mut catalog = EditorCatalog::open(&locator).unwrap();
    assert!(
        catalog.npc_count > 100,
        "expected broad native NPC coverage"
    );
    assert!(
        catalog.nano_count > 30,
        "expected broad native Nano coverage"
    );
    let content = TutorialMissionContent::open(&locator).unwrap();
    assert_eq!(catalog.npc_count, content.gameplay_npcs().len());
    let registry: CharacterRegistryDocument = locator.read_character_models().unwrap();
    assert_eq!(
        catalog.nano_count,
        registry
            .models
            .iter()
            .filter(|model| model.category == "nano")
            .count()
    );
    let nanos = catalog
        .entries
        .iter()
        .filter(|entry| entry.kind == CatalogKind::Nano)
        .collect::<Vec<_>>();
    assert!(
        nanos
            .iter()
            .all(|entry| !entry.glb.is_empty() && entry.icon_path.is_some())
    );
    assert_eq!(
        nanos
            .iter()
            .map(|entry| &entry.glb)
            .collect::<BTreeSet<_>>()
            .len(),
        nanos.len()
    );
    for (name, logical, icon) in [
        ("Coop", "nano_coop", "coop"),
        ("Unstable Nano", "nano_holonano", "holo-nano"),
        ("Van Kleiss", "nano_vankleiss", "van-kleiss"),
        ("Upgrade", "nano_upgrade", "upgrade"),
        ("Ghostfreak", "nano_ghostfreak", "ghostfreak"),
        ("Swampfire", "nano_swampfire", "swampfire"),
        ("Computress", "nano_computress", "computress"),
    ] {
        let matches = nanos
            .iter()
            .filter(|entry| entry.display_name == name)
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "{name} must have one browser entry");
        assert_eq!(matches[0].logical_name, logical);
        assert_eq!(
            matches[0].icon_path.as_deref(),
            Some(format!("icons/entities/nanos/nanoicon_{icon}.png").as_str())
        );
    }
    assert!(
        catalog
            .entries
            .iter()
            .any(|entry| { entry.kind == CatalogKind::Npc && entry.icon_path.is_some() })
    );
    for kind in [CatalogKind::Npc, CatalogKind::Nano] {
        let ids = catalog
            .entries
            .iter()
            .filter(|entry| entry.kind == kind)
            .filter_map(|entry| entry.network_id)
            .collect::<Vec<_>>();
        assert!(
            ids.windows(2).all(|pair| pair[0] < pair[1]),
            "{kind:?} XDT rows must stay strictly sorted by numeric ID"
        );
    }
    assert!(catalog.entries.iter().any(|entry| {
        entry.kind == CatalogKind::Npc
            && entry.network_id.is_some()
            && entry.animations.iter().any(|clip| clip == "stand1")
    }));
    let fusion_eduardo = catalog
        .entries
        .iter()
        .find(|entry| entry.kind == CatalogKind::Npc && entry.network_id == Some(1))
        .expect("XDT NPC 1");
    assert_eq!(fusion_eduardo.semantic_id, "fusion/fusion_eduardo");
    assert!(
        fusion_eduardo
            .animations
            .iter()
            .any(|clip| clip == "stand1")
            && fusion_eduardo.animations.iter().any(|clip| clip == "run"),
        "exact-GLB animation lookup must include fusion/mob/shared registry categories"
    );

    assert!(catalog.entries.iter().any(|entry| {
        entry.kind == CatalogKind::Nano
            && entry.network_id.is_some()
            && entry.animations.iter().any(|clip| clip == "call")
    }));

    assert!(catalog.entries.iter().any(|entry| {
        !entry.glb.is_empty()
            && entry.animations.is_empty()
            && matches!(
                entry.semantic_id.as_str(),
                "npc/npc_key" | "npc/npc_timepod"
            )
    }));

    let mut state = EditorState::new(&catalog);
    assert_eq!(state.pose_mode, EditorPoseMode::Default);
    assert!(state.paused, "default pose must be frozen on frame zero");
    assert!(!state.turntable, "automatic model rotation must be opt-in");
    assert_eq!(
        catalog.entries[state.selected].animations[state.clip_index],
        "stand1"
    );

    state.activate_t_pose();
    assert_eq!(state.pose_mode, EditorPoseMode::TPose);
    assert!(state.paused);

    state.activate_default_pose(&catalog);
    assert_eq!(state.pose_mode, EditorPoseMode::Default);
    assert!(state.paused);

    state.toggle_playback(&catalog);
    assert_eq!(state.pose_mode, EditorPoseMode::Clip);
    assert!(!state.paused);
}

#[test]
fn t_pose_correction_aligns_arm_segments_horizontally() {
    let parent = Quat::from_rotation_y(0.2);
    let local = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_4);
    let global = parent * local;
    let direction = global * Vec3::X;
    let corrected =
        horizontal_t_pose_rotation(parent, global, direction).expect("non-zero arm direction");
    let corrected_direction = (parent * corrected) * Vec3::X;

    assert!(
        corrected_direction.normalize().dot(Vec3::X) > 0.9999,
        "corrected arm must be horizontal: {corrected_direction:?}"
    );
    assert_eq!(t_pose_child_segment("Bip01 R UpperArm"), Some("forearm"));
    assert_eq!(t_pose_child_segment("Bip01 L Forearm"), Some("hand"));
}

#[test]
fn logical_names_become_readable_labels() {
    assert_eq!(humanize_name("npc_dexter"), "Dexter");
    assert_eq!(humanize_name("nano_samuraijack"), "Samuraijack");
}

#[test]
fn catalog_keyboard_navigation_filters_all_kinds_and_reveals_selection() {
    for kind in [CatalogKind::Npc, CatalogKind::Nano, CatalogKind::Equipment] {
        let mut catalog =
            EditorCatalog::open(&AssetLocator::open(project_assets()).unwrap()).unwrap();
        let template = catalog.entries[0].clone();
        catalog.entries = ["First match", "Excluded", "Last match"]
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let mut entry = template.clone();
                entry.kind = kind;
                entry.display_name = name.to_owned();
                entry.semantic_id = format!("equipment/shirt/{index}");
                entry.logical_name = format!("model_{index}");
                entry
            })
            .collect();
        let mut state = EditorState::new(&catalog);
        state.search = "match".to_owned();
        state.search_focused = true;
        state.equipment_category = Some(ffone_runtime_contracts::AvatarItemCategory::Shirt);
        let mut app = App::new();
        app.insert_resource(state)
            .insert_resource(catalog)
            .init_resource::<ModelPreview>()
            .init_resource::<OrbitCamera>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<hnpc::HnpcEditor>()
            .insert_resource(icon_generator::IconGenerator::new(project_assets()))
            .add_systems(
                Update,
                (handle_editor_shortcuts, reveal_keyboard_selection).chain(),
            );
        let viewport = app
            .world_mut()
            .spawn((
                CatalogScroll,
                ScrollPosition::default(),
                ComputedNode {
                    size: Vec2::new(250.0, 60.0),
                    inverse_scale_factor: 1.0,
                    ..default()
                },
            ))
            .id();
        let press = |app: &mut App, key| {
            let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keyboard.reset_all();
            keyboard.press(key);
            app.update();
        };
        press(&mut app, KeyCode::ArrowDown);
        assert_eq!(app.world().resource::<EditorState>().selected, 2);
        assert_eq!(app.world().get::<ScrollPosition>(viewport).unwrap().y, 60.0);
        assert_eq!(app.world().resource::<EditorState>().search, "match");
        press(&mut app, KeyCode::ArrowDown);
        assert_eq!(app.world().resource::<EditorState>().selected, 2);
        press(&mut app, KeyCode::ArrowUp);
        assert_eq!(app.world().resource::<EditorState>().selected, 0);
        assert_eq!(app.world().get::<ScrollPosition>(viewport).unwrap().y, 0.0);
        press(&mut app, KeyCode::ArrowUp);
        assert_eq!(app.world().resource::<EditorState>().selected, 0);
        app.world_mut().resource_mut::<EditorState>().search = "no results".to_owned();
        press(&mut app, KeyCode::ArrowDown);
        assert_eq!(app.world().resource::<EditorState>().selected, 0);
        app.world_mut().resource_mut::<EditorState>().search = "Last match".to_owned();
        press(&mut app, KeyCode::ArrowDown);
        assert_eq!(app.world().resource::<EditorState>().selected, 2);
    }
}

#[test]
fn catalog_scroll_handles_rows_dragging_bounds_and_filter_changes() {
    let catalog = EditorCatalog::open(&AssetLocator::open(project_assets()).unwrap()).unwrap();
    let mut app = App::new();
    app.insert_resource(EditorState::new(&catalog))
        .init_resource::<ButtonInput<MouseButton>>()
        .add_message::<MouseWheel>()
        .add_systems(Update, scroll_catalog_list);
    let viewport = app
        .world_mut()
        .spawn((
            CatalogScroll,
            ScrollPosition::default(),
            Interaction::None, // a child row owns hover
            RelativeCursorPosition {
                cursor_over: true,
                normalized: Some(Vec2::ZERO),
            },
            ComputedNode {
                size: Vec2::new(250.0, 500.0),
                content_size: Vec2::new(250.0, 2000.0),
                inverse_scale_factor: 1.0,
                ..default()
            },
        ))
        .id();
    let track = app
        .world_mut()
        .spawn((
            CatalogScrollbar,
            RelativeCursorPosition::default(),
            ComputedNode {
                size: Vec2::new(12.0, 500.0),
                inverse_scale_factor: 1.0,
                ..default()
            },
        ))
        .id();
    app.update();
    let wheel = |app: &mut App, unit, y| {
        app.world_mut().write_message(MouseWheel {
            phase: bevy::input::touch::TouchPhase::Moved,
            unit,
            x: 0.0,
            y,
            window: Entity::PLACEHOLDER,
        });
    };
    wheel(&mut app, MouseScrollUnit::Line, -2.0);
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(viewport).unwrap().y, 88.0);
    wheel(&mut app, MouseScrollUnit::Pixel, -10000.0);
    app.update();
    assert_eq!(
        app.world().get::<ScrollPosition>(viewport).unwrap().y,
        1500.0
    );
    app.world_mut()
        .get_mut::<RelativeCursorPosition>(viewport)
        .unwrap()
        .cursor_over = false;
    wheel(&mut app, MouseScrollUnit::Line, 2.0);
    app.update();
    assert_eq!(
        app.world().get::<ScrollPosition>(viewport).unwrap().y,
        1500.0
    );
    *app.world_mut()
        .get_mut::<RelativeCursorPosition>(track)
        .unwrap() = RelativeCursorPosition {
        cursor_over: true,
        normalized: Some(Vec2::ZERO),
    };
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    assert_eq!(
        app.world().get::<ScrollPosition>(viewport).unwrap().y,
        750.0
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .get_mut::<RelativeCursorPosition>(track)
        .unwrap()
        .normalized = Some(Vec2::new(0.0, 0.7));
    app.update();
    assert_eq!(
        app.world().get::<ScrollPosition>(viewport).unwrap().y,
        1500.0
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.world_mut().resource_mut::<EditorState>().search = "Bubbles".to_owned();
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(viewport).unwrap().y, 0.0);
    assert_eq!(catalog_scroll_thumb(20.0, 0.0, 500.0), (0.0, 20.0));
}

#[test]
fn published_nano_models_have_native_contracts_and_exact_faces() {
    let locator = AssetLocator::open(project_assets()).unwrap();
    let registry: CharacterRegistryDocument = locator.read_character_models().unwrap();
    // A semantic identity need not equal the GLB root or filename:
    // Belladonna's native root is nano_belladonnaN.
    let native_path = |name: &str| {
        let id = format!("nano/nano_{name}");
        let model = registry.models.iter().find(|model| model.id == id).unwrap();
        project_assets().join(&model.glb)
    };
    for name in [
        "belladonna",
        "bubbles",
        "dexter",
        "buttercup",
        "ben",
        "eduardo",
        "utonium",
        "jackolantern",
        "ghostfreak",
        "computress",
        "swampfire",
        "runty",
    ] {
        let bytes = fs::read(native_path(name)).unwrap();
        ffone_skinned_model::validate_glb_render_contract(&bytes).unwrap();
        let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let metadata = std::str::from_utf8(&bytes[20..20 + length]).unwrap();
        for legacy_identity in [
            "\"pathId\"",
            "\"fileId\"",
            "CustomAssetBundle",
            ".resourceFile",
        ] {
            assert!(
                !metadata.contains(legacy_identity),
                "{name}: {legacy_identity}"
            );
        }
    }
    for (name, role, texture) in [
        ("belladonna", "main", "nano_belladonna"),
        ("bubbles", "main", "nano_bubbles"),
        ("bubbles", "sub", "nano_bubbles_face"),
        ("dexter", "sub", "nano_dexter_face"),
        ("buttercup", "sub", "nano_buttercup_face"),
        ("ben", "sub", "nano_ben_face"),
        ("eduardo", "sub", "nano_eduardo_face"),
        ("utonium", "sub", "nano_utonium_face"),
        ("jackolantern", "main", "nano_jackolantern"),
        ("computress", "main", "nano_computress"),
        ("runty", "main", "nano_runty"),
    ] {
        let path = native_path(name);
        let bytes = fs::read(&path).unwrap();
        let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let glb: Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
        let material = glb["materials"]
            .as_array()
            .unwrap()
            .iter()
            .find(|material| material["name"].as_str().unwrap().contains(role))
            .unwrap();
        let binding = material["extras"]["ffone"]["textureBindings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|binding| binding["slot"] == "_MainTex")
            .unwrap();
        assert_eq!(binding["sourceName"], texture, "{name} {role}");
        assert!(
            path.parent()
                .unwrap()
                .join(binding["uri"].as_str().unwrap())
                .is_file()
        );
    }
}
