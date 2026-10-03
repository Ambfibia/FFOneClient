use super::*;

#[test]
fn past_and_future_sector_v_publish_six_budget_protected_hologram_bodies() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let expected = [
        ("map_07_08", "buttercup_hologram"),
        ("map_12_03", "dexter_hologram"),
        ("map_12_03", "NO1_hologram"),
        ("map_12_03", "NO2_hologram"),
        ("map_12_03", "SJ_hologram"),
        ("map_12_03", "ben_hologram"),
    ];
    for (tile, effect_name) in expected {
        let path = root.join(format!("map/tiles/{tile}/behaviour.json"));
        let document: JsonValue =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let record = document["effectEmitters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["effectName"].as_str() == Some(effect_name))
            .unwrap_or_else(|| panic!("{tile} has no {effect_name}"));
        let mut persistent_bodies = 0;
        for closure_id in record["resolvedParticlePrefabs"].as_array().unwrap() {
            let Some(closure_id) = closure_id.as_str() else {
                continue;
            };
            let closure = document["effectPrefabClosures"]
                .as_array()
                .unwrap()
                .iter()
                .find(|closure| closure["id"].as_str() == Some(closure_id))
                .unwrap();
            persistent_bodies += closure["objects"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|object| {
                    let value = &object["value"];
                    value["lifeTime"].as_f64().is_some_and(|life| life >= 300.0)
                        && value["generationsPerSecond"]
                            .as_f64()
                            .is_some_and(|rate| rate <= 1.0 / 300.0)
                        && value["numberPerGeneration"].as_f64() == Some(1.0)
                })
                .count();
        }
        assert_eq!(persistent_bodies, 1, "{tile} {effect_name}");
    }
}
