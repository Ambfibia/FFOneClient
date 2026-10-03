use super::*;

#[test]
fn directional_turn_layer_matches_the_exact_eight_way_source_table_and_tick() {
    let expected = [
        (0, (0.0, 0.0)),
        (1, (0.0, 0.0)),
        (2, (0.0, 0.5)),
        (3, (0.0, 1.0)),
        (4, (0.5, 0.0)),
        (5, (0.0, 0.0)),
        (6, (0.0, 0.5)),
        (7, (1.0, 0.0)),
        (8, (0.5, 0.0)),
    ];
    for (direction, target) in expected {
        assert_eq!(legacy_directional_turn_target(direction), target);
    }
    assert_eq!(move_towards(0.0, 1.0, 0.2), 0.2);
    assert_eq!(move_towards(0.8, 1.0, 0.3), 1.0);
    assert_eq!(move_towards(1.0, 0.0, 0.2), 0.8);
    assert_eq!(move_towards(0.1, 0.0, 0.3), 0.0);
}
