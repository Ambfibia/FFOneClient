
/// Exact native publication routes for the six player-weapon BulletTable
/// success cues. The original bundles split melee impacts into Combat while
/// laser and sonic impacts live in Shared; deriving one common folder from
/// the cue name silently drops both melee sounds.
pub(super) fn exact_tutorial_success_sound_path(success_sound: &str) -> Option<&'static str> {
    match success_sound {
        "ElectricTarget-01" => Some("audio/sfx/combat/electrictarget_01.ogg"),
        "ElectricTarget-03" => Some("audio/sfx/combat/electrictarget_03.ogg"),
        "ElectricTarget-04" => Some("audio/sfx/combat/electrictarget_04.ogg"),
        "ExplosHvyTarget-01" => Some("audio/sfx/combat/exploshvytarget_01.ogg"),
        "ExplosHvyTarget-02" => Some("audio/sfx/combat/exploshvytarget_02.ogg"),
        "ExplosHvyTarget-03" => Some("audio/sfx/combat/exploshvytarget_03.ogg"),
        "ExplosHvyTarget-04" => Some("audio/sfx/combat/exploshvytarget_04.ogg"),
        "ExplosHvyTarget-05" => Some("audio/sfx/combat/exploshvytarget_05.ogg"),
        "ExplosLtTarget-01" => Some("audio/sfx/combat/exploslttarget_01.ogg"),
        "ExplosSlimeTarget-02" => Some("audio/sfx/combat/explosslimetarget_02.ogg"),
        "FireTarget-04" => Some("audio/sfx/combat/firetarget_04.ogg"),
        "FireTarget-05" => Some("audio/sfx/combat/firetarget_05.ogg"),
        "IceTarget-01" => Some("audio/sfx/combat/icetarget_01.ogg"),
        "LaserHvyTarget-01" => Some("audio/sfx/combat/laserhvytarget_01.ogg"),
        "LaserHvyTarget-02" => Some("audio/sfx/combat/laserhvytarget_02.ogg"),
        "LaserHvyTarget-05" => Some("audio/sfx/combat/laserhvytarget_05.ogg"),
        "LaserLtTarget-01" => Some("audio/sfx/combat/laserlttarget_01.ogg"),
        "LaserLtTarget-04" => Some("audio/sfx/combat/laserlttarget_04.ogg"),
        "MeleeMedTarget-01" => Some("audio/sfx/combat/meleemedtarget_01.ogg"),
        "MeleeMedTarget-03" => Some("audio/sfx/combat/meleemedtarget_03.ogg"),
        "SonicTarget-01" => Some("audio/sfx/combat/sonictarget_01.ogg"),
        "SonicTarget-02" => Some("audio/sfx/combat/sonictarget_02.ogg"),
        "SonicTarget-03" => Some("audio/sfx/combat/sonictarget_03.ogg"),
        _ => None,
    }
}
