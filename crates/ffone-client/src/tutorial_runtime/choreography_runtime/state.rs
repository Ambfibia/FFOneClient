use super::*;

pub struct TutorialChoreographyRuntimePlugin;

impl Plugin for TutorialChoreographyRuntimePlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<TutorialChoreographyPlayer>()
            .init_resource::<TutorialChoreographyPresentation>()
            .init_resource::<TutorialChoreographyIssueQueue>()
            .init_resource::<TutorialPanAssets>()
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_tutorial_choreography_overlay,
            )
            .add_systems(
                Update,
                ((sync_tutorial_choreography_overlay, sync_tutorial_pan_strip))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
