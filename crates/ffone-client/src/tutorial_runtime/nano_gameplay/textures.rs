use super::*;

#[derive(Debug, Component)]
pub(super) struct TutorialGameplayNanoFaceTextureBound;

pub(super) fn bind_tutorial_gameplay_nano_face_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<TutorialNanoGameplayState>,
    parents: Query<&ChildOf>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    mut surfaces: Query<
        (
            Entity,
            &mut MeshMaterial3d<LegacyModelMaterial>,
            &PendingLegacyModelMaterial,
        ),
        Without<TutorialGameplayNanoFaceTextureBound>,
    >,
    mut issues: ResMut<TutorialNanoGameplayIssueQueue>,
    mut events: ResMut<TutorialNanoGameplayEventQueue>,
) {
    if !state.asset_contract_ready {
        return;
    }
    let Some(root) = state.entity else {
        return;
    };
    for (entity, mut handle, metadata) in &mut surfaces {
        if metadata.true_name != TUTORIAL_NANO_FACE_MATERIAL_NAME
            || !is_descendant_of(entity, root, &parents)
        {
            continue;
        }
        let face_texture = match load_legacy_main_texture_replacement(
            &asset_server,
            metadata,
            TUTORIAL_NANO_FACE_TEXTURE_PATH,
        ) {
            Ok(texture) => texture,
            Err(error) => {
                let error = format!("gameplay Nano face texture override is not exact: {error}");
                let was_active = state.is_active();
                let owner = state.owner;
                state.status = TutorialNanoGameplayStatus::Blocked(error.clone());
                state.entity = None;
                state.owner = None;
                state.activated_generation = None;
                state.asset_contract_ready = false;
                state.face_texture_bound = false;
                state.animation.clear();
                state.applied_animation_request_serial = None;
                commands.entity(root).despawn();
                issues.push(TutorialNanoGameplayIssue::AssetBlocked(error));
                if was_active {
                    events.push(TutorialNanoGameplayEvent::Dismissed {
                        owner,
                        entity: root,
                    });
                }
                return;
            }
        };
        crate::legacy_model_material::make_legacy_material_unique(&mut handle.0, &mut materials);
        let Some(mut material) = materials.get_mut(&handle.0) else {
            continue;
        };
        material.base_texture = Some(face_texture);
        commands
            .entity(entity)
            .insert(TutorialGameplayNanoFaceTextureBound);
        state.face_texture_bound = true;
    }
}
