//! Exact playback for the authored `EPbarrier` infected-zone effects.
//!
//! The static world exporter preserved the prefab's visible GLB surfaces, but
//! baked every one at frame zero. All 799 primary emitters across 33 unified
//! map tiles resolve the same `Tutorial.resourceFile` GameObject 4961
//! (`EPbarrier1`) and AnimationClip 1093 (`nif-default`). This module restores
//! that clip without making the proprietary bundle a runtime input.
//!
//! Primary acceptance source: `primary:Tutorial.resourceFile`, 27,003,826
//! bytes, SHA-256
//! `49a684ff4236848d0b882a5d725ffbe99d5cfb5d2090dc8705350e97dd3fd024`.
//! The focused clip dump was produced with
//! `fusionforge fusionforge dump-object <Tutorial serialized asset> 1093
//! <output.json>` from serialized asset
//! `CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a` (the patched cache
//! was navigation only; the raw primary container remains authoritative).
//! Ordinary-zone ownership was accepted against
//! `primary:Map_12_03.unity3d`, 114,906 bytes, SHA-256
//! `e974490a30e1a31dabe677423583d0127788fbac48f7f84d251abba7a9a5a631`:
//! serialized `BuildPlayer-Map_12_03` GameObject 13669 owns
//! `EffectEmitterController` MonoBehaviour 13670, which resolves shared
//! `EPbarrier1` pathId 4961 and particle prefab pathId 5101.
//! Published ring GLBs declare `transformPolicy = "local native mesh;
//! placement transform is external"`. Consequently the clip's local scale
//! curves must preserve each scene visual's external translation; treating
//! that world translation as a mesh-space pivot launches the rings thousands
//! of units away instead of producing the source outward wave.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::{legacy_model_material::LegacyModelMaterial, world::SpawnedNativeWorldVisual};

const CLIP_DURATION_SECONDS: f32 = 5.999_995_7;

#[derive(Debug, Clone, Copy)]
struct CurveKey {
    time: f32,
    value: f32,
    in_slope: f32,
    out_slope: f32,
}

impl CurveKey {
    const fn new(time: f32, value: f32, in_slope: f32, out_slope: f32) -> Self {
        Self {
            time,
            value,
            in_slope,
            out_slope,
        }
    }
}

const RING_SCALE_0: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 9.771_798, 9.771_798),
    CurveKey::new(0.033_333_335, 1.325_726_6, 11.296_366, 11.296_366),
    CurveKey::new(1.699_999_7, 74.213_85, 32.324_898, 32.324_898),
    CurveKey::new(1.999_999_4, 80.509_96, 9.771_662, 9.771_662),
];
const RING_COLOR_0: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, -0.499_999_5, -0.499_999_5),
    CurveKey::new(1.666_666_4, 0.166_666_8, -0.5, -0.5),
    CurveKey::new(1.999_999_4, 2.980_232_2e-7, -0.5, -0.5),
];
const RING_EMISSION_0: &[CurveKey] = &[
    CurveKey::new(0.0, 0.705_882_4, -0.017_450_452, -0.017_450_452),
    CurveKey::new(1.166_666_9, 0.265_522_75, -0.514_509_44, -0.514_509_44),
    CurveKey::new(
        1.966_666_1,
        0.000_581_681_73,
        -0.034_511_123,
        -0.034_511_123,
    ),
    CurveKey::new(1.999_999_4, -5.960_464_5e-8, -0.017_452_257, -0.017_452_257),
];

const RING_SCALE_1: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 0.0, 0.0),
    CurveKey::new(0.966_667_06, 1.0, 0.000_053_644_23, 0.000_053_644_23),
    CurveKey::new(1.000_000_4, 1.000_003_6, 6.041_836, 6.041_836),
    CurveKey::new(1.033_333_7, 1.402_788_6, 14.252_705, 14.252_705),
    CurveKey::new(2.633_332_3, 79.785_34, 14.252_829, 14.252_829),
    CurveKey::new(2.933_332, 80.188_14, 0.0, 0.0),
    CurveKey::new(4.599_997, 80.188_14, 0.0, 0.0),
    CurveKey::new(CLIP_DURATION_SECONDS, 80.188_14, 0.0, 0.0),
];
const RING_COLOR_1: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 0.0, 0.0),
    CurveKey::new(0.966_667_06, 1.0, -0.000_003_576_282, -0.000_003_576_282),
    CurveKey::new(1.033_333_7, 0.979_999_8, -0.6, -0.6),
    CurveKey::new(2.633_332_3, 0.020_000_696, -0.6, -0.6),
    CurveKey::new(2.666_665_6, 7.152_557_4e-7, -0.300_010_74, -0.300_010_74),
    CurveKey::new(2.699_998_9, 0.0, -0.000_010_728_846, -0.000_010_728_846),
    CurveKey::new(2.733_332_2, 0.0, 0.0, 0.0),
    CurveKey::new(2.799_998_8, 0.0, 0.0, 0.0),
    CurveKey::new(4.466_664, 0.0, 0.0, 0.0),
    CurveKey::new(CLIP_DURATION_SECONDS, 0.0, 0.0, 0.0),
];
const RING_EMISSION_1: &[CurveKey] = &[
    CurveKey::new(0.0, 0.705_882_4, 0.0, 0.0),
    CurveKey::new(1.000_000_4, 0.705_882_4, -0.012_536_657, -0.012_536_657),
    CurveKey::new(2.366_665_8, 0.060_378_61, -0.374_739_83, -0.374_739_83),
    CurveKey::new(2.633_332_3, 0.000_835_776_3, -0.049_469_814, -0.049_469_814),
    CurveKey::new(2.666_665_6, -5.960_464_5e-8, -0.012_536_657, -0.012_536_657),
    CurveKey::new(2.699_998_9, 0.0, 8.940_705e-7, 8.940_705e-7),
    CurveKey::new(2.733_332_2, 0.0, 0.0, 0.0),
    CurveKey::new(4.399_997, 0.0, 0.0, 0.0),
    CurveKey::new(CLIP_DURATION_SECONDS, 0.0, 0.0, 0.0),
];

const RING_SCALE_2: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 0.0, 0.0),
    CurveKey::new(0.633_333_44, 1.0, 0.000_017_881_379, 0.000_017_881_379),
    CurveKey::new(0.666_666_8, 1.000_001_2, 6.325_578_7, 6.325_578_7),
    CurveKey::new(0.700_000_17, 1.421_705_6, 15.352_955, 15.352_955),
    CurveKey::new(2.299_999_2, 95.213_26, 15.353_18, 15.353_18),
    CurveKey::new(2.633_332_3, 95.634_964, 0.0, 0.0),
    CurveKey::new(4.299_997_3, 95.634_964, 0.0, 0.0),
    CurveKey::new(5.966_662_4, 95.634_964, 0.0, 0.0),
    CurveKey::new(CLIP_DURATION_SECONDS, 95.634_964, 0.0, 0.0),
];
const RING_COLOR_2: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 0.0, 0.0),
    CurveKey::new(0.633_333_44, 1.0, -8.940_689e-7, -8.940_689e-7),
    CurveKey::new(0.700_000_17, 0.979_999_9, -0.599_999_85, -0.599_999_85),
    CurveKey::new(2.299_999_2, 0.020_000_696, -0.6, -0.6),
    CurveKey::new(2.333_332_5, 7.152_557_4e-7, -0.300_010_74, -0.300_010_74),
    CurveKey::new(2.366_665_8, 0.0, -0.000_010_728_846, -0.000_010_728_846),
    CurveKey::new(2.399_999_1, 0.0, 0.0, 0.0),
    CurveKey::new(2.466_665_7, 0.0, 0.0, 0.0),
    CurveKey::new(4.133_331, 0.0, 0.0, 0.0),
    CurveKey::new(5.799_996, 0.0, 0.0, 0.0),
    CurveKey::new(CLIP_DURATION_SECONDS, 0.0, 0.0, 0.0),
];

const RING_SCALE_3: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 0.0, 0.0),
    CurveKey::new(1.300_000_1, 1.0, 0.0, 0.0),
    CurveKey::new(1.333_333_4, 1.0, 4.885_898_6, 4.885_898_6),
    CurveKey::new(1.366_666_7, 1.325_726_3, 11.296_363, 11.296_363),
    CurveKey::new(3.033_331_9, 74.213_8, 32.325_012, 32.325_012),
    CurveKey::new(3.433_331_5, 80.509_96, 0.0, 0.0),
    CurveKey::new(5.099_996_6, 80.509_96, 0.0, 0.0),
    CurveKey::new(CLIP_DURATION_SECONDS, 80.509_96, 0.0, 0.0),
];
const RING_COLOR_3: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 0.0, 0.0),
    CurveKey::new(1.300_000_1, 1.0, 0.0, 0.0),
    CurveKey::new(1.499_999_9, 0.916_666_75, -0.5, -0.5),
    CurveKey::new(3.166_665, 0.083_334_27, -0.5, -0.5),
    CurveKey::new(3.299_998_3, 0.016_667_664, -0.5, -0.5),
    CurveKey::new(3.333_331_6, 0.000_001_013_279, -0.250_015_2, -0.250_015_2),
    CurveKey::new(3.366_665, 0.0, -0.000_015_199_199, -0.000_015_199_199),
    CurveKey::new(3.399_998_2, 0.0, 0.0, 0.0),
    CurveKey::new(3.466_664_8, 0.0, 0.0, 0.0),
    CurveKey::new(5.133_33, 0.0, 0.0, 0.0),
    CurveKey::new(CLIP_DURATION_SECONDS, 0.0, 0.0, 0.0),
];
const RING_EMISSION_3: &[CurveKey] = &[
    CurveKey::new(0.0, 0.705_882_4, 0.0, 0.0),
    CurveKey::new(1.333_333_4, 0.705_882_4, -0.008_725_234, -0.008_725_234),
    CurveKey::new(2.999_998_6, 0.052_287_996, -0.293_922_13, -0.293_922_13),
    CurveKey::new(
        3.299_998_3,
        0.000_581_741_33,
        -0.034_512_017,
        -0.034_512_017,
    ),
    CurveKey::new(3.333_331_6, -5.960_464_5e-8, -0.008_726_128, -0.008_726_128),
    CurveKey::new(3.366_665, 0.0, 8.940_705e-7, 8.940_705e-7),
    CurveKey::new(3.399_998_2, 0.0, 0.0, 0.0),
    CurveKey::new(5.066_663_3, 0.0, 0.0, 0.0),
    CurveKey::new(CLIP_DURATION_SECONDS, 0.0, 0.0, 0.0),
];

const RING_SCALE_4: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 0.0, 0.0),
    CurveKey::new(1.666_666_4, 1.0, 0.0, 0.0),
    CurveKey::new(2.633_332_3, 1.0, 0.0, 0.0),
    CurveKey::new(2.666_665_6, 1.0, 4.885_695, 4.885_695),
    CurveKey::new(2.699_998_9, 1.325_712_7, 11.296_107, 11.296_107),
    CurveKey::new(4.366_664, 74.213_76, 32.325_127, 32.325_127),
    CurveKey::new(4.666_663_6, 80.509_94, 9.772_577, 9.772_577),
];
const RING_COLOR_4: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 0.0, 0.0),
    CurveKey::new(1.666_666_4, 1.0, 0.0, 0.0),
    CurveKey::new(2.633_332_3, 1.0, 0.0, 0.0),
    CurveKey::new(2.833_332, 0.916_667_34, -0.5, -0.5),
    CurveKey::new(4.499_997, 0.083_334_92, -0.5, -0.5),
    CurveKey::new(4.666_663_6, 0.000_001_668_930_1, -0.5, -0.5),
];
const RING_EMISSION_4: &[CurveKey] = &[
    CurveKey::new(0.0, 0.705_882_4, 0.0, 0.0),
    CurveKey::new(1.666_666_4, 0.705_882_4, 0.0, 0.0),
    CurveKey::new(2.666_665_6, 0.705_882_4, -0.008_725_234, -0.008_725_234),
    CurveKey::new(4.333_330_6, 0.052_288_413, -0.293_923_02, -0.293_923_02),
    CurveKey::new(
        4.633_330_3,
        0.000_581_681_73,
        -0.034_513_805,
        -0.034_513_805,
    ),
    CurveKey::new(4.666_663_6, -5.960_464_5e-8, -0.017_452_257, -0.017_452_257),
];

const RING_SCALE_5: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 0.0, 0.0),
    CurveKey::new(1.666_666_4, 1.0, 0.0, 0.0),
    CurveKey::new(3.333_331_6, 1.0, 0.0, 0.0),
    CurveKey::new(3.966_664_3, 1.0, 0.0, 0.0),
    CurveKey::new(3.999_997_6, 1.0, 4.885_493, 4.885_493),
    CurveKey::new(4.033_331, 1.325_699_2, 11.295_852, 11.295_852),
    CurveKey::new(5.699_996, 74.213_73, 32.325_127, 32.325_127),
    CurveKey::new(CLIP_DURATION_SECONDS, 80.509_93, 9.772_348, 9.772_348),
];
const RING_COLOR_5: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 0.0, 0.0),
    CurveKey::new(1.666_666_4, 1.0, 0.0, 0.0),
    CurveKey::new(3.333_331_6, 1.0, 0.0, 0.0),
    CurveKey::new(3.966_664_3, 1.0, 0.0, 0.0),
    CurveKey::new(4.166_664, 0.916_667_94, -0.5, -0.5),
    CurveKey::new(5.833_329, 0.083_335_4, -0.5, -0.5),
    CurveKey::new(CLIP_DURATION_SECONDS, 0.000_002_145_767_2, -0.5, -0.5),
];
const RING_EMISSION_5: &[CurveKey] = &[
    CurveKey::new(0.0, 0.705_882_4, 0.0, 0.0),
    CurveKey::new(1.666_666_4, 0.705_882_4, 0.0, 0.0),
    CurveKey::new(3.333_331_6, 0.705_882_4, 0.0, 0.0),
    CurveKey::new(4.033_331, 0.705_300_75, -0.034_507_547, -0.034_507_547),
    CurveKey::new(5.699_996, 0.042_883_396, -0.269_807_22, -0.269_807_22),
    CurveKey::new(5.966_662_4, 0.000_581_800_94, -0.034_514_7, -0.034_514_7),
    CurveKey::new(
        CLIP_DURATION_SECONDS,
        -5.960_464_5e-8,
        -0.017_455_833,
        -0.017_455_833,
    ),
];

const CENTER_SCALE: &[CurveKey] = &[
    CurveKey::new(0.0, 1.0, 0.007_567_405_2, 0.007_567_405_2),
    CurveKey::new(0.966_667_06, 1.077_147_7, 0.022_015_553, 0.022_015_553),
];
const CENTER_PULSE: &[CurveKey] = &[
    CurveKey::new(0.0, 0.0, 0.299_206_2, 0.299_206_2),
    CurveKey::new(0.033_333_335, 0.009_973_541, 0.573_987_5, 0.573_987_5),
    CurveKey::new(0.166_666_67, 0.208_630_17, 2.186_037_3, 2.186_037_3),
    CurveKey::new(0.533_333_36, 0.990_026_4, 0.573_987_8, 0.573_987_8),
    CurveKey::new(0.633_333_44, 0.936_276_56, -1.775_150_5, -1.775_150_5),
    CurveKey::new(0.933_333_7, 0.063_722_67, -1.775_139_8, -1.775_139_8),
    CurveKey::new(0.966_667_06, 0.016_840_756, -1.406_456_2, -1.406_456_2),
];

const RING_SCALE_CURVES: [&[CurveKey]; 6] = [
    RING_SCALE_0,
    RING_SCALE_1,
    RING_SCALE_2,
    RING_SCALE_3,
    RING_SCALE_4,
    RING_SCALE_5,
];
const RING_COLOR_CURVES: [&[CurveKey]; 6] = [
    RING_COLOR_0,
    RING_COLOR_1,
    RING_COLOR_2,
    RING_COLOR_3,
    RING_COLOR_4,
    RING_COLOR_5,
];
const RING_EMISSION_CURVES: [Option<&[CurveKey]>; 6] = [
    Some(RING_EMISSION_0),
    Some(RING_EMISSION_1),
    None,
    Some(RING_EMISSION_3),
    Some(RING_EMISSION_4),
    Some(RING_EMISSION_5),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EpBarrierSurfaceKind {
    Ring { phase: usize },
    Center,
}

// These are the stable published model identities shared by every primary
// EPbarrier instance. Per-tile `sourceModelPath` ordinals are deliberately not
// used: ordinary infected-zone tiles start their 15-child prefab blocks at
// arbitrary v-indices, while the normalized model routes remain exact.
const EP_BARRIER_ANIMATED_SURFACES: [(&str, EpBarrierSurfaceKind); 13] = [
    (
        "objects/vehicles/motttt/models/plane11_dupe1/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 5 },
    ),
    (
        "objects/vehicles/motttt/models/plane11/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 3 },
    ),
    (
        "objects/vehicles/motttt/models/plane10_dupe1/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 4 },
    ),
    (
        "objects/vehicles/motttt/models/plane10/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 2 },
    ),
    (
        "objects/vehicles/motttt/models/plane08/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 0 },
    ),
    (
        "objects/vehicles/motttt/models/plane09/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 1 },
    ),
    (
        "objects/vehicles/motttt/models/plane07/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 5 },
    ),
    (
        "objects/vehicles/motttt/models/plane04/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 3 },
    ),
    (
        "objects/vehicles/motttt/models/plane06/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 4 },
    ),
    (
        "objects/vehicles/motttt/models/plane03/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 2 },
    ),
    (
        "objects/vehicles/motttt/models/plane01/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 0 },
    ),
    (
        "objects/vehicles/motttt/models/plane02/visual.glb",
        EpBarrierSurfaceKind::Ring { phase: 1 },
    ),
    (
        "objects/effects/epbarrier1_03_defaultsd_forcefieldgenerator1/models/epbarrier1_03_defaultsd_forcefieldgenerator1/visual.glb",
        EpBarrierSurfaceKind::Center,
    ),
];

#[cfg(test)]
const EP_BARRIER_STATIC_MODEL_PATHS: [&str; 2] = [
    "objects/effects/epbarrier1_03_default_forcefieldgenerator/models/epbarrier1_03_default_forcefieldgenerator/visual.glb",
    "objects/unclassified/no/models/no/visual.glb",
];

#[derive(Component, Debug, Clone)]
struct TutorialEpBarrierSurface {
    baseline: Transform,
    kind: EpBarrierSurfaceKind,
    started_at_seconds: f32,
    instance_key: String,
}

#[derive(Component, Debug)]
struct TutorialEpBarrierMaterialInstance {
    root: Entity,
}

#[derive(Resource, Debug, Default)]
struct TutorialEpBarrierTimelines(HashMap<String, f32>);

pub struct TutorialEpBarrierPlugin;

impl Plugin for TutorialEpBarrierPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialEpBarrierTimelines>()
            .add_systems(
                Update,
                (
                    tag_tutorial_ep_barrier_surfaces,
                    tag_tutorial_ep_barrier_materials,
                    animate_tutorial_ep_barrier_materials,
                    animate_tutorial_ep_barrier_transforms,
                )
                    .chain(),
            );
    }
}

fn tag_tutorial_ep_barrier_surfaces(
    mut commands: Commands,
    time: Res<Time>,
    mut timelines: ResMut<TutorialEpBarrierTimelines>,
    existing: Query<&TutorialEpBarrierSurface>,
    visuals: Query<
        (Entity, &Name, &SpawnedNativeWorldVisual, &Transform),
        Added<SpawnedNativeWorldVisual>,
    >,
) {
    let active_instances = existing
        .iter()
        .map(|surface| surface.instance_key.as_str())
        .collect::<HashSet<_>>();
    timelines
        .0
        .retain(|instance, _| active_instances.contains(instance.as_str()));

    let now = time.elapsed_secs();
    for (entity, name, visual, transform) in &visuals {
        let Some(kind) = classify_surface(&visual.model_path) else {
            continue;
        };
        let Some(instance_key) = barrier_instance_key(name.as_str()) else {
            continue;
        };
        let started_at_seconds = *timelines.0.entry(instance_key.clone()).or_insert(now);
        commands.entity(entity).insert(TutorialEpBarrierSurface {
            baseline: *transform,
            kind,
            started_at_seconds,
            instance_key,
        });
    }
}

fn animate_tutorial_ep_barrier_transforms(
    time: Res<Time>,
    renderers: Query<(&TutorialEpBarrierMaterialInstance, Option<&ViewVisibility>)>,
    mut surfaces: Query<(Entity, &TutorialEpBarrierSurface, &mut Transform)>,
) {
    // EPbarrier is published 799 times. Advancing time is implicit from the
    // global clock, so an off-screen instance does not need to dirty thirteen
    // transform trees every frame. It is sampled at the current exact clip
    // time as soon as any of its renderers becomes visible again.
    let visible_roots = renderers
        .iter()
        .filter(|(_, visibility)| visibility.is_none_or(|visibility| visibility.get()))
        .map(|(instance, _)| instance.root)
        .collect::<HashSet<_>>();
    let now = time.elapsed_secs();
    for (entity, surface, mut transform) in &mut surfaces {
        if !visible_roots.contains(&entity) {
            continue;
        }
        let clip_time = (now - surface.started_at_seconds).rem_euclid(CLIP_DURATION_SECONDS);
        let scale = match surface.kind {
            EpBarrierSurfaceKind::Ring { phase } => {
                evaluate_curve(RING_SCALE_CURVES[phase], clip_time, 1.0)
            }
            EpBarrierSurfaceKind::Center => evaluate_curve(CENTER_SCALE, clip_time, 1.0),
        };
        let sampled = scale_from_authored_local_pivot(surface.baseline, scale);
        if *transform != sampled {
            *transform = sampled;
        }
    }
}

/// Isolate the material once, when Bevy adds a renderer below an EPbarrier
/// visual. The old animation system rediscovered ancestry for every material
/// in the entire world on every frame, even though ownership never changes.
fn tag_tutorial_ep_barrier_materials(
    mut commands: Commands,
    parents: Query<&ChildOf>,
    roots: Query<&TutorialEpBarrierSurface>,
    mut renderers: Query<
        (Entity, &mut MeshMaterial3d<LegacyModelMaterial>),
        Added<MeshMaterial3d<LegacyModelMaterial>>,
    >,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
) {
    for (entity, mut material_handle) in &mut renderers {
        let Some((root, _)) = ancestor_surface_with_entity(entity, &parents, &roots) else {
            continue;
        };
        let Some(instance_material) = materials.get(&material_handle.0).cloned() else {
            continue;
        };
        material_handle.0 = materials.add(instance_material);
        commands
            .entity(entity)
            .insert(TutorialEpBarrierMaterialInstance { root });
    }
}

fn animate_tutorial_ep_barrier_materials(
    time: Res<Time>,
    roots: Query<&TutorialEpBarrierSurface>,
    renderers: Query<(
        Entity,
        &MeshMaterial3d<LegacyModelMaterial>,
        &TutorialEpBarrierMaterialInstance,
        Option<&ViewVisibility>,
    )>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
) {
    let now = time.elapsed_secs();
    for (_entity, material_handle, instance, visibility) in &renderers {
        if !visibility.is_none_or(|visibility| visibility.get()) {
            continue;
        }
        let Ok(surface) = roots.get(instance.root) else {
            continue;
        };
        let clip_time = (now - surface.started_at_seconds).rem_euclid(CLIP_DURATION_SECONDS);
        let changed = {
            let Some(material) = materials.get_mut_untracked(&material_handle.0) else {
                continue;
            };
            match surface.kind {
                EpBarrierSurfaceKind::Ring { phase } => {
                    let color = evaluate_curve(RING_COLOR_CURVES[phase], clip_time, 1.0).max(0.0);
                    let emission = RING_EMISSION_CURVES[phase]
                        .map_or(0.0, |curve| evaluate_curve(curve, clip_time, 0.0))
                        .max(0.0);
                    set_material_rgb(&mut material.uniform.base_color, color, color, color)
                        | set_material_rgb(
                            &mut material.uniform.emission,
                            emission,
                            emission,
                            emission,
                        )
                }
                EpBarrierSurfaceKind::Center => {
                    let pulse = evaluate_curve(CENTER_PULSE, clip_time, 0.0).max(0.0);
                    set_material_rgb(
                        &mut material.uniform.base_color,
                        pulse * (20.0 / 255.0),
                        pulse * (25.0 / 255.0),
                        pulse * (226.0 / 255.0),
                    ) | set_material_rgb(&mut material.uniform.emission, 0.0, 0.0, pulse)
                }
            }
        };
        // Emit exactly one asset event only when the sampled uniform changed.
        // That prevents Bevy/wgpu from rebuilding a bind group for an already
        // settled key or for an off-screen material.
        if changed {
            // Bevy 0.19 tracks mutable dereferences, not get_mut calls.
            // Publish the preceding untracked edits when a full rebuild is required.
            let _ = materials
                .get_mut(material_handle.id())
                .map(|material| material.into_inner());
        }
    }
}

fn set_material_rgb(color: &mut LinearRgba, red: f32, green: f32, blue: f32) -> bool {
    let changed = color.red.to_bits() != red.to_bits()
        || color.green.to_bits() != green.to_bits()
        || color.blue.to_bits() != blue.to_bits();
    if changed {
        color.red = red;
        color.green = green;
        color.blue = blue;
    }
    changed
}

fn barrier_instance_key(name: &str) -> Option<String> {
    let prefab = name.split_once("[prefab:")?.1;
    let (owner, _) = prefab.split_once(':')?;
    (!owner.is_empty()).then(|| owner.to_owned())
}

fn classify_surface(model_path: &str) -> Option<EpBarrierSurfaceKind> {
    let normalized = model_path.replace('\\', "/");
    EP_BARRIER_ANIMATED_SURFACES
        .iter()
        .find_map(|(path, kind)| (normalized == *path).then_some(*kind))
}

fn evaluate_curve(keys: &[CurveKey], time: f32, fallback: f32) -> f32 {
    let Some(first) = keys.first() else {
        return fallback;
    };
    if time <= first.time {
        return first.value;
    }
    let last = keys.last().unwrap();
    if time >= last.time {
        return last.value;
    }
    let pair = keys
        .windows(2)
        .find(|pair| time >= pair[0].time && time <= pair[1].time)
        .unwrap();
    let duration = pair[1].time - pair[0].time;
    if duration <= f32::EPSILON {
        return pair[1].value;
    }
    let t = (time - pair[0].time) / duration;
    let (t2, t3) = (t * t, t * t * t);
    (2.0 * t3 - 3.0 * t2 + 1.0) * pair[0].value
        + (t3 - 2.0 * t2 + t) * duration * pair[0].out_slope
        + (-2.0 * t3 + 3.0 * t2) * pair[1].value
        + (t3 - t2) * duration * pair[1].in_slope
}

fn scale_from_authored_local_pivot(baseline: Transform, scale: f32) -> Transform {
    Transform {
        // The source AnimationClip changes localScale only. The published GLB
        // keeps its local mesh pivot while `scene.json` owns placement, so the
        // external translation and rotation stay fixed throughout the wave.
        translation: baseline.translation,
        rotation: baseline.rotation,
        scale: baseline.scale * scale,
    }
}

fn ancestor_surface_with_entity<'a>(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    roots: &'a Query<&TutorialEpBarrierSurface>,
) -> Option<(Entity, &'a TutorialEpBarrierSurface)> {
    for _ in 0..64 {
        if let Ok(surface) = roots.get(entity) {
            return Some((entity, surface));
        }
        entity = parents.get(entity).ok()?.parent();
    }
    None
}

#[cfg(test)]
mod tests;
