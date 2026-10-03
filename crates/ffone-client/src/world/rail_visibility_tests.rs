use super::*;

#[test]
fn city_station_rails_keep_real_bounds_when_center_fade_crosses_the_track() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let scene: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("map/tiles/map_04_04/scene.json")).unwrap())
            .unwrap();
    let mut checked = 0;
    for visual in scene["visuals"].as_array().unwrap() {
        let model = scene["models"]
            .as_array()
            .unwrap()
            .iter()
            .find(|model| model["id"] == visual["model"])
            .unwrap();
        let path = model["path"].as_str().unwrap();
        if !path.contains("/dt_mtcs_expressway_union_dt_sdts_monorail_01_dt_highway/")
            && !path.contains("/dt_mtcs_expressway_union_dt_sdts_monorail_variant_0004/")
        {
            continue;
        }
        let bytes = fs::read(root.join(path)).unwrap();
        let gltf = gltf::Gltf::from_slice(&bytes).unwrap();
        let primitive = gltf.meshes().next().unwrap().primitives().next().unwrap();
        let array3 = |key: &str| -> [f32; 3] {
            std::array::from_fn(|i| visual["transform"][key][i].as_f64().unwrap() as f32)
        };
        let rotation = Quat::from_array(std::array::from_fn(|i| {
            visual["transform"]["rotation"][i].as_f64().unwrap() as f32
        }));
        let matrix = Mat4::from_scale_rotation_translation(
            Vec3::from_array(array3("scale")),
            rotation,
            Vec3::from_array(array3("translation")),
        );
        let points: Vec<_> = primitive
            .reader(|_| gltf.blob.as_deref())
            .read_positions()
            .unwrap()
            .map(|p| matrix.transform_point3(Vec3::from_array(p)))
            .collect();
        let minimum = points
            .iter()
            .copied()
            .fold(Vec3::splat(f32::INFINITY), Vec3::min);
        let maximum = points
            .iter()
            .copied()
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
        let center = (minimum + maximum) * 0.5;
        let tag = pack_native_world_object_range(
            center,
            native_world_object_range((maximum - minimum).length() * 0.5),
        )
        .unwrap();
        let (decoded, end) = unpack_native_world_object_range(tag);
        assert!(
            points
                .iter()
                .any(|p| p.distance(decoded) > end - NATIVE_WORLD_OBJECT_FADE_BAND),
            "old fade intersects real rail vertices: {path}"
        );
        assert!(!native_world_bounds_fit_center_range(minimum, maximum, tag));

        let mut app = App::new();
        app.add_systems(
            Update,
            (
                prepare_native_world_object_ranges,
                propagate_native_world_object_ranges_to_late_meshes,
            )
                .chain(),
        );
        let scene_root = app.world_mut().spawn_empty().id();
        let key = NativeWorldObjectRangeGroupKey::Visual(checked);
        let aabb = Aabb::from_min_max(minimum, maximum);
        let entity = app
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                Mesh3d::default(),
                aabb,
                NativeWorldObjectRangeMember {
                    scene_root,
                    group: key.clone(),
                },
            ))
            .id();
        app.world_mut()
            .entity_mut(scene_root)
            .insert(NativeWorldObjectRangeGroups {
                pending: HashMap::from([(
                    key.clone(),
                    NativeWorldPendingRangeGroup {
                        expected_members: 1,
                        members: vec![NativeWorldPreparedRangeMember {
                            visual_root: entity,
                            local_minimum: minimum,
                            local_maximum: maximum,
                            meshes: vec![entity],
                        }],
                    },
                )]),
                ready_to_finalize: VecDeque::from([key.clone()]),
                unbounded: HashSet::new(),
            });
        app.update();
        assert!(
            app.world()
                .get::<NativeWorldObjectRangeReady>(entity)
                .is_some()
        );
        assert!(
            app.world()
                .get::<NativeWorldObjectRangeUnbounded>(entity)
                .is_some()
        );
        assert!(
            app.world()
                .get::<NativeWorldObjectRangeContract>(entity)
                .is_none()
        );
        assert!(app.world().get::<VisibilityRange>(entity).is_none());
        assert_eq!(app.world().get::<Aabb>(entity), Some(&aabb));
        let late = app
            .world_mut()
            .spawn((ChildOf(entity), Mesh3d::default(), aabb, Visibility::Hidden))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(late),
            Some(&Visibility::Inherited)
        );
        assert!(app.world().get::<MeshTag>(late).is_none());
        assert_eq!(app.world().get::<Aabb>(late), Some(&aabb));
        checked += 1;
    }
    assert_eq!(
        checked, 2,
        "both published City Station rail surfaces must be exercised"
    );
}

#[test]
fn center_range_guard_preserves_small_objects_and_accounts_for_quantization() {
    let center = Vec3::new(-2219.88, -0.72, 2322.4);
    for radius in [1.0, 10.0, 100.0, 256.0] {
        let half = Vec3::splat(radius / 3.0_f32.sqrt());
        let tag =
            pack_native_world_object_range(center, native_world_object_range(radius)).unwrap();
        assert!(native_world_bounds_fit_center_range(
            center - half,
            center + half,
            tag
        ));
    }
    let tag = pack_native_world_object_range(center, EXTENDED_WORLD_CAMERA_FAR_NATIVE).unwrap();
    let (decoded, end) = unpack_native_world_object_range(tag);
    let half = Vec3::X * (end - NATIVE_WORLD_OBJECT_FADE_BAND);
    assert!(!native_world_bounds_fit_center_range(
        decoded - half,
        decoded + half,
        tag
    ));
}
