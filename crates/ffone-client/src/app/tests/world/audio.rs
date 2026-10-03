use super::*;

#[test]
fn npc_attack_pc_result_accepts_openfusion_zero_entity_type_for_death_audio() {
    let local = AttackResult0104 {
        entity_type: 0,
        id: 77,
        protected: 0,
        damage: 100,
        hp: 0,
        hit_flag: 2,
    };
    let remote = AttackResult0104 { id: 88, ..local };
    let results = [remote, local];

    let selected = local_player_npc_attack_result(&results, 77)
        .expect("typed NPC_ATTACK_PCS must accept OpenFusion's zero eCT");
    assert_eq!(selected.hp, 0);
    assert_eq!(selected.hit_flag & 2, 2);
    assert!(local_player_npc_attack_result(&results, 99).is_none());
}

#[test]
fn player_damage_audio_uses_one_priority_death_edge_for_lethal_mob_damage() {
    assert_eq!(
        local_player_damage_audio_edge(Some(600), 450, false),
        Some(LocalPlayerDamageAudioEdge::Damage { critical: false })
    );
    assert_eq!(
        local_player_damage_audio_edge(Some(150), 0, true),
        Some(LocalPlayerDamageAudioEdge::Death { critical: true })
    );
    assert_eq!(local_player_damage_audio_edge(Some(0), -25, true), None);
    assert_eq!(local_player_damage_audio_edge(Some(150), 150, false), None);
    assert_eq!(local_player_damage_audio_edge(None, 0, false), None);
}

#[test]
fn wound_retriggers_only_for_nonlethal_hp_loss_of_the_same_player() {
    use crate::app::world_combat::local_player_wound_edge;
    assert!(!local_player_wound_edge(None, Some((7, 100))));
    assert!(!local_player_wound_edge(Some((7, 100)), Some((8, 50))));
    assert!(!local_player_wound_edge(Some((7, 100)), Some((7, 100))));
    assert!(!local_player_wound_edge(Some((7, 50)), Some((7, 100))));
    assert!(!local_player_wound_edge(Some((7, 50)), Some((7, 0))));
    assert!(local_player_wound_edge(Some((7, 100)), Some((7, 75))));
    assert!(local_player_wound_edge(Some((7, 75)), Some((7, 50))));
}

#[test]
fn transportation_move_ok_voice_is_catalog_proven_and_mvehicle_is_silent() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&asset_root, false).unwrap();
    assert_eq!(transportation_move_ok_voice_set(&catalog, ""), None);
    assert_eq!(transportation_move_ok_voice_set(&catalog, "mvehicle"), None);
    let true_names = transportation_move_ok_voice_set(&catalog, "MonkeyTransport1").unwrap();
    assert_eq!(
        true_names,
        [
            "MonkeyTransport1_ClickMove01".to_owned(),
            "MonkeyTransport1_ClickMove02".to_owned(),
            "MonkeyTransport1_ClickMove03".to_owned(),
        ]
    );
    for locale in ["en", "ru"] {
        for true_name in &true_names {
            let matches = catalog.by_true_name(true_name);
            let [asset] = matches.as_slice() else {
                panic!("{true_name} must resolve exactly once");
            };
            assert_eq!(asset.category, NativeAudioCategory::Voice);
            assert!(
                asset_root
                    .join(catalog.path_for_locale(asset, locale).unwrap())
                    .is_file()
            );
        }
    }
}

#[test]
fn option_sound_mix_multiplies_clean_master_and_semantic_channel_gates() {
    let mut sound = SoundSettings::default();
    assert_eq!(option_sound_gain(&sound, None), 0.5);
    assert_eq!(
        option_sound_gain(&sound, Some(NativeAudioCategory::Sfx)),
        0.25
    );
    sound.effects.enabled = false;
    assert_eq!(
        option_sound_gain(&sound, Some(NativeAudioCategory::Sfx)),
        0.0
    );
    sound.master.enabled = false;
    assert_eq!(
        option_sound_gain(&sound, Some(NativeAudioCategory::Voice)),
        0.0
    );
}

pub(super) fn assert_local_infection_audio(app: &mut App, damaged: bool, gender: i8, locale: &str) {
    let mut query = app
        .world_mut()
        .query::<(Entity, &AudioPlayer, Option<&LocalizedVoice>)>();
    let sounds = query.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(sounds.len(), if damaged { 2 } else { 0 });
    let catalog = app.world().resource::<NativeAudioCatalog>();
    let asset_server = app.world().resource::<AssetServer>();
    let mut poison = 0;
    let mut voice_count = 0;
    let entities = sounds
        .iter()
        .map(|(entity, ..)| *entity)
        .collect::<Vec<_>>();
    for (_, player, voice) in sounds {
        let expected_path = if let Some(voice) = voice {
            let names = if gender == 1 {
                ["M_Avatar_GooDmg01", "M_Avatar_GooDmg02"]
            } else {
                ["F_Avatar_GooDmg01", "F_Avatar_GooDmg02"]
            };
            assert!(names.contains(&voice.true_name.as_str()));
            voice_count += 1;
            catalog
                .path_for_locale(catalog.by_true_name(&voice.true_name)[0], locale)
                .unwrap()
        } else {
            poison += 1;
            catalog.by_true_name("SFX_PoisonDamage")[0].path.as_str()
        };
        assert_eq!(
            asset_server.get_path(player.0.id()).unwrap().path(),
            std::path::Path::new(expected_path)
        );
    }
    assert_eq!((poison, voice_count), if damaged { (1, 1) } else { (0, 0) });
    for entity in entities {
        app.world_mut().despawn(entity);
    }
}
