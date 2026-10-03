use super::*;

#[test]
fn gameplay_loading_keeps_presentation_closed_through_render_extraction_frames() {
    let mut loading = GameplayLoadingState::default();
    loading.begin(ResourceLoadingScope::Tutorial);
    assert!(loading.visible);
    assert!(!loading.settle_render_presentation());
    assert!(!loading.settle_render_presentation());
    assert!(loading.settle_render_presentation());
    assert!(loading.visible);
    assert_eq!(loading.overall_progress, 0.99);
    assert_eq!(loading.phase_progress, 0.99);
    assert_eq!(loading.phase, GameplayLoadingPhase::RenderExtraction);

    loading.finish();
    assert!(!loading.visible);
    assert_eq!(loading.overall_progress, 1.0);
    assert_eq!(loading.phase_progress, 1.0);

    loading.begin(ResourceLoadingScope::Tutorial);
    assert!(!loading.settle_render_presentation());
    loading.loading(0.5, 0.25);
    assert!(
        !loading.settle_render_presentation(),
        "a newly observed missing tile must restart the render extraction gate"
    );
    assert_eq!(loading.overall_progress, 0.99);
    assert_eq!(loading.phase_progress, 0.99);
}

#[test]
fn gameplay_loading_reports_asset_assembly_binding_and_render_phases() {
    let mut loading = GameplayLoadingState::default();
    loading.begin(ResourceLoadingScope::Login);
    assert_eq!(loading.localized_step().key, "ui.loading.login");
    assert_eq!(
        loading.localized_current_resource().key,
        "ui.loading.current.login"
    );
    loading.begin(ResourceLoadingScope::Tutorial);
    assert_eq!(
        loading.localized_current_resource().key,
        "ui.loading.current.tutorial"
    );
    loading.prepare(GameplayLoadingPhase::SceneAssembly, 0.7, 0.4);
    assert_eq!(
        loading.localized_current_resource().key,
        "ui.loading.phase.scene"
    );
    loading.prepare(GameplayLoadingPhase::PresentationBinding, 0.8, 0.5);
    assert_eq!(
        loading.localized_current_resource().key,
        "ui.loading.phase.presentation"
    );
    loading.settle_render_presentation();
    assert_eq!(
        loading.localized_current_resource().key,
        "ui.loading.phase.render"
    );
}
