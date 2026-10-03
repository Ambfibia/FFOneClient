use super::*;

pub(super) fn asset_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Gltf>()
        .init_asset::<WorldAsset>()
        .init_asset::<AnimationClip>()
        .init_asset::<AnimationGraph>()
        .init_asset::<LegacyModelMaterial>()
        .add_plugins(TutorialNanoGameplayPlugin);
    app
}

#[test]
fn every_catalog_nano_can_be_summoned_with_each_skill_selection() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(root.clone()).unwrap();
    let catalog =
        crate::gameplay_nano_portraits::GameplayNanoPortraitCatalog::open(&locator).unwrap();
    let mut checked = 0;
    for nano_id in 1..=i16::MAX {
        let Some(model_path) = catalog.model_path(nano_id) else {
            continue;
        };
        let bytes = std::fs::read(root.join(model_path)).unwrap();
        let source = gltf::Gltf::from_slice(&bytes).unwrap();
        let names = source
            .animations()
            .filter_map(|clip| clip.name())
            .collect::<Vec<_>>();
        for skill_slot in 1..=3 {
            let mut app = asset_app();
            let owner = app.world_mut().spawn(Transform::IDENTITY).id();
            {
                let mut queue = app
                    .world_mut()
                    .resource_mut::<TutorialNanoGameplayCommandQueue>();
                queue.equip_world(
                    nano_id,
                    1,
                    150,
                    WorldNanoGameplayPresentation {
                        model_path: model_path.to_owned(),
                        style: 1,
                        skill_slot,
                    },
                );
                queue.summon(owner);
            }
            apply_tutorial_nano_gameplay_commands(app.world_mut());
            let root = app
                .world()
                .resource::<TutorialNanoGameplayState>()
                .entity()
                .unwrap();
            // This test isolates the production animation/reveal gate;
            // texture and skin loading are covered by the GPU fixture.
            app.world_mut()
                .resource_mut::<TutorialNanoGameplayState>()
                .face_texture_bound = true;
            let mut gltf = gltf_with_gameplay_clips();
            gltf.named_animations.clear();
            for name in &names {
                let mut clip = AnimationClip::default();
                clip.set_duration(10.0);
                let handle = app
                    .world_mut()
                    .resource_mut::<Assets<AnimationClip>>()
                    .add(clip);
                gltf.named_animations.insert((*name).into(), handle);
            }
            let handle = app.world_mut().resource_mut::<Assets<Gltf>>().add(gltf);
            app.world_mut()
                .resource_mut::<TutorialNanoGameplayAssets>()
                .gltf = Some(handle);
            app.world_mut()
                .spawn((AnimationPlayer::default(), ChildOf(root)));
            app.update();
            let state = app.world().resource::<TutorialNanoGameplayState>();
            assert!(
                state.is_active(),
                "Nano {nano_id}, slot {skill_slot}: {:?}",
                state.status()
            );
            assert_eq!(state.animation.clip(), Some("call"));
            let world = app.world_mut();
            let mut scenes =
                world.query_filtered::<&Visibility, With<TutorialGameplayNanoScene>>();
            assert!(
                scenes
                    .iter(world)
                    .all(|visibility| *visibility == Visibility::Inherited)
            );
            app.world_mut()
                .resource_mut::<TutorialNanoGameplayCommandQueue>()
                .play_world_skill(owner);
            app.update();
            assert!(
                app.world()
                    .resource::<TutorialNanoGameplayState>()
                    .is_active()
            );
            checked += 1;
        }
    }
    assert_eq!(checked, catalog.len() * 3);
    assert!(
        checked >= 180,
        "the production catalog must not silently shrink"
    );
}
