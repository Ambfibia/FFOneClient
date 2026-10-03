use super::*;

/// WindowResolution carries both OS DPI and the production Scale UI setting.
/// Keep one rendered texel per displayed pixel instead of magnifying a 500x564
/// image on high-resolution windows. The logical UI rect and framing stay fixed.
pub(super) fn sync_inventory_preview_resolution(
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Option<Res<UiScale>>,
    target: Res<NativePlayerInventoryPreviewImage>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = window.scale_factor() * ui_scale.as_deref().map_or(1.0, |scale| scale.0);
    if !scale.is_finite() || scale <= 0.0 {
        return;
    }
    let size = (Vec2::new(
        NATIVE_PLAYER_INVENTORY_PREVIEW_WIDTH as f32,
        NATIVE_PLAYER_INVENTORY_PREVIEW_HEIGHT as f32,
    ) * scale)
        .ceil()
        .max(Vec2::ONE)
        .as_uvec2();
    // Do not dirty the image every frame: that would re-upload/reallocate it.
    if images
        .get(&target.0)
        .is_some_and(|image| image.size() != size)
    {
        let mut image = images.get_mut(&target.0).expect("checked inventory target");
        image.resize(bevy::render::render_resource::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_target_follows_ui_density_without_changing_handle_or_logical_rect() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>()
            .insert_resource(UiScale(1.0))
            .add_systems(Startup, setup_native_player_preview_stage)
            .add_systems(Update, sync_inventory_preview_resolution);
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        app.update();
        let handle = app
            .world()
            .resource::<NativePlayerInventoryPreviewImage>()
            .0
            .clone();

        for (density, expected) in [
            (1.0, UVec2::new(500, 564)),
            (1.5, UVec2::new(750, 846)),
            (2.0, UVec2::new(1000, 1128)),
            (2.5, UVec2::new(1250, 1410)),
            (1.0, UVec2::new(500, 564)),
        ] {
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .resolution
                .set_scale_factor_override(Some(density));
            app.update();
            let target = app.world().resource::<NativePlayerInventoryPreviewImage>();
            assert_eq!(target.0, handle);
            let image = app
                .world()
                .resource::<Assets<Image>>()
                .get(&handle)
                .unwrap();
            assert_eq!(image.size(), expected);
            assert_eq!(image.sampler, bevy::image::ImageSampler::linear());
        }

        app.world_mut().resource_mut::<UiScale>().0 = 1.25;
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<Image>>()
                .get(&handle)
                .unwrap()
                .size(),
            UVec2::new(625, 705),
        );
    }
}
