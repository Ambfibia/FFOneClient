use super::*;

#[test]
fn merged_collision_icons_use_unique_named_exact_publications() {
    let nano_root = workspace_root().join("assets/game/icons/entities/nanos");
    for (slug, path_id, bytes, sha256) in [
        (
            "johnny-test",
            387_616_601_i64,
            4_130_u64,
            "7a90477a9fc37bcbbe0d000c8aea49cf62d3574539fbd124072386d2162a160a",
        ),
        (
            "ice-king",
            2_412_523_389,
            4_599,
            "4d3345a0ae3316ae234c7ed3425d80451522e58daaebcc851d4c04b55979baa4",
        ),
        (
            "rigby",
            198_866_308,
            4_899,
            "7989fe90aba76c6f972cf96ab616a7dd8f819794f1fcd7ab6dcc85ab5f15ab0a",
        ),
        (
            "ampfibian",
            2_514_490_651,
            5_515,
            "4b1553b96573bd811e1ef2dab6fa2e374d6fc7744721fc9e5440149dbbea4640",
        ),
    ] {
        let path = nano_root.join(format!("nanoicon_{slug}.png"));
        let payload = fs::read(&path)
            .unwrap_or_else(|error| panic!("read Academy PathID {path_id}: {error}"));
        assert_eq!(payload.len() as u64, bytes, "Academy PathID {path_id}");
        assert_eq!(sha256_lower(&payload), sha256, "Academy PathID {path_id}");
    }
}
