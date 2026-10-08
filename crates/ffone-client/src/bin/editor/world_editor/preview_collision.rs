use super::*;
pub(super) fn update(commands:&mut Commands,e:&WorldEditor,preview:&mut WorldPreview,root:Entity,server:&AssetServer,materials:&mut Assets<StandardMaterial>,transforms:&mut Query<&mut Transform,Without<PreviewRoot>>,desired:&mut BTreeSet<String>) {
    if !e.show_collisions { return; }
    let material = preview.collision_material.get_or_insert_with(||materials.add(StandardMaterial {
        base_color:Color::srgba(0.2,0.8,1.,0.25),alpha_mode:AlphaMode::Blend,unlit:true,cull_mode:None,..default()
    })).clone();
    let mut budget=128usize;
    for tile in e.maps.values() {
        let doc=&e.sources[tile.scene].draft;
        let Ok(root_pose)=serde_json::from_value::<ffone_client::world::AuthoredWorldTransform>(doc["root"].clone()).and_then(|t|t.try_to_bevy("collision preview").map_err(serde::de::Error::custom)) else {continue;};
        for c in doc["colliders"].as_array().into_iter().flatten().filter(|c|c["enabled"].as_bool().unwrap_or(true)) {
            let Some(model)=doc["models"].as_array().and_then(|m|m.iter().find(|m|m["id"]==c["model"])) else {continue;};
            let Some(path)=model["path"].as_str() else {continue;};
            let Ok(pose)=serde_json::from_value::<ffone_client::world::AuthoredWorldTransform>(c["transform"].clone()) else {continue;};
            let Ok(pose)=pose.try_to_bevy("collision preview") else {continue;};
            let key=format!("{}:collision:{}",tile.id,c["name"].as_str().unwrap_or_default());
            desired.insert(key.clone());
            let transform=root_pose.mul_transform(pose);
            let signature=format!("{path}:{}:{}",c["mesh"],c["primitive"]);
            if preview.signatures.get(&key)!=Some(&signature){
                if let Some(entity)=preview.objects.remove(&key){commands.entity(entity).insert(Visibility::Hidden);preview.retiring.push(entity);}
                preview.signatures.insert(key.clone(),signature);
            }
            if let Some(entity)=preview.objects.get(&key) {
                if let Ok(mut current)=transforms.get_mut(*entity){*current=transform;}
            } else {
                if budget==0{preview.pending=true;continue;}budget-=1;
                let entity=commands.spawn((Name::new(key.clone()),Mesh3d(server.load(GltfAssetLabel::Primitive {mesh:c["mesh"].as_u64().unwrap_or(0) as usize,primitive:c["primitive"].as_u64().unwrap_or(0) as usize}.from_asset(path.to_owned()))),MeshMaterial3d(material.clone()),transform,Visibility::Inherited,ChildOf(root))).id();
                preview.objects.insert(key,entity);
            }
        }
    }
}
