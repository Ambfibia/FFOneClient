use super::*;

#[test]
fn world_and_event_nano_ready_icons_are_exact_primary_publications() {
    let ready_root = workspace_root().join("assets/game/icons/entities/nanos/ready");
    for (slug, path_id, bytes, sha256) in [
        (
            "belladonna",
            2_779_i64,
            3_616_u64,
            "286166eb0abadbfb1f99752e758f3a36fce0ca9fc7c01cc213f605faa6352e08",
        ),
        (
            "computress",
            2_781,
            3_339,
            "bb4e17be5506792fb52010bf086abfa9e880287a8a6e795907be1b811e36076f",
        ),
        (
            "runty",
            2_783,
            3_020,
            "ec227582a54a510c54dcf9bb58e05e62704490e9d71b67f8459598f473520807",
        ),
        (
            "coop",
            3_007,
            4_294,
            "24207e478ae8d7d5f95109d0dced21960cc90e260f8163a21f6a5b6f5e178651",
        ),
        (
            "panini",
            3_199,
            4_153,
            "d6b699f8f19009089d8a0d20afec227e207613f061e01f9b6f3b4e49a708e5fd",
        ),
        (
            "jack-olantern",
            3_341,
            4_851,
            "558811508d5070789f96249b0ad1435984ba586f34e279623b26671dfaaab971",
        ),
    ] {
        let path = ready_root.join(format!("nanoready_{slug}.png"));
        let payload = fs::read(&path)
            .unwrap_or_else(|error| panic!("read primary PathID {path_id}: {error}"));
        assert_eq!(payload.len() as u64, bytes, "primary PathID {path_id}");
        assert_eq!(sha256_lower(&payload), sha256, "primary PathID {path_id}");
    }
}
