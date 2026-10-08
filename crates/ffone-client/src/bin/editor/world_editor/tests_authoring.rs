use super::*;

#[test]
fn routes_allocate_ids_assign_all_actor_kinds_and_survive_restart_without_rewriting_sources() {
    let (_dir, mut e) = fixture();
    let folder = e.folder.clone().unwrap();
    let routes = json!({"npc":{"9":{"aPoints":[{"iX":10,"iY":20,"iZ":30},{"iX":100,"iY":200,"iZ":300}],"aNPCTypes":[],"aNPCIDs":[],"iBaseSpeed":300,"bLoop":true,"keep":"old"}},"skyway":{},"slider":{}});
    model::write_atomic(&folder.join("paths.json"), &routes).unwrap();
    let mobs = e
        .sources
        .iter()
        .position(|s| s.path.ends_with("mobs.json"))
        .unwrap();
    e.sources[mobs].draft["mobs"]["8"] = json!({"iNPCType":2,"iX":0,"iY":0,"iZ":0});
    e.rebuild();
    e.open_routes().unwrap();
    e.new_route().unwrap();
    assert_eq!(e.routes.as_ref().unwrap().id, 10);
    assert!(e.save_route().is_err());
    e.routes.as_mut().unwrap().points = vec![
        json!({"iX":100,"iY":200,"iZ":300,"iStopTicks":2}),
        json!({"iX":400,"iY":500,"iZ":600,"iStopTicks":0}),
    ];
    e.save_route().unwrap();
    for kind in 0..=2 {
        e.selected = e.entities.iter().position(|p| p.kind == kind);
        let p = e.selected().unwrap().clone();
        e.assign_route().unwrap();
        assert_eq!(
            e.sources[p.source].draft.pointer(&p.pointer).unwrap()["iPathID"],
            10
        );
        e.undo(false).unwrap();
        assert!(
            e.sources[p.source]
                .draft
                .pointer(&p.pointer)
                .unwrap()
                .get("iPathID")
                .is_none()
        );
        e.undo(true).unwrap();
    }
    e.save_work().unwrap();
    let disk = terrain::read_source(&folder.join("paths.json")).unwrap();
    assert_eq!(disk, routes);
    let mut restored = WorldEditor::open(e.root.clone());
    restored.restore_work().unwrap();
    restored.open_routes().unwrap();
    let source = restored.routes.as_ref().unwrap().source;
    assert_eq!(
        restored.sources[source].draft["npc"]["10"]["aPoints"][0]["iStopTicks"],
        2
    );
    for p in restored.entities.iter().filter(|p| {
        p.kind <= 2
            && restored.sources[p.source]
                .draft
                .pointer(&p.pointer)
                .unwrap()
                .get("iPathID")
                .is_some()
    }) {
        assert_eq!(
            restored.sources[p.source]
                .draft
                .pointer(&p.pointer)
                .unwrap()["iPathID"],
            10
        );
    }
    restored.new_route().unwrap();
    assert_eq!(restored.routes.as_ref().unwrap().id, 11);
}

#[test]
fn transport_route_edit_preserves_linked_id_and_slider_numeric_order() {
    let (_dir, mut e) = fixture();
    model::write_atomic(&e.folder.as_ref().unwrap().join("paths.json"),&json!({"npc":{},"skyway":{"7":{"iRouteID":15,"iMonkeySpeed":1500,"keep":true,"aPoints":[{"iX":1,"iY":2,"iZ":3},{"iX":5,"iY":6,"iZ":7}]}},"slider":{}})).unwrap();
    e.open_routes().unwrap();
    routes::action(&mut e, Action::RouteKind(1)).unwrap();
    e.choose_route(0).unwrap();
    let r = e.routes.as_mut().unwrap();
    r.target = 100;
    r.speed = 1200;
    r.points[0]["iX"] = json!(20);
    e.save_route().unwrap();
    let source = e.routes.as_ref().unwrap().source;
    assert_eq!(e.sources[source].draft["skyway"]["7"]["iRouteID"], 15);
    assert_eq!(e.sources[source].draft["skyway"]["7"]["keep"], true);
    routes::action(&mut e, Action::RouteKind(2)).unwrap();
    e.new_route().unwrap();
    e.routes.as_mut().unwrap().points = (0..12)
        .map(|i| json!({"iX":i,"iY":0,"iZ":0,"bStop":i==0}))
        .collect();
    e.save_route().unwrap();
    e.choose_route(0).unwrap();
    assert_eq!(e.routes.as_ref().unwrap().points[10]["iX"], 10);
    assert_eq!(e.routes.as_ref().unwrap().points[0]["bStop"], true);
}

#[test]
fn type_picker_single_click_previews_double_click_commits_and_can_be_undone() {
    let (_dir, mut e) = fixture();
    e.selected = Some(0);
    let p = e.selected().unwrap().clone();
    let catalog = empty_catalog();
    e.open_type_picker(true, &catalog).unwrap();
    let picker = e.type_picker.as_mut().unwrap();
    picker.choices.push(type_picker::Choice::Actor {
        id: 2,
        catalog: 0,
        label: "2 · Fusion Spawn".into(),
    });
    picker.filter = "fusion".into();
    assert_eq!(picker.filtered(), vec![0]);
    type_picker::action(&mut e, Action::PickerSelect(0), &catalog).unwrap();
    assert_eq!(e.entities[0].type_id, 1);
    assert!(e.type_picker.is_some());
    type_picker::action(&mut e, Action::PickerSelect(0), &catalog).unwrap();
    assert_eq!(e.entities[0].type_id, 2);
    assert!(e.type_picker.is_none());
    assert_eq!(
        e.sources[p.source].draft.pointer(&p.pointer).unwrap()["unknown"],
        json!([3, 4])
    );
    e.undo(false).unwrap();
    assert_eq!(e.entities[0].type_id, 1);
}

#[test]
fn replacing_a_world_model_keeps_placement_and_updates_visuals_and_collision_together() {
    let (_dir, mut e) = fixture();
    for at in [[0, 0], [1, 1]] {
        map_fixture(&e, at);
        let path = e
            .root
            .join(format!("map/tiles/{}/scene.json", atlas::tile_id(at)));
        let mut scene = terrain::read_source(&path).unwrap();
        let id = format!("model-{}", at[0]);
        scene["visuals"][0]["model"] = json!(id);
        scene["models"] = json!([{"id":id,"path":format!("objects/{id}.glb")}]);
        model::write_atomic(&path, &scene).unwrap();
        let path = e
            .root
            .join(format!("map/tiles/{}/objects.json", atlas::tile_id(at)));
        let mut row = terrain::read_source(&path).unwrap();
        row["objects"][0]["object"] = json!(id);
        model::write_atomic(&path, &row).unwrap();
    }
    model::write_atomic(&e.root.join("map/catalog.json"), &json!({})).unwrap();
    e.center = atlas::tile_center([0, 0], 3.);
    e.load_tile().unwrap();
    e.selected = e.entities.iter().position(|p| p.kind == 4);
    let p = e.selected().unwrap().clone();
    let template = model_templates::snapshot(
        &e.root,
        &objects::Entry {
            tile: "map_01_01".into(),
            node: "test#1".into(),
            asset: "model-1".into(),
            name: "Tree".into(),
            model: Some("objects/model-1.glb".into()),
            position: atlas::tile_center([1, 1], 3.),
        },
    )
    .unwrap();
    e.replace_object_model(&p, template).unwrap();
    assert_eq!(e.selected().unwrap().position, p.position);
    assert_eq!(e.maps.len(), 1);
    assert_eq!(
        e.sources[p.source].draft.pointer(&p.pointer).unwrap()["object"],
        "model-1"
    );
    let scene = e.maps["map_00_00"].scene;
    assert_eq!(e.sources[scene].draft["visuals"][0]["model"], "model-1");
    assert_eq!(
        e.sources[scene].draft["visuals"][0]["transform"],
        e.sources[scene].draft["colliders"][0]["transform"]
    );
    e.undo(false).unwrap();
    assert_eq!(e.sources[scene].draft["visuals"][0]["model"], "model-0");
}
