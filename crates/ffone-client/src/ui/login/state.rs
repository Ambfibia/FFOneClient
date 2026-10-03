use super::*;

pub const LOGIN_SOURCE_RUNTIME: &str = "fusion-2.x.x";

pub const LOGIN_STATUS_ADAPTER_Y_OFFSET: f32 = 0.0;

pub(super) fn login_background_mode(
    surface: LoginSurface,
    loaded_background_assigned: bool,
) -> LoginBackgroundMode0104 {
    match surface {
        LoginSurface::AutoLogin => LoginBackgroundMode0104::Black,
        LoginSurface::WarpShard => LoginBackgroundMode0104::FallbackScaleToFit,
        LoginSurface::Manual
        | LoginSurface::WaitingForWebAuthentication
        | LoginSurface::WebAuthentication => {
            if loaded_background_assigned {
                LoginBackgroundMode0104::LoadedScaleAndCrop
            } else {
                LoginBackgroundMode0104::FallbackScaleToFit
            }
        }
    }
}

#[derive(Default, Resource)]
pub(super) struct LoginLoadedBackgroundState {
    pub(super) assigned: bool,
}

#[derive(Component)]
pub(super) struct LoginStatusText;
