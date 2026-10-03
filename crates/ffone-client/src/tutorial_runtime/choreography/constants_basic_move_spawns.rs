use super::*;

pub(super) const FADE_SCREEN_IN: FadeSequence = FadeSequence {
    channel: FadeChannel::CinematicBars,
    from: 0.0,
    to: 1.0,
    steps: 10,
    divisor: 9.0,
    seconds_per_step: 0.1,
};

pub(super) const FADE_SCREEN_OUT: FadeSequence = FadeSequence {
    channel: FadeChannel::CinematicBars,
    from: 1.0,
    to: 0.0,
    steps: 10,
    divisor: 9.0,
    seconds_per_step: 0.1,
};

pub(super) const FADE_OVERLAY_OUT: FadeSequence = FadeSequence {
    channel: FadeChannel::Overlay,
    from: 1.0,
    to: 0.0,
    steps: 10,
    divisor: 9.0,
    seconds_per_step: 0.1,
};

pub(super) const FADE_OVERLAY_IN: FadeSequence = FadeSequence {
    channel: FadeChannel::Overlay,
    from: 0.0,
    to: 1.0,
    steps: 10,
    divisor: 9.0,
    seconds_per_step: 0.1,
};

pub(super) const SHAKE_START_WAVE: CameraShakeWave = CameraShakeWave {
    x_sin_multiplier: 18.0,
    y_cos_multiplier: 9.0,
    z_sin_multiplier: 9.0,
    z_cos_multiplier: 6.0,
};

pub(super) const SHAKE_TARGET_WAVE: CameraShakeWave = CameraShakeWave {
    x_sin_multiplier: 20.0,
    y_cos_multiplier: 6.0,
    z_sin_multiplier: 6.0,
    z_cos_multiplier: 12.0,
};

pub(super) const BASIC_MOVE_SPAWNS: &[NpcSpawn] = &[
    spawn(
        100,
        2669,
        541.0,
        657.0,
        -105.0,
        Some(270),
        Some("run"),
        false,
    ),
    spawn(
        101,
        2670,
        541.0,
        653.0,
        -105.0,
        Some(270),
        Some("run"),
        false,
    ),
    spawn(203, 2666, 883.0, 693.0, -34.0, Some(194), None, false),
    spawn(
        204,
        2667,
        903.0,
        724.0,
        11.0,
        Some(175),
        Some("melee1"),
        false,
    ),
    spawn(205, 2668, 903.0, 723.0, 11.0, Some(1), None, false),
    spawn(305, 2674, 908.0, 745.0, -14.0, Some(127), None, false),
    spawn(314, 2674, 884.0, 694.0, -34.0, Some(59), None, false),
    spawn(315, 2674, 883.0, 694.0, -34.0, Some(347), None, false),
    spawn(316, 2674, 883.0, 709.0, -34.0, Some(24), None, false),
    spawn(317, 2674, 885.0, 692.0, -34.0, Some(86), None, false),
    spawn(321, 2674, 905.0, 727.0, 11.0, Some(7), None, false),
    spawn(322, 2674, 902.0, 726.0, 11.0, Some(324), None, false),
    spawn(323, 2674, 904.0, 726.0, 11.0, Some(1), None, false),
    spawn(324, 2674, 904.0, 719.0, 11.0, Some(191), None, false),
    spawn(325, 2674, 906.0, 719.0, 11.0, Some(125), None, false),
    spawn(326, 2674, 903.0, 716.0, 11.0, Some(180), None, false),
    spawn(200, 2663, 908.0, 745.0, -15.0, Some(354), None, true),
    spawn(201, 2664, 912.0, 742.0, -15.0, Some(500), None, true),
    spawn(202, 2665, 906.0, 747.0, -15.0, None, Some("run"), true),
];

pub(super) const BASIC_MOVE_EXPLOSIONS: &[EffectSpawn] = &[
    effect(767, 907.0, 11.0, 699.0, 1.0, true),
    effect(767, 904.0, 10.0, 722.0, 1.0, true),
    effect(767, 904.0, -8.0, 699.0, 1.0, true),
    effect(767, 884.0, -33.0, 701.0, 1.0, true),
];
