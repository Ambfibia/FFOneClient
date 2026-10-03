use super::*;

pub(super) const EFFECTS_DEPENDENCY_B4: &str = "CustomAssetBundle-b4f543c102ded400fbc6f1da25d9679a";

pub(super) const EFFECTS_DEPENDENCY_BD5: &str = "CustomAssetBundle-bd5f53480423447d7bcaed95cb2a96c8";

// Primary Tutorial.resourceFile AnimationClip.m_Events. Unity invokes these
// from the Animation component's actual playback clock, after the asynchronous
// model has loaded, and once per repeated pass. Keeping them on the scene clock
// made the projectile appear before or after the weapon pose depending on the
// machine's asset-load time.
pub(super) const RETROBUTION_ACTOR_EFFECT_EVENTS: &[RetrobutionActorEffectEvent] = &[
    RetrobutionActorEffectEvent {
        npc_type: 2664,
        clip: "melee1",
        source_clip_path_id: 1_142,
        event_seconds: 0.9,
        effect_id: 736,
        node_name: Some("tag01"),
    },
    RetrobutionActorEffectEvent {
        npc_type: 2666,
        clip: "melee1event",
        source_clip_path_id: 1_103,
        event_seconds: 0.133_333,
        effect_id: 751,
        node_name: None,
    },
    RetrobutionActorEffectEvent {
        npc_type: 2666,
        clip: "melee1event",
        source_clip_path_id: 1_103,
        event_seconds: 2.8,
        effect_id: 38,
        node_name: Some("Bip01 R Hand"),
    },
    RetrobutionActorEffectEvent {
        npc_type: 2666,
        clip: "melee1event",
        source_clip_path_id: 1_103,
        event_seconds: 2.854_584,
        effect_id: 38,
        node_name: Some("Bip01 L Hand"),
    },
    RetrobutionActorEffectEvent {
        npc_type: 2667,
        clip: "melee1",
        source_clip_path_id: 892,
        event_seconds: 0.180_000_01,
        effect_id: 734,
        node_name: Some("tag01"),
    },
    RetrobutionActorEffectEvent {
        npc_type: 2668,
        clip: "melee1",
        source_clip_path_id: 1_018,
        event_seconds: 0.264,
        effect_id: 734,
        node_name: Some("tag01"),
    },
    RetrobutionActorEffectEvent {
        npc_type: 2669,
        clip: "melee1",
        source_clip_path_id: 1_117,
        event_seconds: 0.233_333,
        effect_id: 734,
        node_name: Some("tag01"),
    },
];

// `DeadMotion.Dead` initializes fDeadTimer to `-death.length + 1`, and its
// Update generates the payload once fDeadTimer becomes positive. The primary
// mob_spawn death clip is exactly 1.6 s, so a player kill owns ES372 and the
// Oni 76/77 pair at t=0.6. Merely forcing AnimationNpc("death") does not arm
// this path and is intentionally distinguished by the marker component.
pub(super) const RETROBUTION_ACTOR_DEATH_PRESENTATION_EVENTS: &[RetrobutionActorDeathPresentationEvent] = &[
    RetrobutionActorDeathPresentationEvent {
        npc_type: 2674,
        source_clip_path_id: 1_126,
        event_seconds: 0.6,
        effect_id: 372,
    },
    RetrobutionActorDeathPresentationEvent {
        npc_type: 2897,
        source_clip_path_id: 1_126,
        event_seconds: 0.6,
        effect_id: 372,
    },
];

pub const RETROBUTION_ONI_MOTION: TutorialOniMotionContract = TutorialOniMotionContract {
    initial_horizontal_min: -10.0,
    initial_horizontal_max: 10.0,
    initial_vertical_min: 5.0,
    initial_vertical_max: 10.0,
    initial_move_speed: 1.0,
    acceleration_per_second: 5.0,
    damping_per_second: 2.0,
    gravity_per_second: -9.8,
    arrival_distance: 0.3,
    maximum_distance: 50.0,
    maximum_flight_seconds: 5.0,
    destroy_delay_seconds: 0.5,
    bounce_multiplier: -0.5,
    normal_force: 2.0,
    reverse_force: 4.0,
};
