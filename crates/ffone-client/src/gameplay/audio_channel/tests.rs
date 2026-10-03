use super::*;

#[test]
#[ignore = "requires an audio output device; sources stay paused and silent"]
fn live_regular_and_spatial_sinks_follow_channels_and_master_without_restart() {
    use bevy::audio::{AudioPlugin, Pitch};
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        AudioPlugin::default(),
        GameplayChannelMixPlugin,
    ));
    let source = app
        .world_mut()
        .resource_mut::<Assets<Pitch>>()
        .add(Pitch::new(440.0, std::time::Duration::from_secs(30)));
    let voice = app
        .world_mut()
        .spawn((
            GameplayAudioChannel::new(NativeAudioCategory::Voice, 1.0),
            AudioPlayer(source.clone()),
            PlaybackSettings {
                paused: true,
                ..PlaybackSettings::LOOP
            }
            .with_spatial(true),
            Transform::IDENTITY,
        ))
        .id();
    let sfx = app
        .world_mut()
        .spawn((
            GameplayAudioChannel::ui_sfx(),
            AudioPlayer(source.clone()),
            PlaybackSettings {
                paused: true,
                ..PlaybackSettings::LOOP
            },
        ))
        .id();
    let ambient = app
        .world_mut()
        .spawn((
            GameplayAudioChannel::new(NativeAudioCategory::Ambient, 1.0),
            AudioPlayer(source),
            PlaybackSettings {
                paused: true,
                ..PlaybackSettings::LOOP
            }
            .with_spatial(true),
            Transform::IDENTITY,
        ))
        .id();
    app.world_mut()
        .spawn((SpatialListener::default(), Transform::IDENTITY));
    app.world_mut().resource_mut::<GlobalVolume>().volume = Volume::Linear(0.2);
    app.update();
    app.update();
    let spatial_volume = |app: &App, entity| {
        app.world()
            .get::<SpatialAudioSink>(entity)
            .expect("audio output device required")
            .volume()
            .to_linear()
    };
    let regular_volume = |app: &App| {
        app.world()
            .get::<AudioSink>(sfx)
            .unwrap()
            .volume()
            .to_linear()
    };
    assert!((spatial_volume(&app, voice) - 0.1).abs() < 0.0001);
    assert!((regular_volume(&app) - 0.07).abs() < 0.0001);
    app.world_mut().resource_mut::<RetrobutionAudioMix>().voice = 0.0;
    app.update();
    assert_eq!(spatial_volume(&app, voice), 0.0);
    assert!((regular_volume(&app) - 0.07).abs() < 0.0001);
    assert!((spatial_volume(&app, ambient) - 0.1).abs() < 0.0001);
    app.world_mut().resource_mut::<RetrobutionAudioMix>().voice = 0.8;
    app.world_mut()
        .resource_mut::<RetrobutionAudioMix>()
        .effects = 0.0;
    app.world_mut()
        .resource_mut::<RetrobutionAudioMix>()
        .ambient = 0.0;
    app.world_mut().resource_mut::<GlobalVolume>().volume = Volume::Linear(0.5);
    app.update();
    assert!((spatial_volume(&app, voice) - 0.4).abs() < 0.0001);
    assert_eq!(regular_volume(&app), 0.0);
    assert_eq!(spatial_volume(&app, ambient), 0.0);
    assert!(
        app.world()
            .get::<SpatialAudioSink>(voice)
            .unwrap()
            .is_paused()
    );
    assert!(app.world().get::<AudioSink>(sfx).unwrap().is_paused());
    assert!(
        !app.world()
            .entity(voice)
            .get_ref::<SpatialAudioSink>()
            .unwrap()
            .is_added()
    );
    app.world_mut().resource_mut::<GlobalVolume>().volume = Volume::SILENT;
    app.update();
    assert_eq!(spatial_volume(&app, voice), 0.0);
}

#[test]
fn queued_channels_mute_independently_and_keep_master_out_of_playback_settings() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, GameplayChannelMixPlugin));
    app.world_mut().resource_mut::<GlobalVolume>().volume = Volume::Linear(0.2);
    let voice = app
        .world_mut()
        .spawn((
            GameplayAudioChannel::new(NativeAudioCategory::Voice, 1.0),
            PlaybackSettings::LOOP,
        ))
        .id();
    let effects = app
        .world_mut()
        .spawn((
            GameplayAudioChannel::ui_sfx(),
            PlaybackSettings::LOOP,
        ))
        .id();
    let ambient = app
        .world_mut()
        .spawn((
            GameplayAudioChannel::new(NativeAudioCategory::Ambient, 1.0),
            PlaybackSettings::LOOP.with_spatial(true),
        ))
        .id();
    app.update();
    let volume = |app: &App, entity| {
        app.world()
            .get::<PlaybackSettings>(entity)
            .unwrap()
            .volume
            .to_linear()
    };
    assert_eq!(volume(&app, voice), 0.5);
    assert_eq!(volume(&app, effects), 0.35);
    app.world_mut().resource_mut::<RetrobutionAudioMix>().voice = 0.0;
    app.update();
    assert_eq!(volume(&app, voice), 0.0);
    assert_eq!(volume(&app, effects), 0.35);
    assert_eq!(volume(&app, ambient), 0.5);
    app.world_mut()
        .resource_mut::<RetrobutionAudioMix>()
        .effects = 0.0;
    app.world_mut().resource_mut::<RetrobutionAudioMix>().voice = 0.8;
    app.world_mut()
        .resource_mut::<RetrobutionAudioMix>()
        .ambient = 0.0;
    app.update();
    assert_eq!(volume(&app, voice), 0.8);
    assert_eq!(volume(&app, effects), 0.0);
    assert_eq!(volume(&app, ambient), 0.0);
}
