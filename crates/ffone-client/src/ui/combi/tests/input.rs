use super::*;

#[test]
fn non_combinable_inventory_items_still_resolve_their_icons() {
    struct IconOnly;
    impl CombiItemCatalog0104 for IconOnly {
        fn resolve(&self,_:i16,_:i16)->Option<CombiItemMetadata0104>{None}
        fn resolve_icon(&self,ty:i16,id:i16)->Option<String>{Some(format!("icons/{ty}/{id}.png"))}
        fn enable_equip(&self,_:ItemBase0104)->Option<bool>{Some(false)}
        fn enable_equip_combi(&self,_:ItemBase0104)->Option<bool>{Some(false)}
    }
    let (mut snapshot,_,_,_)=fixture();
    for ty in [4,5,6,7,9,10] {
        let item=item(ty,100,0,0);snapshot.inventory[0]=item;
        assert_eq!(project_inventory_slot(item,false,&IconOnly).icon_path,Some(format!("icons/{ty}/100.png")));
        assert!(project_look(&snapshot,0,&IconOnly).unwrap().icon_path.is_some());
        assert!(project_stats(&snapshot,0,&IconOnly).unwrap().icon_path.is_some());
    }
}
