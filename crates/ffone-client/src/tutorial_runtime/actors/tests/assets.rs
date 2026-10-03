use super::*;

pub(super) fn production_visual_catalog() -> NetworkNpcVisualCatalog0104 {
    let assets = crate::assets::AssetLocator::open(asset_root()).unwrap();
    NetworkNpcVisualCatalog0104::open(&assets).unwrap()
}

#[test]
fn catalog_covers_every_tutorial_type_and_exact_xdt_combat_values() {
    let required = [
        2374, 2375, 2376, 2377, 2378, 2663, 2664, 2665, 2666, 2667, 2668, 2669, 2670, 2671,
        2672, 2673, 2674, 2675, 2676, 2677, 2678, 2694, 2695, 2696, 2800, 2897, 2902, 2903,
        2968,
    ];
    assert_eq!(
        tutorial_actor_npc_types().into_iter().collect::<Vec<_>>(),
        required
    );

    let content = production_content();
    let numbuh_two = content.gameplay_npc(2671).unwrap();
    assert_eq!(
        (
            numbuh_two.team,
            numbuh_two.max_hp,
            numbuh_two.table_scale.unwrap().value()
        ),
        (1, 361, 1.0)
    );
    let spawn = content.gameplay_npc(2674).unwrap();
    assert_eq!((spawn.team, spawn.max_hp, spawn.ai_type), (2, 400, 1));
    assert!((spawn.table_scale.unwrap().value() - 1.5).abs() < f32::EPSILON);
    assert_eq!((spawn.attack_range(), spawn.npc_style), (Some(17.4), 2));
    let cerberus = content.gameplay_npc(2675).unwrap();
    assert_eq!((cerberus.team, cerberus.max_hp), (2, 1000));
    assert_eq!(
        (cerberus.attack_range(), cerberus.npc_style),
        (Some(16.8), 1)
    );
    let oil_ogre = content.gameplay_npc(2676).unwrap();
    assert_eq!((oil_ogre.team, oil_ogre.max_hp), (2, 1300));
    assert_eq!(
        (oil_ogre.attack_range(), oil_ogre.npc_style),
        (Some(16.8), 1)
    );
    let tech_wing = content.gameplay_npc(2677).unwrap();
    assert_eq!((tech_wing.team, tech_wing.max_hp), (2, 400));
    assert_eq!(
        (tech_wing.attack_range(), tech_wing.npc_style),
        (Some(17.0), 2)
    );
    let fusion_buttercup = content.gameplay_npc(2678).unwrap();
    assert_eq!((fusion_buttercup.team, fusion_buttercup.max_hp), (2, 1500));
    assert_eq!(
        (fusion_buttercup.attack_range(), fusion_buttercup.npc_style),
        (Some(17.0), 2)
    );
    let fusion_spawn = content.gameplay_npc(2897).unwrap();
    assert_eq!(
        (
            fusion_spawn.team,
            fusion_spawn.max_hp,
            fusion_spawn.table_scale.unwrap().value(),
        ),
        (1, 361, 3.0)
    );
}

#[test]
fn tutorial_mob_textures_follow_the_shared_xdt_viewer_catalog() {
    let catalog = production_visual_catalog();
    let expected = [
        (2665, None, None),
        (2673, Some("npc_dexter"), Some("npc_dexter_glass")),
        (2674, Some("mob_sneakyspawn"), Some("spawn11_green")),
        (2675, Some("mob_cerberus"), Some("spawn11_green")),
        (2676, Some("mob_oilmonster"), Some("fusionlight")),
        (2677, Some("mob_bat"), Some("fusionlight")),
        (2678, Some("fuison_buttercup"), Some("spawn11_green")),
        (2897, Some("mob_sneakyspawn"), Some("spawn11_green")),
        (2902, None, None),
    ];

    for (npc_type, expected_main, expected_sub) in expected {
        let table_visual = catalog
            .get(npc_type)
            .unwrap_or_else(|| panic!("missing shared visual for NPC type {npc_type}"));
        assert_eq!(
            table_visual
                .main_texture
                .as_ref()
                .map(|texture| texture.true_name.as_str()),
            expected_main,
            "wrong main texture for tutorial NPC type {npc_type}"
        );
        assert_eq!(
            table_visual
                .sub_texture
                .as_ref()
                .map(|texture| texture.true_name.as_str()),
            expected_sub,
            "wrong sub texture for tutorial NPC type {npc_type}"
        );
        for texture in [
            table_visual.main_texture.as_ref(),
            table_visual.sub_texture.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            assert!(
                asset_root().join(&texture.path).is_file(),
                "missing XDT texture {} for tutorial NPC type {npc_type}",
                texture.path
            );
        }
    }
}

#[test]
fn proven_tutorial_visuals_resolve_through_the_shared_catalog() {
    let catalog = production_visual_catalog();
    let expected = [
        (2670, "characters/npcs/npc_ben/npc_ben.glb"),
        (2673, "characters/npcs/npc_dexter/npc_dexter.glb"),
        (2674, "characters/mobs/mob_sneakyspawn/mob_sneakyspawn.glb"),
        (2675, "characters/mobs/mob_cerberus/mob_cerberus.glb"),
        (2676, "characters/mobs/mob_oilmonster/mob_oilmonster.glb"),
        (
            2678,
            "characters/fusions/fusion_buttercup/fusion_buttercup.glb",
        ),
        (2902, "characters/npcs/t_dextersword/t_dextersword.glb"),
        (2695, "characters/npcs/npc_fusiongate/npc_fusiongate.glb"),
        (2897, "characters/mobs/mob_sneakyspawn/mob_sneakyspawn.glb"),
    ];
    for (npc_type, expected_glb) in expected {
        let definition = catalog.get(npc_type).unwrap();
        assert_eq!(definition.glb, expected_glb, "NPC type {npc_type}");
        assert!(asset_root().join(&definition.glb).is_file());
    }
}

pub(super) fn asset_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game")
}
