use crate::legacy_environment::*;
use bevy::{asset::RenderAssetUsages, render::render_resource::PrimitiveTopology};

#[test]
fn original_attribute_mask_rejects_scripted_values_and_requires_contact() {
    assert_eq!(
        legacy_terrain_environment_flags(Some(0x08), true),
        LegacyTerrainEnvironmentFlags {
            poisoned: true,
            healing: false,
        }
    );
    assert_eq!(
        legacy_terrain_environment_flags(Some(0x10), true),
        LegacyTerrainEnvironmentFlags {
            poisoned: false,
            healing: true,
        }
    );
    assert_eq!(
        legacy_terrain_environment_flags(Some(0x0c), false),
        LegacyTerrainEnvironmentFlags::default()
    );
    assert_eq!(
        legacy_terrain_environment_flags(Some(65), true),
        LegacyTerrainEnvironmentFlags::default()
    );
}

#[test]
fn water_material_is_contact_metadata_not_independent_poison_authority() {
    assert_eq!(
        legacy_environment_flags(None, true, false),
        LegacyTerrainEnvironmentFlags::default()
    );
    assert_eq!(
        legacy_environment_flags(Some(0x10), true, false),
        LegacyTerrainEnvironmentFlags {
            poisoned: false,
            healing: true,
        }
    );
    assert_eq!(
        legacy_environment_flags(Some(0x08), true, false),
        LegacyTerrainEnvironmentFlags {
            poisoned: true,
            healing: false,
        }
    );
    assert_eq!(
        legacy_environment_flags(Some(0x08), false, false),
        LegacyTerrainEnvironmentFlags::default()
    );
    assert_eq!(
        legacy_environment_flags(Some(0x18), true, true),
        LegacyTerrainEnvironmentFlags::default(),
        "the clean client suppresses poison and heal while mounted"
    );
}

#[test]
fn water_contact_accepts_initial_submersion_without_sticky_prior_state() {
    assert!(legacy_water_surface_contact(10.0, 10.49));
    assert!(legacy_water_surface_contact(10.0, 9.0));
    assert!(!legacy_water_surface_contact(10.0, 10.5));
    assert!(!legacy_water_surface_contact(10.0, 7.5));
    assert!(!legacy_water_surface_contact(f32::NAN, 9.0));
    assert!(!legacy_water_surface_contact(10.0, f32::INFINITY));
}

#[test]
fn water_height_uses_authored_triangle_surface() {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-1.0, 2.0, -1.0],
            [1.0, 2.0, -1.0],
            [1.0, 2.0, 1.0],
            [-1.0, 2.0, -1.0],
            [1.0, 2.0, 1.0],
            [-1.0, 2.0, 1.0],
        ],
    );
    let global = GlobalTransform::from(Transform::from_xyz(10.0, 3.0, 20.0));
    assert_eq!(
        legacy_water_surface_height(&mesh, &global, 10.0, 20.0),
        Some(5.0)
    );
    assert_eq!(
        legacy_water_surface_height(&mesh, &global, 20.0, 20.0),
        None
    );
}
