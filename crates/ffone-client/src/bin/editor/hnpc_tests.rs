use super::*;
use serde_json::json;

#[test]
fn private_appearance_appends_without_reindexing_or_overwriting_external_edits() {
    let base = json!({"schema":"native","appearances":[{"index":0,"parts":[],"metadata":{"keep":true}}],"textures":[{"keep":1}]});
    let mut disk = base.clone();
    disk["appearances"]
        .as_array_mut()
        .unwrap()
        .push(json!({"index":1,"external":true}));
    let mut draft = base["appearances"][0].clone();
    draft["height"] = json!(4);
    let (index, next) = publish_document(&base, &draft, &disk, 0, false).unwrap();
    assert_eq!(index, 2);
    assert_eq!(next["appearances"][0], base["appearances"][0]);
    assert_eq!(next["appearances"][1], disk["appearances"][1]);
    assert_eq!(next["textures"], base["textures"]);
    assert_eq!(next["appearances"][2]["metadata"]["keep"], true);
    assert_eq!(next["appearances"][2]["index"], 2);
}
#[test]
fn shared_appearance_conflicts_only_with_its_own_changed_record() {
    let base = json!({"appearances":[{"index":0,"height":1},{"index":1,"height":2}]});
    let draft = json!({"index":0,"height":3});
    let mut disk = base.clone();
    disk["appearances"][1]["height"] = json!(4);
    let (_, next) = publish_document(&base, &draft, &disk, 0, true).unwrap();
    assert_eq!(next["appearances"][1]["height"], 4);
    assert_eq!(next["appearances"][0]["height"], 3);
    disk["appearances"][0]["height"] = json!(4);
    assert!(publish_document(&base, &draft, &disk, 0, true).is_err());
}
#[test]
fn published_variants_produce_valid_isolated_runtime_preview_and_native_document() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let catalog = EditorCatalog::open(&locator).unwrap();
    let mut editor = HnpcEditor::default();
    editor
        .open(
            &root.join("data/tables/xdt.json"),
            &json!({"m_iHNpc":1,"m_iHNpcNum":1,"m_iHeight":200}),
            &catalog,
        )
        .unwrap();
    let before = editor.base.clone();
    let original = editor.original.as_ref().unwrap().clone();
    let option = *editor
        .choices("shirt")
        .iter()
        .find(|i| {
            editor.draft["parts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|p| *p != editor.variants[**i].value)
        })
        .unwrap();
    editor.choose(option).unwrap();
    let look = editor.look().unwrap();
    let preview = original
        .with_appearance_override(
            1,
            HnpcRuntimeAppearance {
                legacy_type: 1,
                look: Some(look.clone()),
            },
        )
        .unwrap();
    assert_eq!(
        original
            .appearance(1)
            .unwrap()
            .look
            .as_ref()
            .unwrap()
            .parts
            .len(),
        preview
            .appearance(1)
            .unwrap()
            .look
            .as_ref()
            .unwrap()
            .parts
            .len()
    );
    assert_ne!(
        original
            .appearance(1)
            .unwrap()
            .look
            .as_ref()
            .unwrap()
            .parts
            .iter()
            .find(|p| p.kind == NativePlayerPartKind::Shirt)
            .unwrap(),
        look.parts
            .iter()
            .find(|p| p.kind == NativePlayerPartKind::Shirt)
            .unwrap()
    );
    assert!(
        original
            .with_appearance_override(
                1,
                HnpcRuntimeAppearance {
                    legacy_type: 0,
                    look: Some(look)
                }
            )
            .is_err()
    );
    let (index, next) =
        publish_document(&editor.base, &editor.draft, &editor.base, 1, false).unwrap();
    let loaded = HnpcRuntimeCatalog::from_json(&locator, original.rig_catalog(), next).unwrap();
    assert!(loaded.appearance(index).unwrap().look.is_some());
    assert_eq!(editor.base, before);
    editor.open(&root.join("data/tables/xdt.json"),&json!({"m_iHNpc":1,"m_iHNpcNum":148,"m_iNpcNumber":159,"m_iHeight":200}),&catalog).unwrap();
    assert_eq!(editor.draft["hairColor"],-1);
    assert_eq!(editor.look().unwrap().hair_color,LinearRgba::WHITE);
    let draft=editor.draft.clone();
    let hair=editor.choices("hair")[0];editor.variants[hair].caption="Причёска 🔥".into();editor.filter="ПРИЧЁСКА".into();
    assert!(editor.choices("hair").contains(&hair));assert_eq!(editor.draft,draft);
    editor.filter.clear();
    for kind in ["shirt","pants","shoes","hat","glasses","back","rightWeapon"] {
        let choices=editor.choices(kind);
        let option=*choices.iter().find(|i|editor.variants[**i].caption.contains("· ID ")&&(kind!="hat"||editor.variants[**i].value["equipType"]==0)).expect("player inventory wardrobe variant");
        editor.choose(option).unwrap();
    }
    assert!(editor.choices("shirt").len()>50);
    let (index,next)=publish_document(&editor.base,&editor.draft,&editor.base,148,false).unwrap();
    let loaded=HnpcRuntimeCatalog::from_json(&locator,original.rig_catalog(),next).unwrap();
    assert_eq!(loaded.appearance(index).unwrap().look.as_ref().unwrap().parts.iter().filter(|p|matches!(p.kind,NativePlayerPartKind::Shirt|NativePlayerPartKind::Pants|NativePlayerPartKind::Shoes|NativePlayerPartKind::Hat|NativePlayerPartKind::Glasses|NativePlayerPartKind::Back|NativePlayerPartKind::Weapon)).count(),7);
    let face=editor.choices("face")[0];editor.choose(face).unwrap();editor.choose(hair).unwrap();
    for equip_type in 0..=4 {
        let hat=*editor.choices("hat").iter().find(|i|editor.variants[**i].value["equipType"]==equip_type).expect("player hat policy variant");
        editor.choose(hat).unwrap();let look=editor.look().unwrap();
        let policy=ffone_client::character_creation_data::LegacyHatPolicy::from_equip_type(equip_type).unwrap();
        assert_eq!(look.parts.iter().any(|p|p.kind==NativePlayerPartKind::Hair),policy.hair_variant.is_some());
        assert_eq!(look.parts.iter().any(|p|p.kind==NativePlayerPartKind::Glasses),policy.glasses_visible);
        assert!(look.parts.iter().find(|p|p.kind==NativePlayerPartKind::Face).unwrap().exact_route.contains(&format!("_type{:02}",policy.face_variant)));
        let (index,next)=publish_document(&editor.base,&editor.draft,&editor.base,148,false).unwrap();
        let reopened=HnpcRuntimeCatalog::from_json(&locator,original.rig_catalog(),next).unwrap();
        let reopened_parts=&reopened.appearance(index).unwrap().look.as_ref().unwrap().parts;
        assert_eq!(reopened_parts.len(),look.parts.len());
        for (actual,expected) in reopened_parts.iter().zip(&look.parts) {
            // Accepted NPC texture aliases can have different paths from the
            // inventory copy; compare verified image content and model parts.
            let summary=|p:&NativePlayerPartLook|format!("{:?} {:?} {} {} {:?} {:?}",p.kind,p.assembly,p.exact_route,p.glb,p.primary_texture.as_ref().map(|t|&t.contract.native_png_sha256),p.secondary_texture.as_ref().map(|t|&t.contract.native_png_sha256));
            assert_eq!(summary(actual),summary(expected),"Hat policy {equip_type} changes the saved visual");
        }
    }
    editor.draft["parts"].as_array_mut().unwrap().retain(|p|p["kind"]!="hat");
    let restored=editor.look().unwrap();
    assert!(restored.parts.iter().any(|p|p.kind==NativePlayerPartKind::Hair));
    assert!(restored.parts.iter().any(|p|p.kind==NativePlayerPartKind::Glasses));
}
