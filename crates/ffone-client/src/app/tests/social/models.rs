use super::*;

#[test]
fn ordinary_nanocom_routes_quit_opens_only_an_idle_model_without_network_destination() {
    let runtime = QuitMenuRuntime::default();
    let mut blocked = QuitMenuUiModel::default();
    assert!(!open_quit_menu_from_nanocom(&mut blocked, &runtime, true));
    assert!(!blocked.visible);

    let mut model = QuitMenuUiModel::default();
    assert!(open_quit_menu_from_nanocom(&mut model, &runtime, false));
    assert!(model.visible);
    assert_eq!(runtime.pending(), None);
    assert!(!open_quit_menu_from_nanocom(&mut model, &runtime, false));

    let mut waiting_runtime = QuitMenuRuntime::default();
    assert!(waiting_runtime.begin(QuitMenuDestination::QuitGame));
    let mut waiting_model = QuitMenuUiModel::default();
    assert!(!open_quit_menu_from_nanocom(
        &mut waiting_model,
        &waiting_runtime,
        false,
    ));
    assert!(!waiting_model.visible);
}
