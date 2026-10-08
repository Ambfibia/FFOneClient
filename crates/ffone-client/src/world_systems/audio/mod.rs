//! Retrobution world music, ambient beds and streamed positional emitters.
//!
//! The runtime consumes only the published native contract and semantic audio
//! catalog. Unity bundles and extraction caches are offline evidence and are
//! never opened here.

use std::collections::{HashMap, HashSet};

use bevy::{
    audio::{AudioSink, AudioSinkPlayback, SpatialListener, SpatialScale, Volume},
    prelude::*,
};
use serde::Deserialize;

use crate::{
    assets::AssetLocator,
    gameplay_audio::{GameplaySfxAudio, RetrobutionAudioMix},
    movement::LegacyPlayerController,
    semantic_audio::{NativeAudioCatalog, NativeAudioCategory},
    world::{NativeWorldSceneRoot, NativeWorldScope},
};

pub const RETROBUTION_WORLD_AUDIO_PATH: &str = "data/audio/retrobution-world.json";
const RETROBUTION_WORLD_AUDIO_SCHEMA: &str = "ffone.retrobution-world-audio.v1";
const MUSIC_SCAN_SECONDS: f32 = 1.0;
const ENVIRONMENT_SCAN_SECONDS: f32 = 2.0;
const ENVIRONMENT_START_DISTANCE: f32 = 30.0;
const ENVIRONMENT_STOP_DISTANCE: f32 = 40.0;
const ENVIRONMENT_ACTIVE_LIMIT: usize = 10;
const FADE_OUT_SECONDS: f32 = 0.5;
const FADE_IN_SECONDS: f32 = 1.0;
const ENVIRONMENT_SPATIAL_SCALE: SpatialScale = SpatialScale::new(0.4);

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WorldAudioDocument {
    schema: String,
    coordinate_space: String,
    source: serde_json::Value,
    conversion: serde_json::Value,
    counts: WorldAudioCounts,
    music_zones: Vec<WorldAudioZone>,
    ambient_zones: Vec<WorldAudioZone>,
    environment_tiles: Vec<WorldEnvironmentTile>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WorldAudioCounts {
    music_zones: usize,
    ambient_zones: usize,
    environment_tiles: usize,
    environment_emitters: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WorldAudioZone {
    index: usize,
    #[serde(rename = "name")]
    _name: String,
    instance: bool,
    delay_seconds: f64,
    source_file_name: String,
    source_true_name: Option<String>,
    logical_key: Option<String>,
    resolution: WorldAudioResolution,
    points: Vec<[f64; 2]>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
enum WorldAudioResolution {
    Resolved,
    Silence,
    LegacySourceMissing,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WorldEnvironmentTile {
    tile: [i32; 2],
    scope: NativeWorldScope,
    source: WorldAudioSource,
    object: WorldAudioObject,
    emitters: Vec<WorldEnvironmentEmitter>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WorldAudioSource {
    alias: String,
    relative_path: String,
    bytes: u64,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WorldAudioObject {
    extracted_asset: String,
    name: String,
    path_id: i64,
    script_file_id: i64,
    script_path_id: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WorldEnvironmentEmitter {
    source_file_name: String,
    source_true_name: String,
    logical_key: String,
    position: [f32; 3],
}

#[derive(Debug, Resource)]
pub struct RetrobutionWorldAudioCatalog {
    music_zones: Vec<WorldAudioZone>,
    ambient_zones: Vec<WorldAudioZone>,
    environment_tiles: HashMap<(NativeWorldScope, [i32; 2]), WorldEnvironmentTile>,
}

impl RetrobutionWorldAudioCatalog {
    pub fn open(locator: &AssetLocator, audio: &NativeAudioCatalog) -> Result<Self, String> {
        let document: WorldAudioDocument = locator.read_json(RETROBUTION_WORLD_AUDIO_PATH)?;
        if document.schema != RETROBUTION_WORLD_AUDIO_SCHEMA {
            return Err(format!(
                "world audio has schema {:?}, expected {:?}",
                document.schema, RETROBUTION_WORLD_AUDIO_SCHEMA
            ));
        }
        if !document.coordinate_space.contains("H=diag(-1,1,1)")
            || document.source.is_null()
            || document.conversion.is_null()
        {
            return Err("world audio is missing coordinate or primary provenance".to_owned());
        }
        validate_zones("music", &document.music_zones, audio)?;
        validate_zones("ambient", &document.ambient_zones, audio)?;
        let emitter_count = document
            .environment_tiles
            .iter()
            .map(|tile| tile.emitters.len())
            .sum::<usize>();
        if document.counts.music_zones != document.music_zones.len()
            || document.counts.ambient_zones != document.ambient_zones.len()
            || document.counts.environment_tiles != document.environment_tiles.len()
            || document.counts.environment_emitters != emitter_count
        {
            return Err("world audio count summary contradicts its payload".to_owned());
        }
        let mut environment_tiles = HashMap::new();
        for tile in document.environment_tiles {
            if !(0..16).contains(&tile.tile[0]) || !(0..16).contains(&tile.tile[1]) {
                return Err(format!("environment tile {:?} is outside 16x16", tile.tile));
            }
            if tile.source.alias != "primary"
                || tile.source.relative_path.is_empty()
                || tile.source.bytes == 0
                || tile.source.sha256.len() != 64
                || tile.object.extracted_asset.is_empty()
                || !tile.object.name.starts_with("sound_")
                || tile.object.path_id <= 0
                || tile.object.script_file_id
                    != match tile.scope {
                        NativeWorldScope::WorldMap => 1,
                        NativeWorldScope::Tutorial => 0,
                    }
                || tile.object.script_path_id != 1908
            {
                return Err(format!(
                    "environment tile {:?} has incomplete clean-primary ownership",
                    tile.tile
                ));
            }
            for emitter in &tile.emitters {
                if emitter.source_file_name.is_empty()
                    || emitter.source_true_name.is_empty()
                    || emitter.position.iter().any(|value| !value.is_finite())
                {
                    return Err(format!(
                        "environment tile {:?} contains an invalid emitter",
                        tile.tile
                    ));
                }
                let resolved = audio.by_logical_key(&emitter.logical_key).ok_or_else(|| {
                    format!(
                        "environment {:?} uses missing logical key {:?}",
                        emitter.source_file_name, emitter.logical_key
                    )
                })?;
                if !resolved
                    .true_name
                    .eq_ignore_ascii_case(&emitter.source_true_name)
                {
                    return Err(format!(
                        "environment key {:?} resolves to {:?}, expected {:?}",
                        emitter.logical_key, resolved.true_name, emitter.source_true_name
                    ));
                }
            }
            let key = (tile.scope, tile.tile);
            if environment_tiles.insert(key, tile).is_some() {
                return Err(format!("world audio duplicates environment tile {key:?}"));
            }
        }
        Ok(Self {
            music_zones: document.music_zones,
            ambient_zones: document.ambient_zones,
            environment_tiles,
        })
    }

    #[must_use]
    pub fn counts(&self) -> (usize, usize, usize, usize) {
        (
            self.music_zones.len(),
            self.ambient_zones.len(),
            self.environment_tiles.len(),
            self.environment_tiles
                .values()
                .map(|tile| tile.emitters.len())
                .sum(),
        )
    }
}

fn validate_zones(
    label: &str,
    zones: &[WorldAudioZone],
    audio: &NativeAudioCatalog,
) -> Result<(), String> {
    for (position, zone) in zones.iter().enumerate() {
        if zone.index != position
            || zone.points.len() < 3
            || zone.delay_seconds < 0.0
            || !zone.delay_seconds.is_finite()
            || zone
                .points
                .iter()
                .flatten()
                .any(|coordinate| !coordinate.is_finite())
        {
            return Err(format!("{label} zone {position} is structurally invalid"));
        }
        match zone.resolution {
            WorldAudioResolution::Resolved => {
                let logical_key = zone.logical_key.as_deref().ok_or_else(|| {
                    format!("{label} zone {position} resolved without a logical key")
                })?;
                let resolved = audio.by_logical_key(logical_key).ok_or_else(|| {
                    format!("{label} zone {position} uses missing key {logical_key:?}")
                })?;
                if !zone
                    .source_true_name
                    .as_deref()
                    .is_some_and(|true_name| true_name.eq_ignore_ascii_case(&resolved.true_name))
                {
                    return Err(format!(
                        "{label} zone {position} key {logical_key:?} no longer owns {:?}",
                        zone.source_true_name
                    ));
                }
            }
            WorldAudioResolution::Silence => {
                if zone.logical_key.is_some() || zone.source_file_name != "none.ogg" {
                    return Err(format!("{label} silence zone {position} is contradictory"));
                }
            }
            WorldAudioResolution::LegacySourceMissing => {
                if zone.logical_key.is_some() || zone.source_true_name.is_none() {
                    return Err(format!("{label} missing zone {position} is contradictory"));
                }
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, Resource)]
pub struct RetrobutionInstanceAudioState {
    pub active: bool,
}

/// Loading uses the lobby coordinate until the world is ready for play.
#[derive(Clone, Copy, Debug, Default, Resource)]
pub struct RetrobutionLoadingAudioState {
    pub active: bool,
}

#[derive(Debug, Default, Resource)]
pub struct RetrobutionMusicRequests {
    pending: Option<(String, bool)>,
}

impl RetrobutionMusicRequests {
    pub fn request(&mut self, name: String, repeat: bool) {
        self.pending = Some((name, repeat));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ZoneTarget {
    index: usize,
    logical_key: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ChannelPhase {
    #[default]
    Stopped,
    WaitingForSink,
    FadingIn,
    Stable,
    FadingOut,
}

#[derive(Debug, Default)]
struct ChannelRuntime {
    phase: ChannelPhase,
    current_entity: Option<Entity>,
    current: Option<ZoneTarget>,
    target: Option<ZoneTarget>,
    transition_elapsed: f32,
    last_started: Vec<f64>,
    selected_zone: Option<usize>,
}

impl ChannelRuntime {
    fn with_zone_count(count: usize) -> Self {
        Self {
            last_started: vec![f64::NEG_INFINITY; count],
            ..default()
        }
    }

    fn request(&mut self, target: ZoneTarget) {
        // The 136 authored polygons resolve to about twenty distinct clips, so
        // adjacent zones very often name the same track. Retrobution keys its
        // running source on that clip, not on the polygon: crossing into
        // another zone that plays the same audio must keep the current source
        // instead of fading it out and restarting from zero. Only the owning
        // zone index moves across.
        if self
            .current
            .as_ref()
            .is_some_and(|current| current.logical_key == target.logical_key)
            && matches!(
                self.phase,
                ChannelPhase::WaitingForSink | ChannelPhase::FadingIn | ChannelPhase::Stable
            )
        {
            if let Some(current) = self.current.as_mut() {
                current.index = target.index;
            }
            return;
        }
        self.target = Some(target);
        self.transition_elapsed = 0.0;
        if self.current_entity.is_some() {
            self.phase = ChannelPhase::FadingOut;
        }
    }

    fn request_stop(&mut self) {
        if self.current_entity.is_none()
            && self
                .current
                .as_ref()
                .is_some_and(|current| current.logical_key.is_none())
        {
            return;
        }
        self.request(ZoneTarget {
            index: usize::MAX,
            logical_key: None,
        });
    }
}

#[derive(Debug, Resource)]
struct WorldAudioRuntime {
    scan_elapsed: f32,
    environment_elapsed: f32,
    elapsed: f64,
    was_loading: bool,
    music: ChannelRuntime,
    ambient: ChannelRuntime,
    override_track: Option<(ZoneTarget, bool)>,
}

impl FromWorld for WorldAudioRuntime {
    fn from_world(world: &mut World) -> Self {
        let catalog = world.resource::<RetrobutionWorldAudioCatalog>();
        Self {
            scan_elapsed: MUSIC_SCAN_SECONDS,
            environment_elapsed: ENVIRONMENT_SCAN_SECONDS,
            elapsed: 0.0,
            was_loading: false,
            music: ChannelRuntime::with_zone_count(catalog.music_zones.len()),
            ambient: ChannelRuntime::with_zone_count(catalog.ambient_zones.len()),
            override_track: None,
        }
    }
}

#[derive(Component)]
struct RetrobutionMusicAudio;

#[derive(Component)]
struct RetrobutionAmbientAudio;

#[derive(Component, Clone, Debug, Eq, PartialEq, Hash)]
struct RetrobutionEnvironmentAudio {
    scope: NativeWorldScope,
    tile: [i32; 2],
    emitter: usize,
}

pub struct RetrobutionWorldAudioPlugin;

impl Plugin for RetrobutionWorldAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RetrobutionInstanceAudioState>()
            .init_resource::<RetrobutionLoadingAudioState>()
            .init_resource::<RetrobutionMusicRequests>()
            .init_resource::<WorldAudioRuntime>()
            .add_systems(
                Update,
                (
                    select_retrobution_world_audio,
                    drive_retrobution_world_audio_channels.after(select_retrobution_world_audio),
                    drive_retrobution_environment_audio,
                ),
            );
    }
}

fn polygon_contains(points: &[[f64; 2]], point: [f64; 2]) -> bool {
    let mut inside = false;
    let mut previous = points.len() - 1;
    for current in 0..points.len() {
        let a = points[current];
        let b = points[previous];
        if (a[1] > point[1]) != (b[1] > point[1])
            && point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

fn resolve_music_request(
    name: &str,
    zones: &RetrobutionWorldAudioCatalog,
    audio: &NativeAudioCatalog,
) -> Option<String> {
    // The server cue omits the space in the track's semantic title.
    if name.eq_ignore_ascii_case("vsweeper.ogg") {
        return audio
            .by_logical_key("music/vs_weeper")
            .filter(|asset| asset.category == NativeAudioCategory::Music)
            .map(|asset| asset.logical_key.clone());
    }
    if let Some(key) = zones.music_zones.iter().find_map(|zone| {
        zone.source_file_name
            .eq_ignore_ascii_case(name)
            .then_some(zone.logical_key.as_ref())
            .flatten()
    }) {
        return Some(key.clone());
    }
    let stem = name.strip_suffix(".ogg").unwrap_or(name);
    let mut candidates = audio.assets().iter().filter(|asset| {
        asset.category == NativeAudioCategory::Music && asset.true_name.eq_ignore_ascii_case(stem)
    });
    let candidate = candidates.next()?;
    candidates
        .next()
        .is_none()
        .then(|| candidate.logical_key.clone())
}

fn eligible_zone(
    zones: &[WorldAudioZone],
    last_started: &[f64],
    point: [f64; 2],
    instance: bool,
    now: f64,
) -> Option<Option<ZoneTarget>> {
    for zone in zones {
        if zone.instance != instance || !polygon_contains(&zone.points, point) {
            continue;
        }
        if last_started[zone.index] + zone.delay_seconds < now {
            return Some(Some(ZoneTarget {
                index: zone.index,
                logical_key: zone.logical_key.clone(),
            }));
        }
        // Retrobution breaks on the first matching polygon even while its
        // replay delay is blocked; the current channel is left unchanged.
        return Some(None);
    }
    None
}

fn select_channel_zone(
    channel: &mut ChannelRuntime,
    zones: &[WorldAudioZone],
    point: [f64; 2],
    instance: bool,
    now: f64,
) {
    let zone = zones
        .iter()
        .find(|zone| zone.instance == instance && polygon_contains(&zone.points, point));
    let selected = zone.map(|zone| zone.index);
    let entered = channel.selected_zone != selected;
    channel.selected_zone = selected;
    let Some(zone) = zone else {
        channel.request_stop();
        return;
    };
    // Replay delays apply while staying in a polygon, never to a world-state
    // transition. Re-entering a region must replace the infected-zone track.
    if entered || channel.last_started[zone.index] + zone.delay_seconds < now {
        channel.request(ZoneTarget {
            index: zone.index,
            logical_key: zone.logical_key.clone(),
        });
    }
}

fn select_retrobution_world_audio(
    time: Res<Time>,
    catalog: Res<RetrobutionWorldAudioCatalog>,
    instance: Res<RetrobutionInstanceAudioState>,
    loading: Res<RetrobutionLoadingAudioState>,
    audio: Res<NativeAudioCatalog>,
    mut requests: ResMut<RetrobutionMusicRequests>,
    players: Query<&GlobalTransform, With<LegacyPlayerController>>,
    roots: Query<&NativeWorldSceneRoot>,
    mut runtime: ResMut<WorldAudioRuntime>,
    square_settings: Query<(&NativeWorldSceneRoot,&crate::world::NativeSquareSettings)>,
) {
    let delta = time.delta_secs().max(0.0);
    runtime.elapsed += f64::from(delta);
    runtime.scan_elapsed += delta;
    if runtime.scan_elapsed < MUSIC_SCAN_SECONDS {
        return;
    }
    runtime.scan_elapsed %= MUSIC_SCAN_SECONDS;
    debug!(
        "world audio scan: loading={} instance={} player={:?} world_roots={} elapsed={} phase={:?} current={:?} target={:?}",
        loading.active,
        instance.active,
        players.iter().next().map(GlobalTransform::translation),
        roots.iter().filter(|root| root.selection_scope == NativeWorldScope::WorldMap).count(),
        runtime.elapsed,
        runtime.music.phase,
        runtime.music.current,
        runtime.music.target,
    );
    let finished_loading = runtime.was_loading && !loading.active;
    runtime.was_loading = loading.active;
    if finished_loading {
        // A zone's repeat delay must never keep the loading theme alive.
        // Re-entering gameplay starts a fresh zone selection after a warp.
        runtime.music.last_started.fill(f64::NEG_INFINITY);
        runtime.music.selected_zone = None;
    }
    if let Some((name, repeat)) = requests.pending.take() {
        if name.eq_ignore_ascii_case("stop") {
            runtime.override_track = None;
            runtime.music.request_stop();
            runtime.ambient.request_stop();
            runtime.music.last_started.fill(f64::NEG_INFINITY);
            runtime.music.selected_zone = None;
            runtime.ambient.last_started.fill(f64::NEG_INFINITY);
            return;
        }
        if let Some(key) = resolve_music_request(&name, &catalog, &audio) {
            runtime.override_track = Some((
                ZoneTarget {
                    index: usize::MAX,
                    logical_key: Some(key),
                },
                repeat,
            ));
        } else {
            warn!("Server music request has no declared native track: {name:?}");
        }
    }
    let mut override_active = false;
    if let Some((target, repeat)) = runtime.override_track.clone() {
        // A non-repeating source override is consumed when MusicController
        // observes the requested name as its current track. This scan still
        // selects it; the next scan resumes the authored zone/delay rules.
        let acknowledged = runtime.music.current.as_ref() == Some(&target);
        runtime.music.request(target);
        override_active = true;
        if !repeat && acknowledged {
            runtime.override_track = None;
            runtime.music.selected_zone = None;
        }
    }
    if loading.active {
        if !override_active
            && let Some(Some(target)) = eligible_zone(
                &catalog.music_zones,
                &runtime.music.last_started,
                [1.0, 1.0],
                false,
                runtime.elapsed,
            )
        {
            runtime.music.request(target);
        }
        runtime.ambient.request_stop();
        return;
    }
    if !roots
        .iter()
        .any(|root| root.selection_scope == NativeWorldScope::WorldMap)
    {
        runtime.override_track = None;
        runtime.music.selected_zone = None;
        runtime.music.request_stop();
        runtime.ambient.request_stop();
        return;
    }
    let Some(player) = players.iter().next() else {
        if finished_loading && !override_active {
            runtime.music.request_stop();
        }
        return;
    };
    let native = player.translation();
    let point = [-f64::from(native.x), f64::from(native.z)];
    if !override_active {
        if let Some(key)=square_settings.iter().find(|(root,_)|root.tile==crate::world::square_at(native)).map(|(_,s)|s.music.trim()).filter(|s|!s.is_empty()) {
            if key=="stop" { runtime.music.request_stop();override_active=true; }
            else if let Some(key)=resolve_music_request(key,&catalog,&audio) {
                runtime.music.request(ZoneTarget{index:usize::MAX,logical_key:Some(key)});override_active=true;
            }
        }
    }
    if !override_active {
        let elapsed = runtime.elapsed;
        select_channel_zone(
            &mut runtime.music,
            &catalog.music_zones,
            point,
            instance.active,
            elapsed,
        );
    }
    if let Some(Some(target)) = eligible_zone(
        &catalog.ambient_zones,
        &runtime.ambient.last_started,
        point,
        instance.active,
        runtime.elapsed,
    ) {
        let effective_music_key = runtime
            .music
            .target
            .as_ref()
            .or(runtime.music.current.as_ref())
            .and_then(|value| value.logical_key.as_ref());
        if target.logical_key.as_ref() != effective_music_key {
            runtime.ambient.request(target);
        }
    }
}

fn spawn_channel_target<M: Component>(
    commands: &mut Commands,
    asset_server: &AssetServer,
    audio_catalog: &NativeAudioCatalog,
    channel: &mut ChannelRuntime,
    marker: M,
    label: &str,
    looping: bool,
    now: f64,
) {
    let Some(target) = channel.target.take() else {
        channel.phase = ChannelPhase::Stopped;
        return;
    };
    channel.transition_elapsed = 0.0;
    let Some(logical_key) = target.logical_key.as_deref() else {
        if let Some(last_started) = channel.last_started.get_mut(target.index) {
            *last_started = now;
        }
        channel.current = Some(target);
        channel.current_entity = None;
        channel.phase = ChannelPhase::Stopped;
        return;
    };
    let Some(audio) = audio_catalog.by_logical_key(logical_key) else {
        warn!("{label} target lost semantic audio key {logical_key:?}");
        if let Some(last_started) = channel.last_started.get_mut(target.index) {
            *last_started = now;
        }
        channel.phase = ChannelPhase::Stopped;
        return;
    };
    let settings = if looping {
        PlaybackSettings::LOOP
    } else {
        PlaybackSettings::DESPAWN
    }
    .with_volume(Volume::Linear(0.0));
    debug!("{label} starting {logical_key} in zone {}", target.index);
    let entity = commands
        .spawn((
            Name::new(format!("Retrobution {label} {}", audio.true_name)),
            marker,
            AudioPlayer::new(asset_server.load(audio.path.clone())),
            settings,
        ))
        .id();
    if let Some(last_started) = channel.last_started.get_mut(target.index) {
        *last_started = now;
    }
    channel.current = Some(target);
    channel.current_entity = Some(entity);
    channel.phase = ChannelPhase::WaitingForSink;
}

fn drive_channel<M: Component + Default>(
    commands: &mut Commands,
    asset_server: &AssetServer,
    audio_catalog: &NativeAudioCatalog,
    channel: &mut ChannelRuntime,
    sinks: &mut Query<&mut AudioSink>,
    delta: f32,
    gain: f32,
    label: &str,
    looping: bool,
    now: f64,
) {
    if channel.phase == ChannelPhase::Stopped && channel.target.is_some() {
        spawn_channel_target(
            commands,
            asset_server,
            audio_catalog,
            channel,
            M::default(),
            label,
            looping,
            now,
        );
    }
    match channel.phase {
        ChannelPhase::Stopped => {}
        ChannelPhase::WaitingForSink => {
            let Some(entity) = channel.current_entity else {
                channel.phase = ChannelPhase::Stopped;
                return;
            };
            let Ok(mut sink) = sinks.get_mut(entity) else {
                return;
            };
            debug!("{label} playback sink ready for {:?}", channel.current);
            sink.set_volume(Volume::Linear(0.0));
            channel.transition_elapsed = 0.0;
            channel.phase = ChannelPhase::FadingIn;
        }
        ChannelPhase::FadingIn => {
            let Some(entity) = channel.current_entity else {
                channel.phase = ChannelPhase::Stopped;
                return;
            };
            let Ok(mut sink) = sinks.get_mut(entity) else {
                return;
            };
            channel.transition_elapsed += delta;
            let fraction = (channel.transition_elapsed / FADE_IN_SECONDS).clamp(0.0, 1.0);
            sink.set_volume(Volume::Linear(gain * fraction));
            if fraction >= 1.0 {
                channel.phase = ChannelPhase::Stable;
            }
        }
        ChannelPhase::Stable => {
            let Some(entity) = channel.current_entity else {
                channel.phase = ChannelPhase::Stopped;
                return;
            };
            match sinks.get_mut(entity) {
                Ok(mut sink) if !sink.empty() => sink.set_volume(Volume::Linear(gain)),
                Ok(_) if !looping => {
                    channel.current = None;
                    channel.current_entity = None;
                    channel.phase = ChannelPhase::Stopped;
                }
                Ok(_) => {}
                Err(_) => {
                    channel.current = None;
                    channel.current_entity = None;
                    channel.phase = ChannelPhase::Stopped;
                }
            }
        }
        ChannelPhase::FadingOut => {
            let Some(entity) = channel.current_entity else {
                channel.current = None;
                channel.phase = ChannelPhase::Stopped;
                return;
            };
            let Ok(mut sink) = sinks.get_mut(entity) else {
                channel.current_entity = None;
                channel.current = None;
                channel.phase = ChannelPhase::Stopped;
                spawn_channel_target(
                    commands,
                    asset_server,
                    audio_catalog,
                    channel,
                    M::default(),
                    label,
                    looping,
                    now,
                );
                return;
            };
            channel.transition_elapsed += delta;
            let fraction = (channel.transition_elapsed / FADE_OUT_SECONDS).clamp(0.0, 1.0);
            sink.set_volume(Volume::Linear(gain * (1.0 - fraction)));
            if fraction >= 1.0 {
                sink.stop();
                commands.entity(entity).despawn();
                channel.current_entity = None;
                channel.current = None;
                channel.phase = ChannelPhase::Stopped;
                spawn_channel_target(
                    commands,
                    asset_server,
                    audio_catalog,
                    channel,
                    M::default(),
                    label,
                    looping,
                    now,
                );
            }
        }
    }
}

impl Default for RetrobutionMusicAudio {
    fn default() -> Self {
        Self
    }
}

impl Default for RetrobutionAmbientAudio {
    fn default() -> Self {
        Self
    }
}

fn drive_retrobution_world_audio_channels(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    audio_catalog: Res<NativeAudioCatalog>,
    mix: Res<RetrobutionAudioMix>,
    mut runtime: ResMut<WorldAudioRuntime>,
    mut sinks: Query<&mut AudioSink>,
) {
    let delta = time.delta_secs().max(0.0);
    let now = runtime.elapsed;
    drive_channel::<RetrobutionMusicAudio>(
        &mut commands,
        &asset_server,
        &audio_catalog,
        &mut runtime.music,
        &mut sinks,
        delta,
        mix.music.clamp(0.0, 1.0),
        "music",
        false,
        now,
    );
    drive_channel::<RetrobutionAmbientAudio>(
        &mut commands,
        &asset_server,
        &audio_catalog,
        &mut runtime.ambient,
        &mut sinks,
        delta,
        mix.ambient.clamp(0.0, 1.0),
        "ambient",
        true,
        now,
    );
}

fn drive_retrobution_environment_audio(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    catalog: Res<RetrobutionWorldAudioCatalog>,
    audio_catalog: Res<NativeAudioCatalog>,
    mix: Res<RetrobutionAudioMix>,
    roots: Query<&NativeWorldSceneRoot>,
    players: Query<&GlobalTransform, With<LegacyPlayerController>>,
    listeners: Query<&GlobalTransform, With<SpatialListener>>,
    gameplay_sounds: Query<(), With<GameplaySfxAudio>>,
    active: Query<(Entity, &RetrobutionEnvironmentAudio, &GlobalTransform)>,
    mut runtime: ResMut<WorldAudioRuntime>,
) {
    runtime.environment_elapsed += time.delta_secs().max(0.0);
    if runtime.environment_elapsed < ENVIRONMENT_SCAN_SECONDS {
        return;
    }
    runtime.environment_elapsed %= ENVIRONMENT_SCAN_SECONDS;
    let Some(listener) = players
        .iter()
        .next()
        .map(GlobalTransform::translation)
        .or_else(|| listeners.iter().next().map(GlobalTransform::translation))
    else {
        return;
    };
    let resident = roots
        .iter()
        .map(|root| (root.scope, root.tile))
        .collect::<HashSet<_>>();
    let mut active_keys = HashSet::new();
    let mut active_count = gameplay_sounds.iter().count();
    for (entity, marker, transform) in &active {
        let key = (marker.scope, marker.tile, marker.emitter);
        if !resident.contains(&(marker.scope, marker.tile))
            || transform.translation().distance(listener) > ENVIRONMENT_STOP_DISTANCE
        {
            commands.entity(entity).despawn();
        } else {
            active_keys.insert(key);
            active_count += 1;
        }
    }
    for &(scope, tile) in &resident {
        let Some(definition) = catalog.environment_tiles.get(&(scope, tile)) else {
            continue;
        };
        for (emitter_index, emitter) in definition.emitters.iter().enumerate() {
            if active_count >= ENVIRONMENT_ACTIVE_LIMIT
                || active_keys.contains(&(scope, tile, emitter_index))
            {
                continue;
            }
            let position = Vec3::from_array(emitter.position);
            if position.distance(listener) >= ENVIRONMENT_START_DISTANCE {
                continue;
            }
            let Some(audio) = audio_catalog.by_logical_key(&emitter.logical_key) else {
                continue;
            };
            commands.spawn((
                Name::new(format!(
                    "EnviroSound {} [{}_{},{}]",
                    emitter.source_file_name, tile[0], tile[1], emitter_index
                )),
                RetrobutionEnvironmentAudio {
                    scope,
                    tile,
                    emitter: emitter_index,
                },
                crate::audio_channel::GameplayAudioChannel::new(NativeAudioCategory::Ambient, 1.0),
                Transform::from_translation(position),
                AudioPlayer::new(asset_server.load(audio.path.clone())),
                PlaybackSettings::LOOP
                    .with_volume(Volume::Linear(mix.ambient.clamp(0.0, 1.0)))
                    .with_spatial(true)
                    .with_spatial_scale(ENVIRONMENT_SPATIAL_SCALE),
            ));
            active_count += 1;
        }
    }
}

#[cfg(test)]
mod tests;
