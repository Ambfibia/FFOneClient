use crate::character_creation_data::*;

fn production_data() -> CharacterCreationData {
    CharacterCreationData::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"))
        .expect("open production character-creation data")
}

#[test]
fn expanded_palettes_resolve_every_new_eye_for_both_creator_genders() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let data = CharacterCreationData::open(root).unwrap();
    assert_eq!(
        data.ui_palettes().map(|palette| palette.len()),
        [36, 54, 10]
    );
    for gender in [UiGender::Boy, UiGender::Girl] {
        for face in 2..=6 {
            for eye_color in 6..=10 {
                let appearance = CharacterAppearance {
                    gender,
                    face,
                    skin_color: 36,
                    hair_color: 54,
                    eye_color,
                    ..Default::default()
                };
                let resolved = data
                    .resolve_creator(1, 0, "Test", "Player", &appearance)
                    .unwrap();
                assert_eq!(resolved.style.eye_color, eye_color as i8);
                assert_eq!(resolved.style.skin_color, 36);
                assert_eq!(resolved.style.hair_color, 54);
            }
        }
    }
}

#[test]
fn production_documents_are_manifest_verified_and_exactly_sized() {
    let data = production_data();
    assert_eq!(data.names_document().first_names.len(), 601);
    assert_eq!(data.names_document().middle_names.len(), 601);
    assert_eq!(data.names_document().last_names.len(), 602);
    assert_eq!(data.appearance_document().creation_rows.len(), 32);
    assert_eq!(data.avatar_items_document().items.len(), 3_981);
}

#[test]
fn default_creator_resolves_to_network_payload_and_native_player_look() {
    let data = production_data();
    let resolved = data
        .resolve_creator(0, 0, "Test", "Hero", &CharacterAppearance::default())
        .expect("resolve exact default creator");
    assert_eq!(resolved.style.gender, 1);
    assert_eq!(resolved.selected_indices.face_style_index, 2);
    assert_eq!(resolved.selected_indices.hair_style_index, 2);
    resolved.look.validate().expect("valid native player look");
    assert_eq!(
        data.appearance_label(&CharacterAppearance::default(), AppearanceField::Hair)
            .unwrap(),
        "RAZOR CUT"
    );
    for field in [
        AppearanceField::Shirt,
        AppearanceField::Pants,
        AppearanceField::Shoes,
    ] {
        let icons = data
            .clothing_icon_window(&CharacterAppearance::default(), field)
            .expect("five exact native creator icons");
        assert_eq!(icons.len(), 5);
        assert!(icons.iter().all(|path| path.ends_with(".png")));
    }
}

#[test]
fn live_pc_appearance_resolves_the_same_shared_rig_contract_as_creator_data() {
    let data = production_data();
    let creator = data
        .resolve_creator(700, 1, "Remote", "Player", &CharacterAppearance::default())
        .expect("resolve creator source look");
    let mut equipment = [ffone_protocol::ItemBase0104 {
        item_type: 0,
        item_id: 0,
        option: 0,
        time_limit: 0,
    }; 9];
    equipment[CharacterEquipSlot0104::UpperBody as usize].item_id =
        creator.equipped.upper_body_id;
    equipment[CharacterEquipSlot0104::LowerBody as usize].item_id =
        creator.equipped.lower_body_id;
    equipment[CharacterEquipSlot0104::Foot as usize].item_id = creator.equipped.foot_id;
    let appearance = PcAppearance0104 {
        id: 77,
        style: creator.style,
        condition_bit_flag: 0,
        pc_state: 1,
        special_state: 0,
        level: 1,
        hp: 1_000,
        map_number: 0,
        position: [0, 0, 0],
        angle: 0,
        equipment,
        nano: ffone_protocol::Nano0104 {
            id: 0,
            skill_id: 0,
            stamina: 0,
        },
        render_type: 0,
    };

    let mut live = data
        .resolve_pc_appearance(&appearance)
        .expect("resolve live PC appearance");
    live.identity = creator.look.identity.clone();
    assert_eq!(live, creator.look);
}

#[test]
fn authoritative_empty_apparel_slots_resolve_exact_gendered_naked_routes() {
    let data = production_data();
    for (gender, prefix) in [(UiGender::Boy, "m"), (UiGender::Girl, "f")] {
        let creator = data
            .resolve_creator(
                0,
                0,
                "Naked",
                "Route",
                &CharacterAppearance {
                    gender,
                    ..CharacterAppearance::default()
                },
            )
            .expect("resolve valid source style");
        let look = data
            .resolve_protocol_player_look(
                format!("naked-{prefix}"),
                creator.style.gender,
                creator.style.face_style,
                creator.style.hair_style,
                creator.style.hair_color,
                creator.style.skin_color,
                creator.style.eye_color,
                creator.style.height,
                creator.style.body,
                |_| 0,
            )
            .unwrap_or_else(|error| panic!("{prefix} naked look failed: {error}"));

        for (kind, route) in [
            (
                NativePlayerPartKind::Shirt,
                format!("wear/{prefix}_shirt_naked.nif"),
            ),
            (
                NativePlayerPartKind::Pants,
                format!("wear/{prefix}_pants_naked.nif"),
            ),
            (
                NativePlayerPartKind::Shoes,
                format!("wear/{prefix}_shoes_naked.nif"),
            ),
        ] {
            assert_eq!(
                look.parts
                    .iter()
                    .find(|part| part.kind == kind)
                    .map(|part| part.exact_route.as_str()),
                Some(route.as_str())
            );
        }
    }
}

#[test]
fn protocol_head_face_and_rigid_back_slots_resolve_exact_attachment_kinds() {
    let data = production_data();
    let creator = data
        .resolve_creator(0, 0, "Rigid", "Routes", &CharacterAppearance::default())
        .expect("resolve valid source style");
    let look = data
        .resolve_protocol_player_look(
            "rigid-routes".to_owned(),
            creator.style.gender,
            creator.style.face_style,
            creator.style.hair_style,
            creator.style.hair_color,
            creator.style.skin_color,
            creator.style.eye_color,
            creator.style.height,
            creator.style.body,
            |slot| match slot {
                CharacterEquipSlot0104::Head
                | CharacterEquipSlot0104::Face
                | CharacterEquipSlot0104::Back => 1,
                _ => 0,
            },
        )
        .expect("resolve production rigid attachment routes");

    for kind in [
        NativePlayerPartKind::Hat,
        NativePlayerPartKind::Glasses,
        NativePlayerPartKind::Back,
    ] {
        let part = look
            .parts
            .iter()
            .find(|part| part.kind == kind)
            .unwrap_or_else(|| panic!("missing {kind:?} rigid attachment"));
        assert!(part.exact_route.starts_with("wear/"));
        assert!(part.glb.ends_with(".glb"));
        assert!(!part.uses_shared_skin());
        assert!(
            part.primary_texture.is_some(),
            "{kind:?} must receive the table-driven AttachGO main texture"
        );
    }
    assert_eq!(
        look.parts
            .iter()
            .filter(|part| {
                matches!(
                    part.kind,
                    NativePlayerPartKind::Hat
                        | NativePlayerPartKind::Glasses
                        | NativePlayerPartKind::Back
                )
            })
            .count(),
        3
    );

    let empty = data
        .resolve_protocol_player_look(
            "no-rigid-routes".to_owned(),
            creator.style.gender,
            creator.style.face_style,
            creator.style.hair_style,
            creator.style.hair_color,
            creator.style.skin_color,
            creator.style.eye_color,
            creator.style.height,
            creator.style.body,
            |_| 0,
        )
        .expect("resolve empty rigid attachment slots");
    assert!(empty.parts.iter().all(|part| !matches!(
        part.kind,
        NativePlayerPartKind::Hat | NativePlayerPartKind::Glasses | NativePlayerPartKind::Back
    )));
}

#[test]
fn every_available_back_row_uses_its_legacy_equip_type_assembly() {
    let data = production_data();
    let mut verified = 0_usize;
    for (gender, data_gender) in [
        (PlayerRigGender::Male, DataGender::Male),
        (PlayerRigGender::Female, DataGender::Female),
    ] {
        for item in data
            .items
            .values()
            .filter(|item| item.category == AvatarItemCategory::Back)
        {
            let visual = match gender {
                PlayerRigGender::Male => &item.male,
                PlayerRigGender::Female => &item.female,
            };
            if visual.model_status == NativeLookupStatus::Missing {
                continue;
            }
            let part = data
                .resolve_equipment_part(
                    AvatarItemCategory::Back,
                    item.item_number,
                    data_gender,
                    NativePlayerPartKind::Back,
                )
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to resolve {:?} Back item {} ({:?}): {error}",
                        gender, item.item_number, item.name
                    )
                });
            assert_eq!(
                part.uses_shared_skin(),
                item.equip_type == 1,
                "Back item {} ({:?}) ignored its legacy equipType {}",
                item.item_number,
                item.name,
                item.equip_type
            );
            verified += 1;
        }
    }
    assert!(verified > 300, "expected the complete gendered Back table");
}

#[test]
fn protocol_look_carries_every_supported_weapon_animation_family() {
    let data = production_data();
    let creator = data
        .resolve_creator(0, 0, "Weapon", "Profile", &CharacterAppearance::default())
        .expect("resolve source style");
    for (item_number, expected) in [
        (43, PlayerWeaponAnimationProfile::Stick),
        (197, PlayerWeaponAnimationProfile::Pistol),
        (328, PlayerWeaponAnimationProfile::Rifle),
        (1, PlayerWeaponAnimationProfile::Bomb),
        (365, PlayerWeaponAnimationProfile::Rocket),
    ] {
        let look = data
            .resolve_protocol_player_look(
                format!("weapon-profile-{item_number}"),
                creator.style.gender,
                creator.style.face_style,
                creator.style.hair_style,
                creator.style.hair_color,
                creator.style.skin_color,
                creator.style.eye_color,
                creator.style.height,
                creator.style.body,
                |slot| {
                    if slot == CharacterEquipSlot0104::Hand {
                        item_number
                    } else {
                        0
                    }
                },
            )
            .unwrap_or_else(|error| {
                panic!("failed to resolve weapon item {item_number}: {error}")
            });
        assert_eq!(look.weapon_animation_profile, Some(expected));
        assert!(look.parts.iter().any(|part| {
            part.kind == NativePlayerPartKind::Weapon && !part.uses_shared_skin()
        }));
    }
}

#[test]
fn legacy_hat_equip_type_matrix_matches_primary_set_hat_contract() {
    let expected = [
        LegacyHatPolicy {
            face_variant: 1,
            hair_variant: Some(1),
            glasses_visible: true,
        },
        LegacyHatPolicy {
            face_variant: 1,
            hair_variant: Some(2),
            glasses_visible: true,
        },
        LegacyHatPolicy {
            face_variant: 2,
            hair_variant: None,
            glasses_visible: true,
        },
        LegacyHatPolicy {
            face_variant: 2,
            hair_variant: None,
            glasses_visible: false,
        },
        LegacyHatPolicy {
            face_variant: 1,
            hair_variant: Some(1),
            glasses_visible: false,
        },
        LegacyHatPolicy {
            face_variant: 1,
            hair_variant: Some(2),
            glasses_visible: false,
        },
    ];
    for (equip_type, expected) in expected.into_iter().enumerate() {
        assert_eq!(
            LegacyHatPolicy::from_equip_type(equip_type as u8).unwrap(),
            expected
        );
    }
    assert!(LegacyHatPolicy::from_equip_type(6).is_err());
}

#[test]
fn football_helmet_removes_hair_and_binds_the_screenshot_item_textures() {
    let data = production_data();
    let football = data
        .items
        .get(&(AvatarItemCategory::Hat, 46))
        .expect("Football Helmet table row");
    assert_eq!(football.name, "Football Helmet");
    assert_eq!(football.equip_type, 2);

    let creator = data
        .resolve_creator(0, 0, "Texture", "Sentinel", &CharacterAppearance::default())
        .expect("resolve source style");
    let look = data
        .resolve_protocol_player_look(
            "screenshot-items".to_owned(),
            creator.style.gender,
            creator.style.face_style,
            creator.style.hair_style,
            creator.style.hair_color,
            creator.style.skin_color,
            creator.style.eye_color,
            creator.style.height,
            creator.style.body,
            |slot| match slot {
                CharacterEquipSlot0104::Head => 46,
                CharacterEquipSlot0104::Back => 31,
                CharacterEquipSlot0104::Hand => 200,
                _ => 0,
            },
        )
        .expect("resolve exact screenshot equipment");

    assert!(
        look.parts
            .iter()
            .all(|part| part.kind != NativePlayerPartKind::Hair),
        "equipType 2 must remove hair entirely"
    );
    assert!(
        look.parts
            .iter()
            .find(|part| part.kind == NativePlayerPartKind::Face)
            .is_some_and(|part| part.glb.contains("_type02/")),
        "equipType 2 must select the type02 face"
    );
    for (kind, texture) in [
        (NativePlayerPartKind::Hat, "m_halmet_football.png"),
        (NativePlayerPartKind::Back, "back_octibackpack.png"),
        (NativePlayerPartKind::Weapon, "bazooka_toybazooka.png"),
    ] {
        let part = look
            .parts
            .iter()
            .find(|part| part.kind == kind)
            .unwrap_or_else(|| panic!("missing screenshot {kind:?}"));
        assert!(
            part.primary_texture
                .as_ref()
                .is_some_and(|primary| primary.path.ends_with(texture)),
            "{kind:?} did not bind {texture}"
        );
    }
}

#[test]
fn recovered_primary_girl_skirts_resolve_from_the_current_item_catalog() {
    let data = production_data();
    for (item_number, expected_true_name) in
        [(160, "f_pants_gothgirl"), (315, "f_pants_stylistdandy")]
    {
        let item = data
            .items
            .get(&(AvatarItemCategory::Pants, item_number))
            .unwrap_or_else(|| panic!("missing primary pants item {item_number}"));
        assert_eq!(item.female.model_status, NativeLookupStatus::VerifiedUnique);
        let [model] = item.female.models.as_slice() else {
            panic!("primary pants item {item_number} did not resolve uniquely");
        };
        assert_eq!(model.true_name, expected_true_name);
        assert_eq!(model.exact_route, format!("wear/{expected_true_name}.nif"));
    }
}

#[test]
fn academy_rath_mask_binds_its_own_texture_for_both_genders() {
    let data = production_data();
    let mask = &data.items[&(AvatarItemCategory::Hat, 449)];
    assert_eq!(mask.name, "Rath Mask");
    assert_eq!(mask.equip_type, 3);
    assert_eq!(mask.icon.as_ref().unwrap().status, NativeLookupStatus::VerifiedUnique);
    assert!(mask.icon.as_ref().unwrap().candidates[0].path.ends_with("/cosicon_2020.png"));
    for gender in [UiGender::Boy, UiGender::Girl] {
        let creator = data.resolve_creator(0, 0, "Rath", "Mask", &CharacterAppearance {
            gender, ..CharacterAppearance::default()
        }).unwrap();
        let look = data.resolve_protocol_player_look(
            "rath-mask".into(), creator.style.gender, creator.style.face_style,
            creator.style.hair_style, creator.style.hair_color, creator.style.skin_color,
            creator.style.eye_color, creator.style.height, creator.style.body,
            |slot| if slot == CharacterEquipSlot0104::Head {449} else {0},
        ).unwrap();
        let hat = look.parts.iter().find(|part| part.kind == NativePlayerPartKind::Hat).unwrap();
        assert_eq!(hat.exact_route, "wear/helmet_bigchillmask.nif");
        assert!(hat.primary_texture.as_ref().unwrap().path.ends_with("/helmet_rathmask.png"));
        assert!(look.parts.iter().all(|part| !matches!(part.kind, NativePlayerPartKind::Hair | NativePlayerPartKind::Glasses)));
        assert!(look.parts.iter().any(|part| part.kind == NativePlayerPartKind::Face && part.glb.contains("_type02/")));
    }
}

#[test]
fn every_verified_avatar_texture_route_is_audited_and_exact_publications_bind() {
    let data = production_data();
    let mut verified_routes = std::collections::BTreeSet::new();
    let mut published_routes = std::collections::BTreeSet::new();
    for item in &data.avatar_items_document().items {
        for visual in [&item.male, &item.female] {
            for texture in [&visual.primary_texture, &visual.secondary_texture]
                .into_iter()
                .flatten()
            {
                if texture.status != NativeLookupStatus::VerifiedUnique
                    || texture.candidates.len() != 1
                {
                    continue;
                }
                let reference = &texture.candidates[0];
                verified_routes.insert(reference.path.clone());
                let resolved =
                    data.published_texture_reference(texture)
                        .unwrap_or_else(|error| {
                            panic!(
                                "{:?} item {} texture {} failed audit: {error}",
                                item.category, item.item_number, texture.true_name
                            )
                        });
                if data.runtime_texture_contracts.contains_key(&reference.path) {
                    assert_eq!(
                        resolved.as_ref().map(|texture| texture.path.as_str()),
                        Some(reference.path.as_str())
                    );
                    published_routes.insert(reference.path.clone());
                } else {
                    assert!(resolved.is_none());
                }
            }
        }
    }
    let coverage = &data.runtime_textures_document().coverage;
    assert_eq!(
        verified_routes.len() as u64,
        coverage.avatar_verified_unique_routes
    );
    assert_eq!(
        published_routes.len() as u64,
        coverage.avatar_published_routes
    );
    assert_eq!(
        verified_routes.len() - published_routes.len(),
        coverage.avatar_deferred_verified_routes as usize
    );
    assert_eq!(coverage.avatar_verified_unique_routes, 2_772);
    assert_eq!(coverage.avatar_published_routes, 2_772);
    assert_eq!(coverage.avatar_deferred_verified_routes, 0);
    assert!(coverage.avatar_missing_source_metadata.is_empty());
    assert_eq!(data.runtime_textures_document().textures.len(), 2_932);
}

#[test]
fn every_gender_valid_wearable_has_a_resolved_model_and_texture_contract() {
    let data = production_data();
    for item in &data.avatar_items_document().items {
        if !matches!(
            item.category,
            AvatarItemCategory::Back
                | AvatarItemCategory::Glasses
                | AvatarItemCategory::Hat
                | AvatarItemCategory::Pants
                | AvatarItemCategory::Shirt
                | AvatarItemCategory::Shoes
        ) {
            continue;
        }
        for (gender, allowed, visual) in [
            ("male", item.required_gender != 2, &item.male),
            ("female", item.required_gender != 1, &item.female),
        ] {
            if !allowed {
                continue;
            }
            if visual.source_model_true_name.is_some() {
                assert!(
                    matches!(
                        visual.model_status,
                        NativeLookupStatus::VerifiedUnique
                            | NativeLookupStatus::VerifiedVariants
                    ) && !visual.models.is_empty(),
                    "{:?} item {} ({}) has no resolved {gender} model",
                    item.category,
                    item.item_number,
                    item.name
                );
            }
            for texture in [&visual.primary_texture, &visual.secondary_texture]
                .into_iter()
                .flatten()
            {
                assert_eq!(
                    texture.status,
                    NativeLookupStatus::VerifiedUnique,
                    "{:?} item {} ({}) texture {} is unresolved for {gender}",
                    item.category,
                    item.item_number,
                    item.name,
                    texture.true_name
                );
                assert_eq!(texture.candidates.len(), 1);
                assert!(
                    data.runtime_texture_contracts
                        .contains_key(&texture.candidates[0].path)
                );
            }
        }
    }
    for (category, item_number) in [
        (AvatarItemCategory::Back, 75),
        (AvatarItemCategory::Glasses, 95),
        (AvatarItemCategory::Hat, 204),
    ] {
        let item = data.items.get(&(category, item_number)).unwrap();
        assert!(item.male.primary_texture.is_some());
        assert!(item.female.primary_texture.is_some());
    }
}

#[test]
fn every_published_creator_choice_resolves_a_native_shared_rig_model() {
    let data = production_data();
    for choice in &data.appearance_document().choices {
        let category = match choice.category {
            CharacterAppearanceCategory::Face => AvatarItemCategory::Face,
            CharacterAppearanceCategory::Hair => AvatarItemCategory::Head,
            CharacterAppearanceCategory::Shirt => AvatarItemCategory::Shirt,
            CharacterAppearanceCategory::Pants => AvatarItemCategory::Pants,
            CharacterAppearanceCategory::Shoes => AvatarItemCategory::Shoes,
        };
        let item = data
            .items
            .get(&(category, choice.value))
            .expect("creator choice item");
        let visual = match choice.gender {
            DataGender::Male => &item.male,
            DataGender::Female => &item.female,
        };
        let model = data
            .resolve_model(item, visual, choice.gender, None)
            .unwrap_or_else(|error| {
                panic!(
                    "{:?}/{:?}/{} failed native resolution: {error}",
                    choice.gender, choice.category, choice.creation_index
                )
            });
        assert!(!model.exact_route.is_empty());
        assert!(model.native_asset.path.ends_with(".glb"));
    }
}

#[test]
fn every_published_creator_choice_resolves_a_complete_native_player_look() {
    let data = production_data();
    assert_eq!(data.appearance_document().choices.len(), 231);
    for choice in &data.appearance_document().choices {
        let mut appearance = CharacterAppearance {
            gender: match choice.gender {
                DataGender::Male => UiGender::Boy,
                DataGender::Female => UiGender::Girl,
            },
            ..CharacterAppearance::default()
        };
        let selector = u8::try_from(choice.creation_index)
            .expect("creator choice selector remains protocol-sized");
        match choice.category {
            CharacterAppearanceCategory::Face => appearance.face = selector,
            CharacterAppearanceCategory::Hair => appearance.hair = selector,
            CharacterAppearanceCategory::Shirt => appearance.shirt = selector,
            CharacterAppearanceCategory::Pants => appearance.pants = selector,
            CharacterAppearanceCategory::Shoes => appearance.shoes = selector,
        }
        let resolved = data
            .resolve_creator(0, 0, "Coverage", "Proof", &appearance)
            .unwrap_or_else(|error| {
                panic!(
                    "{:?}/{:?}/{} failed complete native resolution: {error}",
                    choice.gender, choice.category, choice.creation_index
                )
            });
        resolved
            .look
            .validate()
            .expect("resolved creator look remains valid");
        assert_eq!(resolved.look.parts.len(), 5);
        assert!(resolved.look.skin_texture.is_some());
    }
}

#[test]
fn tutorial_weapon_matrix_resolves_exact_rigid_models_for_both_genders() {
    let data = production_data();
    for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
        for item_number in [43, 197, 328] {
            let weapon = data
                .resolve_weapon_attachment(item_number, gender)
                .unwrap_or_else(|error| {
                    panic!("{gender:?} tutorial weapon {item_number} failed: {error}")
                });
            assert_eq!(weapon.kind, NativePlayerPartKind::Weapon);
            assert!(weapon.exact_route.starts_with("wear/"));
            assert!(weapon.exact_route.ends_with(".nif"));
            assert!(weapon.glb.ends_with(".glb"));
            assert!(weapon.primary_texture.is_some());
            assert!(weapon.secondary_texture.is_none());
        }
    }
}
