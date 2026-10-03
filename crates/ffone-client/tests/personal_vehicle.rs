//! Production content checks for the mounted player contract.
use ffone_client::{
    assets::AssetLocator, character_creation_data::CharacterCreationData,
    player_preview::NativePlayerPartKind, player_shared_rig::NativePlayerRigCatalog,
    tutorial_mission_content::TutorialMissionContent,
    tutorial_player_presentation::TutorialPlayerClip,
};
use ffone_runtime_contracts::PlayerRigGender;

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game")
}

#[test]
fn every_published_vehicle_resolves_a_native_model_texture_speed_and_family() {
    let root = root();
    let locator = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let data = CharacterCreationData::open(&root).unwrap();
    let audio = ffone_client::semantic_audio::NativeAudioCatalog::open(root.clone(), false).unwrap();
    let tables: serde_json::Value = locator.read_table_set().unwrap();
    let items = tables["tables"].as_array().unwrap().iter()
        .find_map(|table| table["value"]["m_pVehicleItemTable"]["m_pItemData"].as_array()).unwrap();
    let mut count = 0;
    let vehicles: serde_json::Value = serde_json::from_slice(&locator.read("characters/player/shared/vehicles.json").unwrap()).unwrap();
    let published: std::collections::BTreeSet<_> = vehicles["vehicles"].as_array().unwrap().iter().map(|v| v["itemId"].as_u64().unwrap() as i16).collect();
    for id in 1..=i16::MAX {
        // Two reserved table rows have no model or texture in the source.
        // They are not renderable items and must not receive a substitute.
        if items.get(id as usize).is_none_or(|row| row["m_iMesh"].as_i64() == Some(0)) { continue; }
        if id <= 136 { assert!(published.contains(&id), "primary vehicle {id} is missing"); }
        if !published.contains(&id) { continue; }
        let Some(family @ 1..=3) = content.gameplay_vehicle_equip_type(id) else {
            continue;
        };
        let part = data
            .resolve_vehicle_attachment(id as u32)
            .unwrap_or_else(|error| panic!("vehicle {id}, family {family}: {error}"));
        assert_eq!(part.kind, NativePlayerPartKind::Vehicle);
        assert!(root.join(&part.glb).is_file(), "vehicle {id}: {}", part.glb);
        assert!(
            content.gameplay_vehicle_speed(id).unwrap() > 0,
            "vehicle {id}"
        );
        let texture = part.primary_texture.as_ref().expect("vehicle texture");
        assert!(
            root.join(&texture.path).is_file(),
            "vehicle {id}: {texture:?}"
        );
        assert_eq!(part.secondary_texture.as_ref(), Some(texture));
        if let Some(engine) = content.gameplay_vehicle_engine_sound(id) {
            let matches = audio.by_true_name(engine);
            assert_eq!(matches.len(), 1, "vehicle {id} engine {engine}");
            assert!(root.join(&matches[0].path).is_file());
        }
        count += 1;
    }
    assert!(
        count == published.len(),
        "expected every published vehicle, got {count}"
    );
    assert!(count >= 137);
    assert_eq!(
        content.gameplay_vehicle_engine_sound(1),
        Some("Vehicle_Hover4")
    );
    assert_eq!(
        content.gameplay_vehicle_engine_sound(4),
        Some("Vehicle_Scooter4")
    );
}

#[test]
fn both_genders_have_all_vehicle_clips_and_scooter_turns_in_the_actual_glb() {
    let root = root();
    let catalog = NativePlayerRigCatalog::open(&root).unwrap();
    for (gender, path) in [
        (
            PlayerRigGender::Male,
            "characters/player/male/base/male_skeleton.glb",
        ),
        (
            PlayerRigGender::Female,
            "characters/player/female/base/female_skeleton.glb",
        ),
    ] {
        let model = gltf::Gltf::open(root.join(path)).unwrap();
        let animations: Vec<_> = model.animations().collect();
        let contract = catalog.gender(gender).unwrap();
        let mut count = 0;
        for clip in TutorialPlayerClip::ALL
            .into_iter()
            .filter(|c| c.is_vehicle())
        {
            let entry = contract
                .clips
                .iter()
                .find(|e| e.name == clip.name())
                .unwrap();
            let animation = &animations[entry.gltf_animation_index as usize];
            assert_eq!(animation.name(), Some(clip.name()));
            assert_eq!(animation.channels().count(), entry.channel_count as usize);
            assert_eq!(entry.playback, clip.playback().contract_value());
            assert_eq!(
                entry.source_path_id, 0,
                "native additions have semantic identities"
            );
            count += 1;
        }
        assert_eq!(count, 14);
        for name in ["scooter_turnleft", "scooter_turnright"] {
            assert_eq!(
                animations.iter().filter(|a| a.name() == Some(name)).count(),
                1
            );
            let animation = animations.iter().find(|a| a.name() == Some(name)).unwrap();
            for channel in animation.channels() {
                let sample = usize::from(channel.sampler().interpolation() == gltf::animation::Interpolation::CubicSpline);
                let reader = channel.reader(|_| model.blob.as_deref());
                use gltf::animation::util::ReadOutputs;
                match reader.read_outputs().unwrap() {
                    ReadOutputs::Translations(mut values) | ReadOutputs::Scales(mut values) => {
                        assert!(values.nth(sample).unwrap().iter().all(|v| v.abs() < 0.00001), "{gender:?} {name}: additive origin");
                    }
                    ReadOutputs::Rotations(values) => {
                        let q = values.into_f32().nth(sample).unwrap();
                        assert!(q[..3].iter().all(|v| v.abs() < 0.00001) && (q[3].abs() - 1.0).abs() < 0.00001, "{gender:?} {name}: additive rotation origin");
                    }
                    _ => panic!("unexpected morph channel in vehicle turn"),
                }
            }
        }
    }
}
