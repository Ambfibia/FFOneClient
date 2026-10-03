use super::*;
#[test]
fn production_vehicle_trails_have_exact_variants() {
    let locator = AssetLocator::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
    )
    .unwrap();
    let definitions = read_catalog(&locator).unwrap();
    assert_eq!(definitions.len(), 3);
    for (d, _) in definitions {
        assert_eq!(d.length, 16);
        assert_eq!(d.interpolation_step, 1.5);
    }
    assert_eq!(style("items/vehicle_hoverboard/models/model.glb"), "board");
    assert_eq!(style("items/vehicle_hoverboard2/models/model.glb"), "cloud");
    assert_eq!(style("items/vehicle_cloud/models/model.glb"), "cloud");
    assert_eq!(style("items/vehicle_jetbike/models/model.glb"), "standard");
}
