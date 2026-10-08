use super::*;
use serde_json::json;

#[path="tests_authoring.rs"]
mod authoring;
#[path="tests_native_authoring.rs"]
mod native_authoring;
#[path="tests_session.rs"]
mod session;

fn empty_catalog() -> EditorCatalog {
    EditorCatalog {
        entries: vec![],
        hnpc: None,
        npc_count: 0,
        nano_count: 0,
        shiny_models: BTreeMap::new(),
    }
}

#[test]
fn empty_point_selection_clears_entity_without_moving_it_or_changing_history() {
    let (_dir,mut e)=fixture();
    e.select_entity(0);
    let position=e.selected().unwrap().position;
    let sources=e.sources.iter().map(|s|s.draft.clone()).collect::<Vec<_>>();
    let history=e.undo.len();
    e.focus=Some(Field::X);
    let point=Vec3::new(-100.,8.,200.);
    e.select_coordinates(Some(point)).unwrap();
    assert_eq!(e.selected,None);
    assert_eq!(e.selected_point,Some(point));
    assert!(e.drag.is_none()&&e.focus.is_none());
    assert_eq!(e.entities[0].position,position);
    assert_eq!(e.undo.len(),history);
    assert_eq!(e.sources.iter().map(|s|s.draft.clone()).collect::<Vec<_>>(),sources);
    e.select_entity(0);
    assert_eq!(e.selected_point,None);
    e.select_coordinates(None).unwrap();
    assert_eq!(e.selected,None);
    assert_eq!(e.selected_point,None,"A sky click must not invent ground coordinates");
}

#[test]
fn copied_point_moves_selected_npc_preserving_angle_instance_and_undo() {
    let (_dir,mut e)=fixture();
    let point=Vec3::new(-16.,13.,19.);
    e.select_coordinates(Some(point)).unwrap();
    e.copy_coordinates().unwrap();
    e.select_entity(0);
    let original=e.selected().unwrap().clone();
    let before=e.sources[original.source].draft.clone();
    let history=e.undo.len();
    e.paste_coordinates().unwrap();
    let moved=e.selected().unwrap();
    assert_eq!(moved.position,point);
    assert_eq!((moved.angle,moved.instance),(original.angle,original.instance));
    assert_eq!(e.undo.len(),history+1);
    e.undo(false).unwrap();
    assert_eq!(e.sources[original.source].draft,before);
    e.undo(true).unwrap();
    assert_eq!(e.selected().unwrap().position,point);
    assert_eq!(e.coordinate_clipboard,Some(point));
}

#[test]
fn pasted_object_coordinates_move_collision_in_the_same_transaction() {
    let (_dir,mut e)=fixture();
    map_fixture(&e,[0,0]);
    model::write_atomic(&e.root.join("map/catalog.json"),&json!({})).unwrap();
    e.center=atlas::tile_center([0,0],3.);
    e.load_tile().unwrap();
    let index=e.entities.iter().position(|p|p.kind==4).unwrap();
    let original=e.entities[index].position;
    let point=original+Vec3::new(-6.,7.,8.);
    e.available_tiles.clear();
    e.select_coordinates(Some(point)).unwrap();
    e.copy_coordinates().unwrap();
    e.select_entity(index);
    let scene=e.maps["map_00_00"].scene;
    let before=e.sources[scene].draft.clone();
    e.paste_coordinates().unwrap();
    assert_eq!(e.selected().unwrap().position,point);
    assert_eq!(e.sources[scene].draft["visuals"][0]["transform"],e.sources[scene].draft["colliders"][0]["transform"]);
    e.undo(false).unwrap();
    assert_eq!(e.sources[scene].draft,before);
}

#[test]
fn object_list_and_global_search_use_model_identity_and_nearby_selection_stays_draggable() {
    let (_dir, mut e) = fixture();
    for coord in [[0, 0], [1, 1]] {
        map_fixture(&e, coord);
        let path = e
            .root
            .join(format!("map/tiles/{}/scene.json", atlas::tile_id(coord)));
        let mut scene = terrain::read_source(&path).unwrap();
        scene["visuals"][0]["name"] = json!("unnamed-static-object [test#1 visual]");
        scene["visuals"][0]["model"] = json!("accepted-model");
        scene["models"] =
            json!([{"id":"accepted-model","rootName":"Oak_Tree","path":"objects/tree/visual.glb"}]);
        model::write_atomic(&path, &scene).unwrap();
    }
    model::write_atomic(&e.root.join("map/catalog.json"), &json!({})).unwrap();
    e.center = atlas::tile_center([0, 0], 3.);
    e.load_tile().unwrap();
    let catalog = empty_catalog();
    e.instance = 0;
    let index = e.entities.iter().position(|p| p.kind == 4).unwrap();
    e.switch_list(1);
    assert_eq!(
        e.filtered(&catalog),
        vec![index],
        "Empty search lists the loaded objects"
    );
    e.search = " Tree ".into();
    assert_eq!(e.filtered(&catalog), vec![index]);
    e.object_index = objects::read(&e.root, BTreeSet::from([[0, 0], [1, 1]])).unwrap();
    assert_eq!(
        e.remote_objects().len(),
        1,
        "Unloaded tiles use the same searchable name"
    );
    e.search = "unmatched".into();
    e.switch_list(0);
    assert!(!e.edit_objects);
    e.switch_list(2);
    assert!(e.filtered(&catalog).contains(&index));
    e.terrain_tool = Some(0);
    e.select_entity(index);
    assert!(e.edit_objects);
    assert_eq!(e.terrain_tool, None);
    assert!(
        e.displayed(&catalog).contains(&index),
        "Selection bypasses the stored object keyword"
    );
    assert!(e.pickable().contains(&index));
    let p = e.entities[index].clone();
    let next = p.position + Vec3::new(5., 0., 7.);
    e.transform(index, next, p.angle, p.instance).unwrap();
    assert_eq!(e.selected().unwrap().position, next);
    let scene = e.maps["map_00_00"].scene;
    assert_eq!(
        e.sources[scene].draft["visuals"][0]["transform"],
        e.sources[scene].draft["colliders"][0]["transform"]
    );
    e.undo(false).unwrap();
    assert_eq!(e.selected().unwrap().position, p.position);
}

#[test]
fn object_mesh_picking_ignores_list_keyword_and_hits_away_from_pivot() {
    use bevy::{camera::primitives::Aabb, ecs::system::RunSystemOnce};
    let (_dir, mut e) = fixture();
    map_fixture(&e, [0, 0]);
    model::write_atomic(&e.root.join("map/catalog.json"), &json!({})).unwrap();
    e.center = atlas::tile_center([0, 0], 3.);
    e.load_tile().unwrap();
    let index = e.entities.iter().position(|p| p.kind == 4).unwrap();
    e.instance = 0;
    e.switch_list(1);
    e.search = "unmatched".into();
    assert!(!e.displayed(&empty_catalog()).contains(&index));
    let position = e.entities[index].position + Vec3::X * 50.;
    let mut world = World::new();
    let owner = world.spawn_empty().id();
    world.spawn((
        ChildOf(owner),
        Aabb::from_min_max(Vec3::splat(-1.), Vec3::splat(1.)),
        GlobalTransform::from(Transform::from_translation(position)),
    ));
    let mut preview = preview::WorldPreview::default();
    preview
        .objects
        .insert("map_00_00:visual:bench [test#1]".into(), owner);
    world.insert_resource(e);
    world.insert_resource(preview);
    let ray = Ray3d::new(position - Vec3::Z * 10., Dir3::Z);
    let picked = world
        .run_system_once(
            move |e: Res<WorldEditor>,
                  preview: Res<preview::WorldPreview>,
                  meshes: Query<(Entity, &Aabb, &GlobalTransform)>,
                  parents: Query<&ChildOf>| {
                picking::entity(&e, &preview, ray, &meshes, &parents)
            },
        )
        .unwrap();
    assert_eq!(picked, Some(index));
}

#[test]
fn similarly_numbered_script_node_does_not_block_static_object_movement() {
    let (_dir, mut e) = fixture();
    map_fixture(&e, [0, 0]);
    model::write_atomic(&e.root.join("map/catalog.json"), &json!({})).unwrap();
    let path = e.root.join("map/tiles/map_00_00/behaviour.json");
    model::write_atomic(&path, &json!({"triggers":[{"node":"test#10"}]})).unwrap();
    e.center = atlas::tile_center([0, 0], 3.);
    e.load_tile().unwrap();
    let index = e.entities.iter().position(|p| p.kind == 4).unwrap();
    e.select_entity(index);
    let p = e.entities[index].clone();
    e.transform(index, p.position + Vec3::X, p.angle, p.instance)
        .unwrap();
    model::write_atomic(&path, &json!({"triggers":[{"node":"test#1"}]})).unwrap();
    assert!(e.transform(index, p.position, p.angle, p.instance).is_err());
    assert_eq!(
        e.undo.len(),
        1,
        "Real script ownership still rejects edits atomically"
    );
}

#[test]
fn deleting_and_pasting_objects_keeps_owned_collision_and_one_undo() {
    let (_dir, mut e) = fixture();
    map_fixture(&e, [0, 0]);
    fs::create_dir_all(e.root.join("map")).unwrap();
    model::write_atomic(&e.root.join("map/catalog.json"), &json!({})).unwrap();
    e.center = Vec3::ZERO;
    e.load_tile().unwrap();
    e.selected = e.entities.iter().position(|p| p.kind == 4);
    let original = e.snapshot(e.selected().unwrap()).unwrap();
    let source = original.placement.source;
    e.copy().unwrap();
    e.delete_selected().unwrap();
    assert!(
        e.sources[source].draft["objects"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let scene = e.maps["map_00_00"].scene;
    assert!(
        e.sources[scene].draft["colliders"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    e.undo(false).unwrap();
    assert_eq!(e.sources[source].draft["objects"][0], original.row);
    e.undo(true).unwrap();
    e.available_tiles.clear();
    e.center = Vec3::new(-8., 3., 9.);
    e.paste().unwrap();
    assert_eq!(
        e.sources[source].draft["objects"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        e.sources[scene].draft["visuals"][0]["transform"],
        e.sources[scene].draft["colliders"][0]["transform"]
    );
    e.undo(false).unwrap();
    assert!(
        e.sources[source].draft["objects"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn moving_invisible_npc_updates_or_creates_waypoint_in_the_same_history_entry() {
    let (_dir, mut e) = fixture();
    let path = e.root.join("data/missions/client-npc-waypoints.json");
    model::write_atomic(&path,&json!({"schema":"ffone.client-npc-waypoints.v1","rows":[{"npcType":2,"clientPosition":[4,5,6],"keep":true}]})).unwrap();
    let before = e.sources[0].draft.clone();
    let p = e.entities[0].clone();
    let moved = p.position + Vec3::new(-1., 2., 3.);
    e.transform(0, moved, p.angle, p.instance).unwrap();
    let source = e.sources.iter().position(|s| s.path == path).unwrap();
    assert_eq!(
        e.sources[source].draft["rows"][1]["clientPosition"],
        json!([-moved.x, moved.y, moved.z])
    );
    assert_eq!(e.undo.len(), 1);
    e.undo(false).unwrap();
    assert_eq!(e.sources[0].draft, before);
    assert_eq!(e.sources[source].draft["rows"].as_array().unwrap().len(), 1);
    e.undo(true).unwrap();
    assert_eq!(e.sources[source].draft["rows"][0]["keep"], true);
}

#[test]
fn new_instance_is_staged_in_both_client_and_server_and_is_undoable() {
    let (_dir, mut e) = fixture();
    let value =
        json!({"m_pInstanceTable":{"m_pInstanceData":[{"m_iInstanceNameID":0,"extra":"keep"}]}});
    for path in [
        e.root.join("data/tables/xdt.json"),
        e.folder.as_ref().unwrap().join("xdt.json"),
    ] {
        model::write_atomic(&path, &value).unwrap();
    }
    e.register_instance(159).unwrap();
    for source in e.sources.iter().filter(|s| s.path.ends_with("xdt.json")) {
        assert_eq!(
            source.draft["m_pInstanceTable"]["m_pInstanceData"][0]["extra"],
            "keep"
        );
        assert_eq!(
            source.draft["m_pInstanceTable"]["m_pInstanceData"][1]["m_iInstanceNameID"],
            159
        );
    }
    e.undo(false).unwrap();
    for source in e.sources.iter().filter(|s| s.path.ends_with("xdt.json")) {
        assert_eq!(source.draft, value);
    }
}

#[test]
fn terrain_strokes_undo_and_publish_reopen_with_valid_height_weight_mips_and_collision() {
    fn copy_tree(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    fn copy_shared(value: &Value, from: &Path, to: &Path) {
        match value {
            Value::String(path) if path.starts_with("map/shared/") && from.join(path).is_file() => {
                let target = to.join(path);
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::copy(from.join(path), target).unwrap();
            }
            Value::Array(values) => {
                for value in values {
                    copy_shared(value, from, to);
                }
            }
            Value::Object(values) => {
                for value in values.values() {
                    copy_shared(value, from, to);
                }
            }
            _ => {}
        }
    }
    let (_dir, mut e) = fixture();
    let published = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let id = "map_12_03";
    let relative = format!("map/tiles/{id}");
    copy_tree(&published.join(&relative), &e.root.join(&relative));
    let path = e.root.join(format!("{relative}/terrain/terrain.json"));
    let descriptor = terrain::read_source(&path).unwrap();
    copy_shared(&descriptor, &published, &e.root);
    model::write_atomic(&e.root.join("map/catalog.json"), &json!({})).unwrap();
    e.available_tiles.insert([12, 3]);
    e.center = atlas::tile_center([12, 3], 0.);
    e.load_tile().unwrap();
    e.load_ground(e.center).unwrap();
    let point = e.snap_to_ground(e.center).unwrap();
    e.select_coordinates(Some(point+Vec3::Y*200.)).unwrap();
    assert!((e.selected_point.unwrap().y-point.y).abs()<0.001,"Empty-point coordinates use the terrain height");
    let old = e.ground[id].samples.clone();
    let old_height = e.ground_height(point).unwrap();
    e.terrain_tool = Some(0);
    e.brush_radius = 20.;
    e.brush_strength = 20.;
    e.terrain_brush(point, 0.1, false).unwrap();
    e.finish_brush().unwrap();
    assert!(e.ground_height(point).unwrap() > old_height);
    let raised = e.ground[id].samples.clone();
    assert!(raised != old);
    e.undo(false).unwrap();
    assert!(
        e.ground[id].samples == old,
        "Undo must restore the heightmap"
    );
    e.undo(true).unwrap();
    assert!(
        e.ground[id].samples == raised,
        "Redo must restore the raised heightmap"
    );
    e.terrain_tool = Some(1);
    let mut target = point;
    target.y = old_height;
    e.terrain_brush(target, 0.1, false).unwrap();
    e.finish_brush().unwrap();
    assert!(e.ground_height(point).unwrap() < old_height + 2.1);
    e.terrain_tool = Some(2);
    e.brush_layer = 1;
    let old_weights = e.ground[id].weights.clone();
    e.terrain_brush(point, 0.1, false).unwrap();
    e.finish_brush().unwrap();
    assert!(e.ground[id].weights != old_weights);
    let painted = e.ground[id].weights.clone();
    e.undo(false).unwrap();
    assert!(
        e.ground[id].weights == old_weights,
        "Undo must restore texture weights"
    );
    e.undo(true).unwrap();
    assert!(
        e.ground[id].weights == painted,
        "Redo must restore painted weights"
    );
    e.terrain_tool=Some(3);
    e.brush_layer=e.ground[id].descriptor["splat"]["layers"].as_array().unwrap().iter().position(|layer|layer["trueTextureName"].as_str().is_some_and(terrain::path_texture)).unwrap();
    e.terrain_brush(point,0.1,false).unwrap();e.finish_brush().unwrap();
    assert!(e.ground[id].weights!=painted,"Path brushes use the chosen road texture's control layer");
    e.undo(false).unwrap();assert!(e.ground[id].weights==painted);e.undo(true).unwrap();
    let painted=e.ground[id].weights.clone();
    e.publish().unwrap();
    let scene = terrain::read_source(&e.root.join(format!("{relative}/scene.json"))).unwrap();
    let native = ffone_client::native_terrain::NativeTerrain::open_with_authoritative_environment(
        &e.root,
        &format!("{relative}/terrain/terrain.json"),
        scene["nativeTerrain"]["blake3"].as_str().unwrap(),
        scene["nativeTerrain"]["environment"]["blake3"].as_str(),
    )
    .unwrap();
    assert!(native.height_samples() == e.ground[id].samples.as_slice());
    assert!(
        native
            .editor_weight_mips()
            .iter()
            .map(|c| c[0].2.clone())
            .collect::<Vec<_>>()
            == painted
    );
    let reloaded = ground::Tile::load(&e.root, id).unwrap();
    assert!((reloaded.height_at(point).unwrap() - e.ground_height(point).unwrap()).abs() < 0.001);
}

fn fixture() -> (tempfile::TempDir, WorldEditor) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("assets/game");
    fs::create_dir_all(&root).unwrap();
    let folder = dir.path().join("server/tabledata");
    fs::create_dir_all(&folder).unwrap();
    model::write_atomic(
        &folder.join("NPCs.json"),
        &json!({"metadata":{"keep":true},"NPCs":{
        "7":{"iNPCType":1,"iX":100.25,"iY":200.75,"iZ":300.5,"iAngle":0,"unknown":[3,4]},
        "9":{"iNPCType":2,"iX":400,"iY":500,"iZ":600,"iAngle":45,"iMapNum":81}}}),
    )
    .unwrap();
    model::write_atomic(&folder.join("mobs.json"),&json!({"mobs":{},"groups":{"3":{"iNPCType":1,"iX":0,"iY":0,"iZ":10,"iAngle":-45,"aFollowers":[{"iNPCType":2,"iOffsetX":20,"iOffsetY":30}]}}})).unwrap();
    model::write_atomic(&folder.join("gruntwork.json"),&json!({"rotations":[{"iNPCID":8,"iAngle":90}],"instances":[{"iNPCID":8,"iMapNum":12}],"mobs":[]})).unwrap();
    let mut editor = WorldEditor::open(root);
    editor.load_server(&folder).unwrap();
    (dir, editor)
}
#[test]
fn placement_edit_uses_authoritative_overrides_and_preserves_fractional_axes() {
    let (_dir, mut e) = fixture();
    let before = e.sources[0].draft.clone();
    let p = e.entities[0].clone();
    assert_eq!(p.angle, 90.);
    assert_eq!(p.instance, 12);
    e.transform(0, p.position + Vec3::new(-1., 0., 0.), 135., 81)
        .unwrap();
    let row = &e.sources[0].draft["NPCs"]["7"];
    assert_eq!(row["iX"], 200);
    assert_eq!(row["iY"], 200.75);
    assert_eq!(row["iZ"], 300.5);
    assert_eq!(row["unknown"], json!([3, 4]));
    assert_eq!(row["iAngle"], 0);
    let grunt = e
        .sources
        .iter()
        .find(|s| s.path.ends_with("gruntwork.json"))
        .unwrap();
    assert_eq!(grunt.draft["rotations"][0]["iAngle"], 135);
    assert_eq!(grunt.draft["instances"][0]["iMapNum"], 81);
    e.undo(false).unwrap();
    assert_eq!(e.sources[0].draft, before);
    assert_eq!(e.entities[0].instance, 12);
    e.undo(true).unwrap();
    assert_eq!(e.entities[0].instance, 81);
}
#[test]
fn duplicate_group_keeps_followers_and_allocates_free_placement_id() {
    let (_dir, mut e) = fixture();
    let index = e.entities.iter().position(|p| p.kind == 2).unwrap();
    e.selected = Some(index);
    e.instance = 42;
    let original = e.sources[e.entities[index].source].draft["groups"]["3"].clone();
    e.place(Vec3::new(-5., 2., 6.), true).unwrap();
    let p = e.selected().unwrap();
    assert_eq!(p.pointer, "/groups/4");
    assert_eq!(p.instance, 42);
    let new = e.sources[p.source].draft.pointer(&p.pointer).unwrap();
    assert_eq!(new["aFollowers"], original["aFollowers"]);
    e.undo(false).unwrap();
    assert!(!e.entities.iter().any(|p| p.pointer == "/groups/4"));
}
#[test]
fn work_survives_restart_and_publish_refuses_external_changes() {
    let (_dir, mut e) = fixture();
    let p = e.entities[0].clone();
    e.transform(0, p.position, 180., p.instance).unwrap();
    e.save_work().unwrap();
    let mut restored = WorldEditor::open(e.root.clone());
    restored.restore_work().unwrap();
    assert_eq!(restored.entities[0].angle, 180.);
    let source = restored
        .sources
        .iter()
        .position(|s| s.path.ends_with("gruntwork.json"))
        .unwrap();
    let mut external = restored.sources[source].base.clone();
    external["external"] = json!(true);
    model::write_atomic(&restored.sources[source].path, &external).unwrap();
    assert!(
        restored
            .publish()
            .unwrap_err()
            .contains("changed externally")
    );
    let disk: Value =
        serde_json::from_slice(&fs::read(&restored.sources[source].path).unwrap()).unwrap();
    assert_eq!(disk, external);
}
#[test]
fn object_visual_and_collision_move_together_and_reference_guards_match_written_bytes() {
    let (_dir, mut e) = fixture();
    let tile = e.root.join("map/tiles/map_00_00");
    fs::create_dir_all(&tile).unwrap();
    let transform = json!({"translation":[1.,2.,3.],"rotation":[0.,0.,0.,1.],"scale":[1.,2.,1.]});
    model::write_atomic(&tile.join("objects.json"),&json!({"objects":[{"object":"accepted-object","sourceNode":"map#1","worldMatrix":[[1,0,0,1],[0,2,0,2],[0,0,1,3],[0,0,0,1]],"unknown":"keep"}]})).unwrap();
    model::write_atomic(&tile.join("scene.json"),&json!({"visuals":[{"name":"bench [map#1]","transform":transform}],"colliders":[{"name":"bench [map#1 collision]","transform":transform}]})).unwrap();
    fs::write(tile.join("behaviour.json"), "{}").unwrap();
    let guard = |path| json!({"path":path,"bytes":0,"blake3":"old"});
    model::write_atomic(&tile.join("tile.json"),&json!({"scene":guard("map/tiles/map_00_00/scene.json"),"objects":guard("map/tiles/map_00_00/objects.json"),"files":[guard("map/tiles/map_00_00/scene.json"),guard("map/tiles/map_00_00/objects.json")]})).unwrap();
    model::write_atomic(&e.root.join("map/catalog.json"),&json!({"tiles":[{"tileId":"map_00_00","manifest":guard("map/tiles/map_00_00/tile.json"),"objects":guard("map/tiles/map_00_00/objects.json")}]})).unwrap();
    e.center = Vec3::ZERO;
    e.load_tile().unwrap();
    let i = e.entities.iter().position(|p| p.kind == 4).unwrap();
    let p = e.entities[i].clone();
    e.transform(i, p.position + Vec3::new(10., 0., 0.), p.angle, 0)
        .unwrap();
    let tile_indices = e.maps.get("map_00_00").unwrap();
    let scene = tile_indices.scene;
    assert_eq!(
        e.sources[scene].draft["visuals"][0]["transform"],
        e.sources[scene].draft["colliders"][0]["transform"]
    );
    assert_eq!(
        e.sources[scene].draft["visuals"][0]["transform"]["translation"],
        json!([11., 2., 3.])
    );
    e.publish().unwrap();
    let bytes = fs::read(tile.join("scene.json")).unwrap();
    let manifest: Value =
        serde_json::from_slice(&fs::read(tile.join("tile.json")).unwrap()).unwrap();
    assert_eq!(
        manifest["scene"]["blake3"],
        blake3::hash(&bytes).to_hex().to_string()
    );
    assert_eq!(manifest["scene"]["bytes"], bytes.len());
    let catalog: Value =
        serde_json::from_slice(&fs::read(e.root.join("map/catalog.json")).unwrap()).unwrap();
    assert_eq!(
        catalog["tiles"][0]["manifest"]["blake3"],
        blake3::hash(&fs::read(tile.join("tile.json")).unwrap())
            .to_hex()
            .to_string()
    );
}
#[test]
fn xy_marker_heading_matches_runtime_forward() {
    for angle in [0, 90, -90, 180] {
        let forward = ffone_client::coordinates::ProtocolYawDegrees::new(angle)
            .native_root_rotation()
            * Vec3::NEG_Z;
        let a = (angle as f32).to_radians();
        let shown = Vec2::new(-a.sin(), a.cos());
        assert!((shown - Vec2::new(-forward.x, -forward.z)).length() < 0.0001);
    }
}

#[test]
fn atlas_matches_runtime_minimap_and_selects_the_containing_xy_tile() {
    for (x, z) in [(768., 768.), (2200., 6400.), (7900., 7900.), (512., 512.)] {
        let position = Vec3::new(-x, 31., z);
        let sample = ffone_client::ui::gameplay::minimap_tiles(x, z, 4.)
            .into_iter()
            .find(|s| {
                s.destination.x <= 74.
                    && s.destination.x + s.destination.width >= 74.
                    && s.destination.y <= 74.
                    && s.destination.y + s.destination.height >= 74.
            })
            .unwrap();
        let pixel = Vec2::new(
            sample.source.x
                + (74. - sample.destination.x) / sample.destination.width * sample.source.width,
            sample.source.y
                + (74. - sample.destination.y) / sample.destination.height * sample.source.height,
        );
        let origin = atlas::image_origin(sample.tile_number);
        let actual = Vec2::new(x + origin.x, origin.z - z) / 4.;
        assert!((actual - pixel).length() < 0.001);
        let screen = atlas::project(
            Vec3::new(-4000., 0., 4000.),
            0.2,
            position,
            Vec2::new(500., 400.),
        );
        let restored = atlas::unproject(
            Vec3::new(-4000., 31., 4000.),
            0.2,
            screen,
            Vec2::new(500., 400.),
        );
        assert!((restored - position).length() < 0.001);
        assert_eq!(
            atlas::tile_at(position),
            Some([(x / 512.).floor() as i32, (z / 512.).floor() as i32])
        );
    }
    assert_eq!(atlas::tile_at(Vec3::new(-511.99, 0., 511.99)), Some([0, 0]));
    assert_eq!(atlas::tile_at(Vec3::new(-512., 0., 512.)), Some([1, 1]));
    assert_eq!(atlas::tile_at(Vec3::NAN), None);
}

fn map_fixture(e: &WorldEditor, coord: [i32; 2]) {
    let id = atlas::tile_id(coord);
    let tile = e.root.join(format!("map/tiles/{id}"));
    fs::create_dir_all(&tile).unwrap();
    let p = atlas::tile_center(coord, 3.);
    let transform = json!({"translation":p.to_array(),"rotation":[0.,0.,0.,1.],"scale":[1.,1.,1.]});
    model::write_atomic(&tile.join("objects.json"),&json!({"objects":[{"object":"accepted-object","sourceNode":"test#1","worldMatrix":[[1,0,0,p.x],[0,1,0,p.y],[0,0,1,p.z],[0,0,0,1]]}]})).unwrap();
    model::write_atomic(&tile.join("scene.json"),&json!({"visuals":[{"name":"bench [test#1]","transform":transform}],"colliders":[{"name":"bench [test#1 collision]","transform":transform}]})).unwrap();
    model::write_atomic(&tile.join("tile.json"), &json!({})).unwrap();
    fs::write(tile.join("behaviour.json"), "{}").unwrap();
}
#[test]
fn region_switch_keeps_nine_tiles_preserves_neighbour_edits_and_stable_selection() {
    let (_dir, mut e) = fixture();
    for tile in atlas::neighbours([4, 5]).into_iter().chain([[10, 10]]) {
        map_fixture(&e, tile);
    }
    model::write_atomic(&e.root.join("map/catalog.json"), &json!({})).unwrap();
    e.select_region([4, 5]).unwrap();
    assert_eq!(e.desired_tiles().len(), 9);
    assert_eq!(e.desired_tiles()[0], [4, 5]);
    let install = |e: &mut WorldEditor, coord: [i32; 2]| {
        let id = atlas::tile_id(coord);
        let documents = stream::read_tile(&e.root, &id, false).unwrap();
        e.install_tile(id, documents);
    };
    install(&mut e, [5, 6]);
    e.selected = e.entities.iter().position(|p| p.key == "map_05_06/0");
    for tile in e.desired_tiles() {
        install(&mut e, tile);
    }
    assert_eq!(e.maps.len(), 9);
    assert_eq!(e.selected().unwrap().key, "map_05_06/0");
    let p = e.selected().unwrap().clone();
    let moved = p.position + Vec3::new(-10., 0., 20.);
    e.transform(e.selected.unwrap(), moved, 90., 0).unwrap();
    let source = p.source;
    assert_eq!(
        e.sources[source].draft["objects"][0]["worldMatrix"][0][3],
        moved.x.to_string()
    );
    e.select_region([10, 10]).unwrap();
    install(&mut e, [10, 10]);
    assert_eq!(e.maps.len(), 1);
    assert!(e.sources[source].draft != e.sources[source].base);
    e.select_region([4, 5]).unwrap();
    install(&mut e, [5, 6]);
    assert_eq!(
        e.entities
            .iter()
            .find(|p| p.key == "map_05_06/0")
            .unwrap()
            .position,
        moved
    );
    e.undo(false).unwrap();
    assert_eq!(e.selected().unwrap().key, "map_05_06/0");
    assert_eq!(e.selected().unwrap().position, p.position);
    e.save_work().unwrap();
    let mut restored = WorldEditor::open(e.root.clone());
    restored.restore_work().unwrap();
    assert_eq!(restored.region, Some([4, 5]));
    assert_eq!(restored.maps.len(), 1);
}
#[test]
fn retiring_large_tile_is_bounded_and_releases_all_descendants() {
    let mut world = World::new();
    world.init_resource::<preview::WorldPreview>();
    let root = world.spawn_empty().id();
    let mut owned = vec![root];
    for _ in 0..900 {
        let branch = world.spawn(ChildOf(root)).id();
        let leaf = world.spawn(ChildOf(branch)).id();
        owned.extend([branch, leaf]);
    }
    world
        .resource_mut::<preview::WorldPreview>()
        .retiring
        .push(root);
    let before = owned.len();
    preview::retire(&mut world);
    assert!(
        before
            - owned
                .iter()
                .filter(|&&e| world.entities().contains(e))
                .count()
            <= 256
    );
    assert!(world.entities().contains(root));
    for _ in 0..200 {
        preview::retire(&mut world);
        if !world.entities().contains(root) {
            break;
        }
    }
    assert!(owned.iter().all(|&e| !world.entities().contains(e)));
}
