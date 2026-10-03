use super::*;
#[test]
fn camera_bounds_use_posed_skin_without_double_transform_or_unused_vertices() {
    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0.5, 2.0, 3.0], [1.0, 3.0, 4.0], [10000.0; 3]],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(vec![[0, 1, 0, 0]; 3]),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_WEIGHT,
        vec![[0.25, 0.75, 0.0, 0.0]; 3],
    );
    mesh.insert_indices(bevy::mesh::Indices::U32(vec![0, 1, 0]));
    let misleading_node = GlobalTransform::from(
        Transform::from_xyz(-50.0, 20.0, 0.0).with_scale(Vec3::splat(100.0)),
    );
    let matrices = [
        Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)),
        Mat4::from_translation(Vec3::new(-2.0, 4.0, 0.0)),
    ];
    let (min, max) = posed_mesh_bounds(&mesh, &misleading_node, Some(&matrices)).unwrap();
    assert!(min.abs_diff_eq(Vec3::new(-0.5, 5.0, 3.0), 1e-6));
    assert!(max.abs_diff_eq(Vec3::new(0.0, 6.0, 4.0), 1e-6));
    let (rigid_min, _) = posed_mesh_bounds(&mesh, &misleading_node, None).unwrap();
    assert!(rigid_min.abs_diff_eq(
        misleading_node.transform_point(Vec3::new(0.5, 2.0, 3.0)),
        1e-6
    ));
    assert!(posed_mesh_bounds(&mesh, &misleading_node, Some(&matrices[..1])).is_none());
}
#[test]
fn fit_contains_tall_and_wide_models_at_both_window_sizes() {
    for size in [Vec2::new(1600.0, 940.0), Vec2::new(1180.0, 720.0)] {
        let (_, viewport) = preview_viewport_logical_rect(size).unwrap();
        let aspect = viewport.x / viewport.y;
        for extent in [Vec3::new(1.0, 8.0, 1.0), Vec3::new(14.0, 2.0, 3.0)] {
            let distance = fit_distance(extent, aspect);
            let half_vertical = 22.5_f32.to_radians();
            let half_horizontal = (half_vertical.tan() * aspect).atan();
            let angular_radius = (extent.length() * 0.5 / distance).asin();
            assert!(angular_radius < half_vertical.min(half_horizontal));
        }
    }
}
#[test]
fn production_equipment_resolves_both_genders_and_keeps_other_slots() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let mut catalog = EditorCatalog::open(&locator).unwrap();
    let library = EquipmentLibrary::open(&root, &mut catalog).unwrap();
    assert!(library.entries.len() > 1000);
    for female in [false, true] {
        let mut outfit = library.outfits[usize::from(female)];
        for category in CATEGORIES {
            let item = library
                .entries
                .values()
                .find(|item| {
                    item.category == category
                        && (item.required_gender == 0
                            || item.required_gender == if female { 2 } else { 1 })
                })
                .unwrap();
            outfit[slot(category)].item_id = i16::try_from(item.item_number).unwrap();
            let look = library.resolve(female, outfit).unwrap();
            look.validate().unwrap();
            assert_eq!(
                look.gender,
                if female {
                    ffone_runtime_contracts::PlayerRigGender::Female
                } else {
                    ffone_runtime_contracts::PlayerRigGender::Male
                }
            );
            assert!(look.parts.len() >= 5);
        }
    }
}

#[test]
fn icon_equipment_contains_only_selected_part_for_every_category_and_gender() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&root).unwrap();
    let mut catalog = EditorCatalog::open(&locator).unwrap();
    let library = EquipmentLibrary::open(&root, &mut catalog).unwrap();
    for female in [false, true] {
        for category in CATEGORIES {
            let item = library.entries.values().find(|item| item.category == category
                && (item.required_gender == 0 || item.required_gender == if female { 2 } else { 1 })).unwrap();
            let mut outfit = library.outfits[usize::from(female)];
            outfit[slot(category)].item_id = i16::try_from(item.item_number).unwrap();
            let mut look = library.resolve(female, outfit).unwrap();
            isolate_item(&mut look, category).unwrap();
            assert_eq!(look.parts.len(), 1, "{category:?}, female={female}");
            assert!(look.weapon_animation_profile.is_none());
            look.validate().unwrap();
        }
    }
}
