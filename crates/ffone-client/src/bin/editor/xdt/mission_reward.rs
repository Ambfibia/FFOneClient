//! Reward item domains verified against RustyFusion's load_item_data_for_type.
use super::*;
pub(super) const TYPES:&[(i64,&str)]=&[(0,"Weapon"),(1,"Shirt"),(2,"Pants"),(3,"Shoes"),(4,"Hat"),(5,"Glasses"),(6,"Backpack"),(7,"GeneralItem"),(9,"Crate"),(10,"Vehicle")];
pub(super) fn item_table(code:i64)->Option<&'static str>{
    Some(match code{0=>"m_pWeaponItemTable",1=>"m_pShirtsItemTable",2=>"m_pPantsItemTable",3=>"m_pShoesItemTable",4=>"m_pHatItemTable",5=>"m_pGlassItemTable",6=>"m_pBackItemTable",7=>"m_pGeneralItemTable",9=>"m_pChestItemTable",10=>"m_pVehicleItemTable",_=>return None})
}
pub(super) fn type_field(field:&str)->Option<&'static str>{
    match field{"m_iMissionRewardItemID"=>Some("m_iMissionRewarItemType"),"m_iMissionRewardItemID2"=>Some("m_iMissionRewardItemType2"),_=>None}
}
pub(super) fn choices(field:&str)->&'static [(i64,&'static str)]{
    if matches!(field,"m_iMissionRewarItemType"|"m_iMissionRewardItemType2"){TYPES}else{&[]}
}
pub(super) fn invalid(field:&str,value:&Value)->Option<&'static str>{
    let types=choices(field);
    if !types.is_empty()||type_field(field).is_some(){
        let Some(values)=value.as_array().filter(|v|v.len()==4)else{return Some("reward_slots")};
        return values.iter().any(|v|if types.is_empty(){v.as_i64().is_none_or(|v|!(0..=i16::MAX as i64).contains(&v))}
            else{!types.iter().any(|(id,_)|v.as_i64()==Some(*id))}).then_some("reward_value");
    }
    if matches!(field,"m_iCash"|"m_iFusionMatter"){
        return value.as_i64().is_none_or(|v|!(0..=i32::MAX as i64).contains(&v)).then_some("nonnegative");
    }
    None
}
