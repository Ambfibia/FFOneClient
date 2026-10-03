use super::*;

fn grid(size: usize) -> AuthoredTriMeshCollider {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for z in 0..size {
        for x in 0..size {
            let x = x as f32;
            let z = z as f32;
            let h = (x * 0.3).sin() + (z * 0.2).cos();
            let first = vertices.len() as u32;
            vertices.extend([
                Vec3::new(x, h, z),
                Vec3::new(x, h, z + 1.0),
                Vec3::new(x + 1.0, h, z),
                Vec3::new(x + 1.0, h, z + 1.0),
            ]);
            indices.extend([first, first + 1, first + 2, first + 2, first + 1, first + 3]);
        }
    }
    AuthoredTriMeshCollider {
        source_mesh: default(),
        source_model_path: "test".into(),
        vertices: vertices.into(),
        indices: indices.into(),
        local_min: Vec3::new(0.0, -2.0, 0.0),
        local_max: Vec3::new(size as f32, 2.0, size as f32),
        is_trigger: false,
    }
}

fn next(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    (*seed >> 8) as f32 / 16777216.0
}

fn check(collider: &AuthoredTriMeshCollider, matrix: Mat4) {
    let (minimum, maximum) = collider_world_bounds(collider, matrix).unwrap();
    let bounds = AuthoredColliderWorldBounds { minimum, maximum };
    let mut cache = GroundTriangleCache::default();
    cache.enabled = true;
    let entity = Entity::from_bits(1);
    let mut points = Vec::new();
    let mut seed = 42;
    for _ in 0..600 {
        points.push(Vec2::new(
            minimum.x + (maximum.x - minimum.x) * next(&mut seed),
            minimum.z + (maximum.z - minimum.z) * next(&mut seed),
        ));
    }
    let prepared = PreparedGroundCollider::new(collider, matrix, &bounds);
    for cell in &prepared.cells {
        for p in [
            cell.minimum,
            cell.maximum,
            (cell.minimum + cell.maximum) * 0.5,
        ] {
            points.extend([
                p,
                Vec2::new(p.x.next_down(), p.y.next_up()),
                Vec2::new(p.x.next_up(), p.y.next_down()),
            ]);
        }
    }
    for vertex in collider.vertices.iter().step_by(19) {
        let p = matrix.transform_point3(*vertex);
        points.extend([
            Vec2::new(p.x, p.z),
            Vec2::new(p.x.next_up(), p.z.next_down()),
        ]);
    }
    points.extend([
        Vec2::splat(f32::NAN),
        Vec2::splat(f32::INFINITY),
        Vec2::new(minimum.x - 0.01, minimum.z),
        Vec2::new(maximum.x + 0.01, maximum.z),
    ]);
    assert!(prepared.ready);
    cache.entries.insert(entity, prepared);
    for point in points {
        for (low, high) in [
            (-1000000.0, 1000000.0),
            (minimum.y, minimum.y + 0.5),
            (maximum.y - 0.5, maximum.y),
        ] {
            let expected = collider_ground_height_with_bounds(
                collider, matrix, &bounds, point.x, point.y, low, high,
            );
            let actual = cache.height(
                entity, collider, matrix, &bounds, point.x, point.y, low, high,
            );
            assert_eq!(
                actual.map(f32::to_bits),
                expected.map(f32::to_bits),
                "point={point:?}, matrix={matrix:?}"
            );
        }
    }
}

#[test]
fn indexed_ground_matches_original_at_edges_and_transformed_geometry() {
    let collider = grid(20);
    for matrix in [
        Mat4::IDENTITY,
        Mat4::from_scale_rotation_translation(
            Vec3::new(1.2, 0.8, 0.6),
            Quat::from_euler(EulerRot::XYZ, 0.2, 0.47, -0.15),
            Vec3::new(-3898.0, -56.0, 4475.0),
        ),
        Mat4::from_scale(Vec3::new(-1.0, 2.0, 0.7)),
        Mat4::from_cols(
            Vec4::new(1.0, 0.1, 0.3, 0.0),
            Vec4::new(0.2, 1.0, 0.1, 0.0),
            Vec4::new(0.4, 0.2, 1.0, 0.0),
            Vec4::W,
        ),
        Mat4::from_scale(Vec3::new(0.0, 1.0, 1.0)),
        Mat4::from_scale_rotation_translation(Vec3::splat(0.0001), Quat::IDENTITY, Vec3::ZERO),
        Mat4::from_translation(Vec3::new(10000000.0, 0.0, -10000000.0)),
    ] {
        check(&collider, matrix);
    }
    let bounds = AuthoredColliderWorldBounds {
        minimum: collider.local_min,
        maximum: collider.local_max,
    };
    let prepared = PreparedGroundCollider::new(&collider, Mat4::IDENTITY, &bounds);
    assert!(prepared.candidates(5.1, 6.2).unwrap().len() < prepared.triangles.len() / 4);
}

#[test]
fn interval_rejection_never_discards_an_accepted_barycentric_sample() {
    let mut seed = 728;
    for scale in [0.00001, 0.1, 1.0, 1000.0, 10000000.0] {
        for _ in 0..2000 {
            let mut point = || {
                Vec3::new(
                    (next(&mut seed) - 0.5) * scale,
                    next(&mut seed),
                    (next(&mut seed) - 0.5) * scale,
                )
            };
            let triangle = PreparedGroundTriangle {
                a: point(),
                b: point(),
                c: point(),
            };
            for p in [
                triangle.a,
                triangle.b,
                triangle.c,
                (triangle.a + triangle.b + triangle.c) / 3.0,
            ] {
                for x in [p.x, p.x.next_down(), p.x.next_up()] {
                    if triangle_height_at_xz(triangle.a, triangle.b, triangle.c, x, p.z).is_some() {
                        let minimum = Vec2::new(x.next_down(), p.z.next_down());
                        let maximum = Vec2::new(x.next_up(), p.z.next_up());
                        assert!(triangle.may_hit_cell(minimum, maximum));
                    }
                }
            }
        }
    }
}

#[test]
fn ground_index_refreshes_motion_geometry_and_releases_retired_entries() {
    let mut collider = grid(12);
    let mut cache = GroundTriangleCache::default();
    cache.enabled = true;
    let entity = Entity::from_bits(2);
    let compare = |cache: &mut GroundTriangleCache,
                   collider: &AuthoredTriMeshCollider,
                   matrix: Mat4| {
        let (minimum, maximum) = collider_world_bounds(collider, matrix).unwrap();
        let bounds = AuthoredColliderWorldBounds { minimum, maximum };
        let expected =
            collider_ground_height_with_bounds(collider, matrix, &bounds, 4.3, 4.7, -100.0, 100.0);
        let actual = cache.height(entity, collider, matrix, &bounds, 4.3, 4.7, -100.0, 100.0);
        assert_eq!(actual.map(f32::to_bits), expected.map(f32::to_bits));
    };
    compare(&mut cache, &collider, Mat4::IDENTITY);
    compare(&mut cache, &collider, Mat4::from_translation(Vec3::Y * 3.0));
    for vertex in Arc::make_mut(&mut collider.vertices) {
        vertex.y += 2.0;
    }
    compare(&mut cache, &collider, Mat4::IDENTITY);
    Arc::make_mut(&mut collider.indices).reverse();
    compare(&mut cache, &collider, Mat4::IDENTITY);
    cache.begin_frame(|_| true);
    assert_eq!(cache.entries.len(), 1);
    cache.begin_frame(|_| true);
    assert!(cache.entries.is_empty());
    compare(&mut cache, &collider, Mat4::IDENTITY);
    cache.begin_frame(|_| false);
    assert!(cache.entries.is_empty());
    assert_eq!(Arc::strong_count(&collider.vertices), 1);
}

#[test]
#[ignore = "opt-in CPU benchmark; compare exact checksums, not timing thresholds"]
fn ground_triangle_query_benchmark() {
    let collider = grid(64);
    let bounds = AuthoredColliderWorldBounds {
        minimum: collider.local_min,
        maximum: collider.local_max,
    };
    let start = std::time::Instant::now();
    let prepared = PreparedGroundCollider::new(&collider, Mat4::IDENTITY, &bounds);
    let build_ms = start.elapsed().as_secs_f64() * 1000.0;
    let mut sums = Vec::new();
    for indexed in [false, true] {
        let start = std::time::Instant::now();
        let mut sum = 0.0_f64;
        for i in 0..4000 {
            let x = (i % 61) as f32 + 0.3;
            let z = (i % 59) as f32 + 0.7;
            let height = if indexed {
                prepared.height(x, z, -100.0, 100.0)
            } else {
                collider_ground_height_with_bounds(
                    &collider,
                    Mat4::IDENTITY,
                    &bounds,
                    x,
                    z,
                    -100.0,
                    100.0,
                )
            };
            sum += f64::from(std::hint::black_box(height.unwrap()));
        }
        sums.push(sum);
        println!(
            "ground index={indexed} queries=4000 triangles=8192 elapsed_ms={} build_ms={build_ms} checksum={sum}",
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
    assert_eq!(sums[0], sums[1]);
}

#[test]
fn preparation_is_bounded_and_incomplete_queries_use_the_original_result() {
    let collider = grid(24);
    let bounds = AuthoredColliderWorldBounds {
        minimum: collider.local_min,
        maximum: collider.local_max,
    };
    let mut cache = GroundTriangleCache::default();
    cache.enabled = true;
    let entity = Entity::from_bits(3);
    let expected = collider_ground_height_with_bounds(
        &collider,
        Mat4::IDENTITY,
        &bounds,
        5.3,
        7.2,
        -100.0,
        100.0,
    );
    let query = |cache: &mut GroundTriangleCache| {
        cache.height(
            entity,
            &collider,
            Mat4::IDENTITY,
            &bounds,
            5.3,
            7.2,
            -100.0,
            100.0,
        )
    };
    assert_eq!(query(&mut cache), expected);
    assert!(!cache.entries[&entity].ready);
    assert_eq!(cache.entries[&entity].triangle_cursor, 512);
    assert_eq!(query(&mut cache), expected);
    assert_eq!(cache.entries[&entity].triangle_cursor, 512);
    let mut completed = false;
    for _ in 0..32 {
        cache.begin_frame(|_| true);
        assert_eq!(
            query(&mut cache).map(f32::to_bits),
            expected.map(f32::to_bits)
        );
        if cache.entries[&entity].ready {
            completed = true;
            break;
        }
    }
    assert!(completed);
    assert!(
        cache.entries[&entity].candidates(5.3, 7.2).unwrap().len()
            < cache.entries[&entity].triangles.len() / 4
    );
}
