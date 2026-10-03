use crate::hnpc_runtime::*;

#[test]
fn hnpc_palette_preserves_zero_based_extended_colors() {
    let palette: HnpcPalette =
        serde_json::from_str(include_str!("../../../../../assets/game/data/hnpc/palette.json"))
            .unwrap();
    assert_eq!(palette.skin.len(), 20);
    assert_eq!(palette.hair.len(), 20);
    for colors in [&palette.skin, &palette.hair] {
        for rgba in colors {
            assert!(
                rgba.iter()
                    .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
            );
        }
    }
    assert_eq!(
        palette_color(&palette.skin, 0),
        LinearRgba::new(0.2265625, 0.15234375, 0.09765625, 1.0)
    );
    assert_eq!(
        palette_color(&palette.skin, 3),
        LinearRgba::new(0.59375, 0.40625, 0.328125, 1.0)
    );
    assert_eq!(
        palette_color(&palette.skin, 15),
        LinearRgba::new(0.39453125, 0.30859375, 0.23046875, 1.0)
    );
    assert_eq!(
        palette_color(&palette.hair, 19),
        LinearRgba::new(0.09765625, 0.48828125, 0.171875, 1.0)
    );
    // Preserve the existing fallback for unset and out-of-range authored selectors.
    assert_eq!(palette_color(&palette.hair, -1), LinearRgba::WHITE);
    assert_eq!(palette_color(&palette.skin, 20), LinearRgba::WHITE);
}

#[test]
fn production_catalog_closes_all_published_hnpc_appearances() {
    let Some(root) = option_env!("CARGO_MANIFEST_DIR") else {
        return;
    };
    let repo = Path::new(root).join("../..");
    let locator = AssetLocator::open(repo.join("assets/game")).unwrap();
    let rig = NativePlayerRigCatalog::open(repo.join("assets/game")).unwrap();
    let catalog = HnpcRuntimeCatalog::open(&locator, &rig).unwrap();
    for gender in [PlayerRigGender::Male, PlayerRigGender::Female] {
        let ends = catalog.animation_ends(gender);
        let sounds = catalog.animation_sounds(gender);
        let prefix = if gender == PlayerRigGender::Male {
            "M"
        } else {
            "F"
        };
        assert!(sounds.iter().any(|event| event.clip == "applaud"
            && event.time == 0.25
            && event.payload == format!("{prefix}_Avatar_Applause.wav")));
        assert!(
            sounds
                .iter()
                .any(|event| event.clip == "dance2" && event.payload.contains("_SFX_Dance"))
        );
        assert!(
            !sounds.iter().any(|event| event.clip == "talk"),
            "silent source gestures must stay silent"
        );
        assert!((ends["rifleguard"] - 5.083_333_5).abs() < 0.000_01);
        for clip in [
            "talk",
            "talkexclamation",
            "talkquestion",
            "swordguard",
            "scratch",
        ] {
            assert!(
                ends[clip] > 0.0,
                "{gender:?} {clip} needs its authored end event"
            );
        }
    }
    assert!(catalog.len() >= 205);
    for index in 185..205 {
        assert!(
            catalog.appearance(index).unwrap().look.is_some(),
            "appearance {index}"
        );
    }
    assert!(catalog.appearance(0).unwrap().look.is_none());
    assert_eq!(catalog.appearance(148).unwrap().legacy_type, 143);
    assert!(catalog.appearance(148).unwrap().look.is_some());
    assert_eq!(catalog.appearance(149).unwrap().legacy_type, 143);
    assert!(catalog.appearance(149).unwrap().look.is_some());
    for index in [144, 145, 176, 177, 178] {
        let look = catalog.appearance(index).unwrap().look.as_ref().unwrap();
        for (kind, name) in [
            (NativePlayerPartKind::Shirt, "f_shirt_secretagent"),
            (NativePlayerPartKind::Pants, "f_pants_secretagent"),
            (NativePlayerPartKind::Shoes, "f_shoes_secretagent"),
        ] {
            // The accountant keeps her skirt and shoes;
            // only her jacket uses the shared banker model.
            if index == 178 && kind != NativePlayerPartKind::Shirt {
                continue;
            }
            let part = look.parts.iter().find(|part| part.kind == kind).unwrap();
            assert_eq!(part.exact_route, format!("wear/{name}.nif"));
            assert_eq!(
                part.glb,
                format!("characters/player/hnpc/banker/{name}.glb")
            );
            if [144, 145, 178].contains(&index) && kind == NativePlayerPartKind::Shirt {
                assert_eq!(
                    part.primary_texture.as_ref().unwrap().path,
                    "characters/hnpc/textures/female_banker_jacket.png"
                );
            }
            if [144, 145].contains(&index) && kind == NativePlayerPartKind::Pants {
                assert_eq!(
                    part.primary_texture.as_ref().unwrap().path,
                    "characters/hnpc/textures/female_banker_trousers.png"
                );
            }
            if kind == NativePlayerPartKind::Shoes {
                assert_eq!(
                    part.primary_texture.as_ref().unwrap().path,
                    "characters/hnpc/textures/shoes_agentsix.png"
                );
            }
        }
    }
    let accountant = catalog.appearance(178).unwrap().look.as_ref().unwrap();
    let accountant_glasses = accountant
        .parts
        .iter()
        .find(|part| part.kind == NativePlayerPartKind::Glasses)
        .unwrap();
    assert_eq!(accountant_glasses.exact_route, "wear/glasses_3.nif");
    assert_eq!(
        accountant_glasses.glb,
        "characters/player/items/glasses/glasses_glasses1/models/glasses_glasses3/accountant.glb"
    );
    let other_glasses = catalog.appearance(183).unwrap().look.as_ref().unwrap();
    assert!(other_glasses.parts.iter().any(|part| {
        part.kind == NativePlayerPartKind::Glasses
            && part.glb
                == "characters/player/items/glasses/glasses_glasses1/models/glasses_glasses3/model.glb"
    }));
    for (kind, route) in [
        (NativePlayerPartKind::Pants, "wear/f_pants_sailormoon.nif"),
        (NativePlayerPartKind::Shoes, "wear/f_shoes_shyabangset.nif"),
    ] {
        let part = accountant.parts.iter().find(|part| part.kind == kind).unwrap();
        assert_eq!(part.exact_route, route);
        assert!(part.primary_texture.as_ref().unwrap().path.ends_with("/shoes_accountant.png"));
    }
    for (appearance, expected) in [
        (107, PlayerWeaponAnimationProfile::Stick),
        (172, PlayerWeaponAnimationProfile::Pistol),
        (115, PlayerWeaponAnimationProfile::Rifle),
        (97, PlayerWeaponAnimationProfile::Rocket),
    ] {
        assert_eq!(
            catalog
                .appearance(appearance)
                .and_then(|appearance| appearance.look.as_ref())
                .and_then(|look| look.weapon_animation_profile),
            Some(expected)
        );
    }
}
