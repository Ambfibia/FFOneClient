use super::validate_primitive_normal_orientation;

const POSITIONS: [[f64; 3]; 3] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
const NORMALS: [[f64; 3]; 3] = [[0.0, 0.0, 1.0]; 3];

#[test]
fn accepts_ccw_fronts_aligned_with_authored_normals() {
    validate_primitive_normal_orientation("fixture", &POSITIONS, &NORMALS, &[0, 1, 2])
        .expect("CCW triangle");
}

#[test]
fn rejects_the_historical_second_winding_swap() {
    let error =
        validate_primitive_normal_orientation("fixture", &POSITIONS, &NORMALS, &[0, 2, 1])
            .expect_err("opposed winding must fail");
    assert!(
        error
            .to_string()
            .contains("winding opposes authored normals")
    );
}
