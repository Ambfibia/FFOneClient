use super::*;

#[test]
fn clean_pc_stuff_new_inventory_rects_and_texture_owners_are_exact() {
    assert_eq!(CASHMALL_NANO_TAB_TEXTURE_PATH_ID, 98);
    assert_eq!(CASHMALL_NANO_TAB_HOVER_TEXTURE_PATH_ID, 299);
    assert_eq!(CASHMALL_DEXLABS_TEXTURE_PATH_ID, 451);
    assert_eq!(CASHMALL_TAROS_COUNTER_TEXTURE_PATH_ID, 326);
    assert_eq!(
        CASHMALL_PC_STUFF_NANO_TAB_RECT,
        CashmallUiRect0104::new(96.0, 0.0, 129.0, 29.0)
    );
    assert_eq!(
        CASHMALL_PC_STUFF_NANO_TAB_HIT_RECT,
        CashmallUiRect0104::new(135.0, 5.0, 60.0, 15.0)
    );
    assert_eq!(
        CASHMALL_PC_STUFF_DEXLABS_RECT,
        CashmallUiRect0104::new(170.0, 561.0, 203.0, 67.0)
    );
    assert_eq!(
        CASHMALL_PC_STUFF_TAROS_COUNTER_RECT,
        CashmallUiRect0104::new(20.0, 560.0, 149.0, 32.0)
    );
    assert_eq!(
        CASHMALL_PC_STUFF_REDEEM_CODE_RECT,
        CashmallUiRect0104::new(15.0, 598.0, 149.0, 25.0)
    );
}
