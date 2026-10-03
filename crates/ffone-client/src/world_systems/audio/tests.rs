use std::path::Path;

use crate::world_audio::*;

#[test]
fn completed_warp_replaces_loading_theme_with_infected_zone_and_lair_audio() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let audio = NativeAudioCatalog::open(&root, false).unwrap();
    let zones = RetrobutionWorldAudioCatalog::open(&locator, &audio).unwrap();
    let center = |zone: &WorldAudioZone| {
        let point = zone
            .points
            .iter()
            .fold([0.0, 0.0], |a, b| [a[0] + b[0], a[1] + b[1]])
            .map(|v| v / zone.points.len() as f64);
        GlobalTransform::from_translation(Vec3::new(-point[0] as f32, 0.0, point[1] as f32))
    };
    let infected = center(&zones.music_zones[4]);
    let lair = center(&zones.ambient_zones[104]);
    let mut app = App::new();
    app.insert_resource(audio)
        .insert_resource(zones)
        .init_resource::<Time>()
        .init_resource::<RetrobutionInstanceAudioState>()
        .insert_resource(RetrobutionLoadingAudioState { active: true })
        .init_resource::<RetrobutionMusicRequests>()
        .init_resource::<WorldAudioRuntime>()
        .add_systems(Update, select_retrobution_world_audio);
    app.world_mut().spawn(NativeWorldSceneRoot {
        name: "music fixture".into(),
        tile: [0, 0],
        scope: NativeWorldScope::WorldMap,
        selection_scope: NativeWorldScope::WorldMap,
    });
    let player = app
        .world_mut()
        .spawn((LegacyPlayerController::from_baseline_table(), infected))
        .id();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(1));
    app.update();
    let key =
        |channel: &ChannelRuntime| channel.target.as_ref().and_then(|t| t.logical_key.clone());
    assert_eq!(
        key(&app.world().resource::<WorldAudioRuntime>().music).as_deref(),
        Some("music/00_main_theme")
    );
    // Reproduce respawning into the same zone during its repeat cooldown.
    // Previously this left the loading theme selected until that expired.
    {
        let mut runtime = app.world_mut().resource_mut::<WorldAudioRuntime>();
        runtime.music.last_started[4] = runtime.elapsed;
    }
    app.world_mut()
        .resource_mut::<RetrobutionLoadingAudioState>()
        .active = false;
    app.world_mut()
        .resource_mut::<RetrobutionInstanceAudioState>()
        .active = true;
    app.update();
    assert_eq!(
        key(&app.world().resource::<WorldAudioRuntime>().music).as_deref(),
        Some("music/02_pokey_oaks_junior_high")
    );
    app.world_mut().entity_mut(player).insert(lair);
    app.update();
    let runtime = app.world().resource::<WorldAudioRuntime>();
    assert_eq!(key(&runtime.music), None);
    assert_eq!(
        key(&runtime.ambient).as_deref(),
        Some("ambient/ambient_fusion_lairs")
    );
}

#[test]
fn loading_and_instance_music_select_their_own_production_zones() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let audio = NativeAudioCatalog::open(&root, false).unwrap();
    let catalog = RetrobutionWorldAudioCatalog::open(&locator, &audio).unwrap();
    let times = vec![f64::NEG_INFINITY; catalog.music_zones.len()];
    let lobby = eligible_zone(&catalog.music_zones, &times, [1.0, 1.0], false, 0.0)
        .flatten()
        .unwrap();
    assert_eq!(lobby.logical_key.as_deref(), Some("music/00_main_theme"));
    let zone = catalog
        .music_zones
        .iter()
        .find(|zone| zone.logical_key.as_deref() == Some("music/buttercuplair1"))
        .unwrap();
    let point = zone
        .points
        .iter()
        .fold([0.0, 0.0], |sum, p| [sum[0] + p[0], sum[1] + p[1]])
        .map(|v| v / zone.points.len() as f64);
    let inside = eligible_zone(&catalog.music_zones, &times, point, true, 0.0)
        .flatten()
        .unwrap();
    assert_eq!(inside.logical_key.as_deref(), Some("music/buttercuplair1"));
    let outside = eligible_zone(&catalog.music_zones, &times, point, false, 0.0).flatten();
    assert_ne!(outside, Some(inside));
    assert_eq!(
        resolve_music_request("buttercuplair.ogg", &catalog, &audio),
        Some("music/buttercuplair1".into())
    );
    assert_eq!(
        resolve_music_request("../../unknown.ogg", &catalog, &audio),
        None
    );
    assert_eq!(
        resolve_music_request("vsweeper.ogg", &catalog, &audio),
        Some("music/vs_weeper".into())
    );
}

#[test]
fn loading_music_works_without_a_player_and_one_shot_override_is_acknowledged() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let audio = NativeAudioCatalog::open(&root, false).unwrap();
    let zones = RetrobutionWorldAudioCatalog::open(&locator, &audio).unwrap();
    let mut app = App::new();
    app.insert_resource(audio)
        .insert_resource(zones)
        .init_resource::<Time>()
        .init_resource::<RetrobutionInstanceAudioState>()
        .insert_resource(RetrobutionLoadingAudioState { active: true })
        .init_resource::<RetrobutionMusicRequests>()
        .init_resource::<WorldAudioRuntime>()
        .add_systems(Update, select_retrobution_world_audio);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(1));
    app.update();
    assert_eq!(
        app.world()
            .resource::<WorldAudioRuntime>()
            .music
            .target
            .as_ref()
            .and_then(|target| target.logical_key.as_deref()),
        Some("music/00_main_theme")
    );
    app.world_mut()
        .resource_mut::<RetrobutionMusicRequests>()
        .request("MissionTheme1.ogg".into(), false);
    app.update();
    let entity = app.world_mut().spawn_empty().id();
    {
        let mut runtime = app.world_mut().resource_mut::<WorldAudioRuntime>();
        runtime.music.current = runtime.music.target.take();
        runtime.music.current_entity = Some(entity);
        runtime.music.phase = ChannelPhase::Stable;
    }
    app.update();
    assert!(
        app.world()
            .resource::<WorldAudioRuntime>()
            .override_track
            .is_none()
    );
    {
        let mut runtime = app.world_mut().resource_mut::<WorldAudioRuntime>();
        runtime.music.current = None;
        runtime.music.current_entity = None;
        runtime.music.phase = ChannelPhase::Stopped;
    }
    app.update();
    let runtime = app.world().resource::<WorldAudioRuntime>();
    assert!(runtime.override_track.is_none());
    assert_eq!(
        runtime
            .music
            .target
            .as_ref()
            .and_then(|target| target.logical_key.as_deref()),
        Some("music/00_main_theme")
    );
}

#[test]
fn production_contract_resolves_renamed_audio_by_logical_key() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let audio = NativeAudioCatalog::open(&root, false).unwrap();
    let catalog = RetrobutionWorldAudioCatalog::open(&locator, &audio).unwrap();
    assert_eq!(catalog.counts(), (136, 136, 66, 1_324));

    let buttercup = catalog
        .music_zones
        .iter()
        .find(|zone| zone.source_file_name == "buttercuplair.ogg")
        .unwrap();
    assert_eq!(
        buttercup.source_true_name.as_deref(),
        Some("ButtercupLair1")
    );
    assert_eq!(
        buttercup.logical_key.as_deref(),
        Some("music/buttercuplair1")
    );
    let galaxy = catalog
        .music_zones
        .iter()
        .find(|zone| zone.source_file_name == "galaxxy gardens.ogg")
        .unwrap();
    assert_eq!(galaxy.source_true_name.as_deref(), Some("Galaxy Gardens"));
    assert_eq!(galaxy.logical_key.as_deref(), Some("music/galaxy_gardens"));

    assert_eq!(
        catalog
            .music_zones
            .iter()
            .filter(|zone| zone.resolution == WorldAudioResolution::LegacySourceMissing)
            .count(),
        4,
        "clean Retrobution has four real missing BGM loads; do not substitute them"
    );
    assert!(catalog.environment_tiles.values().any(|tile| {
        tile.emitters.iter().any(|emitter| {
            emitter.source_true_name == "HOLOGRAMS 1"
                && emitter.logical_key == "sfx/environment/holograms_1"
        })
    }));
}

#[test]
fn clean_polygon_order_and_delay_are_preserved() {
    let zone = |index, delay| WorldAudioZone {
        index,
        _name: String::new(),
        instance: false,
        delay_seconds: delay,
        source_file_name: "test.ogg".to_owned(),
        source_true_name: Some("test".to_owned()),
        logical_key: Some(format!("music/{index}")),
        resolution: WorldAudioResolution::Resolved,
        points: vec![[0.0, 0.0], [0.0, 10.0], [10.0, 10.0], [10.0, 0.0]],
    };
    let zones = vec![zone(0, 45.0), zone(1, 0.0)];
    assert!(polygon_contains(&zones[0].points, [5.0, 5.0]));
    assert_eq!(
        eligible_zone(
            &zones,
            &[100.0, f64::NEG_INFINITY],
            [5.0, 5.0],
            false,
            120.0
        ),
        Some(None),
        "the later overlapping row must not bypass the first row's delay"
    );
    assert_eq!(
        eligible_zone(
            &zones,
            &[100.0, f64::NEG_INFINITY],
            [5.0, 5.0],
            false,
            146.0
        )
        .unwrap()
        .unwrap()
        .index,
        0
    );
}

#[test]
fn crossing_into_another_zone_with_the_same_clip_keeps_the_running_source() {
    let target = |index: usize, key: &str| ZoneTarget {
        index,
        logical_key: Some(key.to_owned()),
    };
    let mut channel = ChannelRuntime::with_zone_count(3);
    channel.request(target(0, "music/11_darklands"));
    channel.current = channel.target.take();
    channel.current_entity = Some(Entity::from_raw_u32(1).unwrap());
    channel.phase = ChannelPhase::Stable;

    // Twelve authored polygons name `music/11_darklands`; walking between
    // them must not fade out and replay the track.
    channel.request(target(1, "music/11_darklands"));
    assert_eq!(channel.phase, ChannelPhase::Stable);
    assert!(channel.target.is_none());
    assert_eq!(
        channel.current.as_ref().map(|current| current.index),
        Some(1)
    );

    // A different clip still transitions.
    channel.request(target(2, "music/08_woods"));
    assert_eq!(channel.phase, ChannelPhase::FadingOut);
    assert_eq!(channel.target, Some(target(2, "music/08_woods")));
}

#[test]
fn leaving_infected_zone_restores_downtown_during_its_replay_delay() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let audio = NativeAudioCatalog::open(&root, false).unwrap();
    let catalog = RetrobutionWorldAudioCatalog::open(&locator, &audio).unwrap();
    let mut channel = ChannelRuntime::with_zone_count(catalog.music_zones.len());
    channel.current = Some(ZoneTarget {
        index: 4,
        logical_key: Some("music/02_pokey_oaks_junior_high".into()),
    });
    channel.current_entity = Some(Entity::from_raw_u32(1).unwrap());
    channel.phase = ChannelPhase::Stable;
    channel.selected_zone = Some(4);
    for index in 60..=63 {
        assert_eq!(catalog.music_zones[index].logical_key.as_deref(), Some("music/05_midtown"));
        let zone = &catalog.music_zones[index];
        let point = zone.points.iter().fold([0.0, 0.0], |a, p| [a[0] + p[0], a[1] + p[1]])
            .map(|v| v / zone.points.len() as f64);
        channel.last_started[index] = 100.0;
        select_channel_zone(&mut channel, &catalog.music_zones, point, false, 101.0);
        assert_eq!(channel.target.as_ref().unwrap().logical_key.as_deref(), Some("music/05_midtown"));
        assert_eq!(channel.phase, ChannelPhase::FadingOut);
        channel.target = None;
        select_channel_zone(&mut channel, &catalog.music_zones, point, false, 102.0);
        assert!(channel.target.is_none(), "staying in the same zone preserves its replay delay");
    }
    select_channel_zone(&mut channel, &catalog.music_zones, [-1.0, -1.0], false, 103.0);
    assert!(channel.target.as_ref().unwrap().logical_key.is_none(), "leaving all zones stops stale music");
}
