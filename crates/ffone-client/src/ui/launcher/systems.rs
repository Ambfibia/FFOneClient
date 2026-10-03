use super::*;

pub const LAUNCHER_UI_ROTATE_SPEED_PER_FIXED_UPDATE: f32 = 1.600_000_023_841_858;

pub(super) fn apply_launcher_fixed_aim(
    external: Res<LauncherUiExternalState>,
    mut model: ResMut<LauncherUiModel>,
) {
    model.fixed_update_aim(external.aim_vertical_axis, external.aim_horizontal_axis);
}

pub(super) fn sync_launcher_labels(
    labels: Res<LauncherUiLabels>,
    mut texts: Query<(&LauncherUiTextRole, &mut LocalizedText)>,
) {
    if !labels.is_changed() {
        return;
    }
    for (role, mut localized) in &mut texts {
        let next = match role {
            LauncherUiTextRole::Power => labels.power_localized(),
            LauncherUiTextRole::Tip => labels.tip_localized(),
        };
        if *localized != next {
            *localized = next;
        }
    }
}
