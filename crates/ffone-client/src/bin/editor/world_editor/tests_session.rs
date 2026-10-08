use super::*;

#[test]
fn opening_ignores_old_workspace_and_reads_current_server_files() {
    let (_dir, mut e) = fixture();
    let folder = e.folder.clone().unwrap();
    let original = e.entities[0].clone();
    e.transform(
        0,
        original.position + Vec3::X * 30.,
        180.,
        original.instance,
    )
    .unwrap();
    e.save_work().unwrap();
    let backup = fs::read(e.work_path()).unwrap();
    let mut disk = e.sources[0].base.clone();
    disk["NPCs"]["7"]["iX"] = json!(999);
    model::write_atomic(&folder.join("NPCs.json"), &disk).unwrap();

    let mut opened = WorldEditor::open(e.root.clone());
    assert!(opened.enter_session(Some(false), Some(folder)));
    assert_eq!(
        opened.entities[0].position,
        model::native(&disk["NPCs"]["7"]).unwrap()
    );
    assert_eq!(opened.entities[0].angle, original.angle);
    assert!(opened.sources.iter().all(|s| s.base == s.draft));
    assert_eq!(fs::read(opened.work_path()).unwrap(), backup);
}

#[test]
fn returning_from_3d_clears_picker_and_interaction_state_and_keeps_2d_editable() {
    let (_dir, mut e) = fixture();
    e.enter_session(Some(false), None);
    let original = e.entities[0].clone();
    e.transform(
        0,
        original.position + Vec3::X * 10.,
        180.,
        original.instance,
    )
    .unwrap();
    assert!(e.enter_session(Some(true), None));
    e.map_picker = true;
    e.open_type_picker(false, &empty_catalog()).unwrap();
    e.focus = Some(Field::X);
    e.placing = true;
    e.terrain_tool = Some(2);
    let revision = e.revision;
    assert!(e.enter_session(Some(false), None));
    assert!(!e.map_picker);
    assert!(e.type_picker.is_none());
    assert_eq!(e.focus, None);
    assert!(!e.placing);
    assert_eq!(e.terrain_tool, None);
    assert!(e.revision > revision);
    assert_eq!(e.entities[0].position, original.position);
    assert_eq!(e.entities[0].angle, original.angle);
    assert!(e.undo.is_empty());
    let point = Vec3::new(-5., 3., 6.);
    e.select_coordinates(Some(point)).unwrap();
    assert_eq!(e.selected_point, Some(point));
    e.select_entity(0);
    e.transform(0, point, original.angle, original.instance)
        .unwrap();
    assert_eq!(e.selected().unwrap().position, point);
    assert!(!e.work_path().exists());
}

#[test]
fn reselecting_tab_and_reopening_after_other_screen_reads_fresh_server_and_tile_data() {
    let (_dir, mut e) = fixture();
    map_fixture(&e, [0, 0]);
    model::write_atomic(&e.root.join("map/catalog.json"), &json!({})).unwrap();
    e.enter_session(Some(false), None);
    e.center = atlas::tile_center([0, 0], 3.);
    e.load_tile().unwrap();
    let object = e.entities.iter().position(|p| p.kind == 4).unwrap();
    let p = e.entities[object].clone();
    e.select_entity(object);
    e.transform(object, p.position + Vec3::X * 20., 90., 0)
        .unwrap();
    let objects_path = e.sources[p.source].path.clone();
    let mut disk = e.sources[p.source].base.clone();
    disk["objects"][0]["worldMatrix"][0][3] = json!(-14);
    model::write_atomic(&objects_path, &disk).unwrap();
    e.request_open();
    assert!(e.enter_session(Some(false), None));
    assert!(e.maps.is_empty());
    e.load_tile().unwrap();
    assert_eq!(
        e.entities.iter().find(|p| p.kind == 4).unwrap().position.x,
        -14.
    );
    assert!(e.sources.iter().all(|s| s.base == s.draft));

    let original = e.entities[0].clone();
    e.transform(0, original.position, 180., original.instance)
        .unwrap();
    e.enter_session(None, None);
    assert!(e.enter_session(Some(false), None));
    assert_eq!(e.entities[0].angle, original.angle);
    assert!(!e.work_path().exists());
}

#[test]
fn manual_backup_restores_only_on_request_and_uncommitted_edits_do_not_replace_it() {
    let (_dir, mut e) = fixture();
    e.enter_session(Some(false), None);
    let original = e.entities[0].clone();
    e.transform(0, original.position, 135., original.instance)
        .unwrap();
    e.save_work().unwrap();
    let backup = fs::read(e.work_path()).unwrap();
    e.transform(0, original.position, 180., original.instance)
        .unwrap();
    e.enter_session(None, None);
    e.enter_session(Some(false), None);
    assert_eq!(e.entities[0].angle, original.angle);
    assert_eq!(fs::read(e.work_path()).unwrap(), backup);
    e.session_request = Some(super::super::session::Request::Restore);
    assert!(e.enter_session(Some(false), None));
    assert_eq!(e.entities[0].angle, 135.);
    assert!(!e.map_picker);
    assert!(e.sources.iter().any(|s| s.base != s.draft));
}

#[test]
fn rewrite_survives_reopening_but_subsequent_unpublished_edit_is_discarded() {
    let (_dir, mut e) = fixture();
    e.enter_session(Some(false), None);
    let p = e.entities[0].clone();
    e.transform(0, p.position, 135., p.instance).unwrap();
    e.publish().unwrap();
    e.transform(0, p.position, 180., p.instance).unwrap();
    e.request_open();
    e.enter_session(Some(false), None);
    assert_eq!(e.entities[0].angle, 135.);
    assert!(e.sources.iter().all(|s| s.base == s.draft));
}

#[test]
fn invalid_backup_does_not_replace_current_world() {
    let (_dir, mut e) = fixture();
    let original = e.entities[0].clone();
    model::write_atomic(&e.work_path(), &json!({"schema":"broken"})).unwrap();
    assert!(e.restore_work().is_err());
    assert_eq!(e.entities[0].position, original.position);
    assert_eq!(e.entities[0].angle, original.angle);
}

#[test]
fn copied_coordinates_and_entity_remain_available_after_switching_views() {
    let (_dir, mut e) = fixture();
    e.enter_session(Some(false), None);
    let original = e.entities[0].clone();
    let point = original.position + Vec3::new(-6., 7., 8.);
    e.transform(0, point, 135., original.instance).unwrap();
    let point = e.selected().unwrap().position;
    e.copy_coordinates().unwrap();
    e.copy().unwrap();
    let copied = e.clipboard.as_ref().unwrap().row.clone();
    e.enter_session(Some(true), None);
    assert_eq!(e.entities[0].position, original.position);
    assert!(e.sources.iter().all(|s| s.base == s.draft));
    assert_eq!(e.coordinate_clipboard, Some(point));
    assert_eq!(e.clipboard.as_ref().unwrap().row, copied);
    e.select_entity(0);
    e.paste_coordinates().unwrap();
    assert_eq!(e.selected().unwrap().position, point);
    assert!(!e.work_path().exists());
}

#[test]
fn new_session_hides_and_retires_old_preview_and_clears_geometry_caches() {
    use bevy::ecs::system::RunSystemOnce;
    let mut world = World::new();
    let root = world.spawn(Visibility::Inherited).id();
    let child = world.spawn(ChildOf(root)).id();
    let mut preview = preview::WorldPreview::default();
    preview.set_test_root(root);
    preview.objects.insert("old".into(), child);
    preview.terrains.insert("old".into(), child);
    preview.terrain_applied.insert("old".into(), 1);
    world.insert_resource(preview);
    world
        .run_system_once(
            |mut commands: Commands, mut p: ResMut<preview::WorldPreview>| {
                p.reset(&mut commands);
            },
        )
        .unwrap();
    assert_eq!(world.get::<Visibility>(root), Some(&Visibility::Hidden));
    let p = world.resource::<preview::WorldPreview>();
    assert!(p.objects.is_empty() && p.terrains.is_empty() && p.terrain_applied.is_empty());
    for _ in 0..5 {
        preview::retire(&mut world);
    }
    assert!(!world.entities().contains(root));
    assert!(!world.entities().contains(child));
}
