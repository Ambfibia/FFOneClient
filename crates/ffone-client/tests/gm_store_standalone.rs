#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{localization, ui_startup};
#[path = "../src/gameplay/user_store/mod.rs"]
mod user_store_runtime;
#[path = "../src/ui/user_store/mod.rs"]
pub mod user_store_ui;

#[test]
fn gm_session_retains_ui_ownership_and_reset_revokes_it() {
    use user_store_runtime::UserStoreProductionRuntime0104;
    use user_store_ui::*;
    let mut runtime = UserStoreProductionRuntime0104::default();
    runtime.gm_owner = Some(81);
    let mut state = UserStoreUiState0104 {
        active: true,
        ..Default::default()
    };
    let mut popup = UserStorePopupPresentation0104::default();
    let mut outbox = UserStoreUiOutbox0104::default();
    outbox.push(UserStoreUiCommand0104::SendPacket(
        UserStorePacket0104::ready(49),
    ));
    assert!(
        runtime
            .guard_unowned_ui(&mut state, &mut popup, &mut outbox)
            .is_none()
    );
    assert!(state.active);
    assert_eq!(outbox.0.len(), 1);
    runtime.reset();
    assert!(
        runtime
            .guard_unowned_ui(&mut state, &mut popup, &mut outbox)
            .is_some()
    );
    assert!(!state.active);
    assert!(outbox.0.is_empty());
}
