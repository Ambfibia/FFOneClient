use super::*;

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry=entry.unwrap();let target=to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir(){copy_tree(&entry.path(),&target);}else{fs::copy(entry.path(),target).unwrap();}
    }
}
fn copy_shared(value: &Value, from: &Path, to: &Path) {
    match value {
        Value::String(path) if path.starts_with("map/shared/")&&from.join(path).is_file()=>{
            let target=to.join(path);fs::create_dir_all(target.parent().unwrap()).unwrap();fs::copy(from.join(path),target).unwrap();
        }
        Value::Array(values)=>for value in values{copy_shared(value,from,to);},
        Value::Object(values)=>for value in values.values(){copy_shared(value,from,to);},
        _=>{}
    }
}
fn terrain_fixture()->(tempfile::TempDir,WorldEditor){
    let (dir,mut e)=fixture();let published=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let rel="map/tiles/map_12_03";copy_tree(&published.join(rel),&e.root.join(rel));
    copy_shared(&terrain::read_source(&e.root.join(format!("{rel}/terrain/terrain.json"))).unwrap(),&published,&e.root);
    model::write_atomic(&e.root.join("map/catalog.json"),&json!({"tiles":[]})).unwrap();
    e.scan_tiles();e.center=atlas::tile_center([12,3],0.);e.load_tile().unwrap();(dir,e)
}

#[test]
fn new_native_square_matches_neighbour_edges_and_survives_undo_restart_and_publish(){
    let (_dir,mut e)=terrain_fixture();let id="map_13_03";let point=atlas::tile_center([13,3],0.);
    let edge=Vec3::new(-13.*512.,0.,3.5*512.);let expected=e.ground["map_12_03"].height_at(edge).unwrap();
    e.create_terrain(point).unwrap();assert_eq!(e.undo.len(),1);
    assert!((e.ground_height(edge).unwrap()-expected).abs()<0.025);
    let samples=e.ground[id].samples.clone();let weights=e.ground[id].weights.clone();
    assert!(weights.iter().flatten().any(|w|*w>0));
    assert!(!e.root.join(format!("map/tiles/{id}/scene.json")).exists());
    e.undo(false).unwrap();assert!(!e.ground.contains_key(id));assert!(!e.available_tiles.contains(&[13,3]));
    e.undo(true).unwrap();assert_eq!(e.ground[id].samples,samples);
    e.save_work().unwrap();let mut restored=WorldEditor::open(e.root.clone());
    restored.restore_work().unwrap();
    assert_eq!(restored.ground[id].samples,samples);assert_eq!(restored.ground[id].weights,weights);
    restored.publish().unwrap();let scene_path=restored.root.join(format!("map/tiles/{id}/scene.json"));
    let scene=ffone_client::world::NativeWorldScene::from_json_slice(&fs::read(&scene_path).unwrap()).unwrap();
    let instance=scene.native_terrain.unwrap();
    let native=ffone_client::native_terrain::NativeTerrain::open(&restored.root,&instance.path,&instance.blake3).unwrap();
    assert_eq!(native.height_samples(),samples.as_slice());
    assert_eq!(native.editor_weight_mips().iter().map(|chain|chain[0].2.clone()).collect::<Vec<_>>(),weights);
    let disk=ground::Tile::load(&restored.root,id).unwrap();assert!((disk.height_at(edge).unwrap()-expected).abs()<0.025);
}

#[test]
fn terrain_creation_in_unloaded_existing_square_preserves_objects_and_settings(){
    let (_dir,mut e)=terrain_fixture();let id="map_13_03";
    let mut scene=terrain::read_source(&e.root.join("map/tiles/map_12_03/scene.json")).unwrap();
    scene.as_object_mut().unwrap().remove("nativeTerrain");scene["name"]=json!(id);scene["tile"]=json!([13,3]);
    scene["squareSettings"]=json!({"location":"New town","music":"stop","skybox":"past","terrainShader":null});
    let folder=e.root.join(format!("map/tiles/{id}"));fs::create_dir_all(&folder).unwrap();
    model::write_atomic(&folder.join("scene.json"),&scene).unwrap();
    let objects=json!({"tileId":id,"objects":[{"sourceNode":"keep#1","unknown":17}]});
    model::write_atomic(&folder.join("objects.json"),&objects).unwrap();
    model::write_atomic(&folder.join("tile.json"),&json!({"id":id,"files":[]})).unwrap();
    e.create_terrain(atlas::tile_center([13,3],0.)).unwrap();
    let object_source=e.maps[id].objects;let scene_source=e.maps[id].scene;assert_eq!(e.sources[object_source].draft,objects);assert_eq!(e.sources[scene_source].draft["squareSettings"],scene["squareSettings"]);
    e.undo(false).unwrap();assert_eq!(e.sources[scene_source].draft,scene);
}

#[test]
fn collision_toggle_and_square_settings_undo_restore_original_defaults(){
    let (_dir,mut e)=fixture();map_fixture(&e,[0,0]);
    model::write_atomic(&e.root.join("map/catalog.json"),&json!({})).unwrap();e.center=atlas::tile_center([0,0],3.);e.load_tile().unwrap();
    let index=e.entities.iter().position(|p|p.kind==4).unwrap();e.select_entity(index);let p=e.selected().unwrap().clone();
    assert!(e.object_collision(&p));e.toggle_object_collision().unwrap();assert!(!e.object_collision(&p));
    e.undo(false).unwrap();assert!(e.object_collision(&p));e.undo(true).unwrap();assert!(!e.object_collision(&p));
    e.change_square("location",json!("Custom town")).unwrap();assert_eq!(e.location_name(e.center),"Custom town");
    e.undo(false).unwrap();assert_ne!(e.location_name(e.center),"Custom town");
    e.change_square("music",json!("stop")).unwrap();assert_eq!(e.square_settings()["music"],"stop");
    assert!(e.change_square("skybox",json!("invalid")).is_err());
}

#[test]
fn grass_brush_places_chosen_models_with_one_undo_and_no_collision(){
    let (_dir,mut e)=fixture();map_fixture(&e,[0,0]);
    model::write_atomic(&e.root.join("map/catalog.json"),&json!({})).unwrap();e.center=atlas::tile_center([0,0],3.);e.load_tile().unwrap();
    let p=e.entities.iter().find(|p|p.kind==4).unwrap().clone();let mut template=e.snapshot(&p).unwrap();template.parts.remove("colliders");
    e.grass_template=Some(template);e.brush_strength=3.;e.brush_radius=4.;
    let source=e.maps["map_00_00"].objects;let scene=e.maps["map_00_00"].scene;
    e.grass_brush(e.center).unwrap();e.grass_brush(e.center+Vec3::Z*8.).unwrap();e.finish_grass();
    assert_eq!(e.sources[source].draft["objects"].as_array().unwrap().len(),7);
    assert_eq!(e.sources[scene].draft["colliders"].as_array().unwrap().len(),1);assert_eq!(e.undo.len(),1);
    e.undo(false).unwrap();assert_eq!(e.sources[source].draft["objects"].as_array().unwrap().len(),1);
    e.undo(true).unwrap();assert_eq!(e.sources[source].draft["objects"].as_array().unwrap().len(),7);
}
