use super::*;

#[derive(Debug)]
pub(super) struct ParticleEvent {
    pub clip: String,
    pub time: f32,
    pub effect_id: i32,
}

// Use each native model's event time and declared effect, including Nanos
// without a particle event. Animation and audio variants stay model-owned.
pub(super) fn particle_events(bytes: &[u8]) -> Result<Vec<ParticleEvent>, String> {
    let document = crate::network_world_runtime::parse_network_npc_animation_document(bytes)?;
    let mut parsed = Vec::new();
    for animation in document["animations"].as_array().into_iter().flatten() {
        let Some(clip @ ("win" | "lose" | "tie")) = animation["name"].as_str() else {
            continue;
        };
        for event in animation
            .pointer("/extras/nonTrs/events")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            if event["functionName"].as_str() != Some("particle") {
                continue;
            }
            let effect_id = event["stringParameter"]
                .as_str()
                .and_then(|id| id.parse::<i32>().ok())
                .filter(|id| *id > 0)
                .ok_or_else(|| format!("Nano {clip} particle has an invalid effect ID"))?;
            let time = event["time"]
                .as_f64()
                .filter(|t| t.is_finite() && *t >= 0.0 && *t <= f64::from(f32::MAX))
                .ok_or_else(|| format!("Nano {clip} particle has an invalid time"))?
                as f32;
            parsed.push(ParticleEvent {
                clip: clip.into(),
                time,
                effect_id,
            });
        }
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_buttercup_reactions_keep_their_particle_ids_and_audio_events() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
        let bytes =
            std::fs::read(root.join("characters/nanos/nano_buttercup/nano_buttercup.glb")).unwrap();
        let particles = particle_events(&bytes).unwrap();
        let sounds = parse_network_npc_animation_sound_events(&bytes).unwrap();
        for (clip, id, voice) in [
            ("win", 535, "NanCorrWin"),
            ("lose", 536, "NanCorrLose"),
            ("tie", 521, "NanCorrTie"),
        ] {
            let particle = particles.iter().find(|e| e.clip == clip).unwrap();
            assert_eq!(particle.effect_id, id);
            assert_eq!(particle.time, 0.25);
            assert!(
                sounds
                    .iter()
                    .any(|e| e.clip == clip && e.payload.contains(voice))
            );
        }
    }
}
