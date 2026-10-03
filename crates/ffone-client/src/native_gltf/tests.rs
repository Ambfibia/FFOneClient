use super::*;
use bevy::{
    animation::AnimationPlugin, gltf::GltfPlugin, image::ImagePlugin, mesh::MeshPlugin,
    world_serialization::WorldSerializationPlugin,
};
use std::time::{Duration, Instant};

#[test]
fn scene_name_cannot_shadow_the_authored_skeleton_root() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("rig.gltf"),
        r#"{
        "asset":{"version":"2.0"}, "scene":0,
        "scenes":[{"name":"m","nodes":[0]}],
        "nodes":[{"name":"m","children":[1]}, {"name":"Bip01"}]
    }"#,
    )
    .unwrap();
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        AssetPlugin {
            file_path: directory.path().to_string_lossy().into_owned(),
            ..default()
        },
        WorldSerializationPlugin,
        ImagePlugin::default(),
        MeshPlugin,
        AnimationPlugin,
        GltfPlugin::default(),
        NativeGltfPlugin,
    ));
    app.finish();
    app.cleanup();
    let handle: Handle<WorldAsset> = app
        .world()
        .resource::<AssetServer>()
        .load("rig.gltf#Scene0");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !app
        .world()
        .resource::<Assets<WorldAsset>>()
        .contains(&handle)
    {
        assert!(Instant::now() < deadline, "synthetic rig did not load");
        app.update();
        std::thread::sleep(Duration::from_millis(2));
    }
    let mut assets = app.world_mut().resource_mut::<Assets<WorldAsset>>();
    let scene = assets.get_mut_untracked(&handle).unwrap();
    let names = scene
        .world
        .query::<(Entity, &Name)>()
        .iter(&scene.world)
        .map(|(entity, name)| (entity, name.as_str().to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 2);
    let root = names.iter().find(|(_, name)| name == "m").unwrap().0;
    let bone = names.iter().find(|(_, name)| name == "Bip01").unwrap().0;
    assert_eq!(scene.world.get::<ChildOf>(bone).unwrap().parent(), root);
    let wrapper = scene.world.get::<ChildOf>(root).unwrap().parent();
    assert!(scene.world.get::<Name>(wrapper).is_none());
    assert_eq!(
        scene
            .world
            .get::<bevy::gltf::GltfSceneName>(wrapper)
            .unwrap()
            .0,
        "m"
    );
}
