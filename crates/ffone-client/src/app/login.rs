//! Client configuration, credentials, login loading and login UI effects.

use super::loading_screen::{GameplayLoadingState, ResourceLoadingScope};
use super::open_external_url;
use super::runtime_status::RuntimeStatus;
use super::state::ClientState;
use bevy::prelude::*;
use ffone_client::{
    character_creation_data::CharacterCreationData,
    localization::{Language, Localization, LocalizedText},
    login_ui::{
        LoginSurface, LoginUiAssetStatus, LoginUiEffect, LoginUiEffectOutbox, LoginUiModel,
        LoginUiOutbox, LoginBrowser,
    },
    network::{NetworkBridge, NetworkCommand},
    system_message_ui::{SystemMessageButtonType, SystemMessageRequest, SystemMessageUiModel},
};
use std::{env, path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Resource)]
pub(super) struct ClientConfig {
    pub(super) asset_root: PathBuf,
    pub(super) asset_root_explicit: bool,
    pub(super) login_address: String,
    pub(super) login_user: Option<String>,
    pub(super) password_env: String,
    pub(super) character_uid: Option<i64>,
    pub(super) character_asset_root: PathBuf,
    pub(super) character_asset_root_explicit: bool,
    pub(super) character_model: String,
    pub(super) character_root: String,
    pub(super) character_animation: String,
    pub(super) validate_assets: bool,
    pub(super) network_smoke: bool,
    pub(super) language: String,
    pub(super) language_command_line: Option<String>,
    pub(super) language_environment: Option<String>,
}

#[derive(Resource)]
pub(super) struct LoadedCharacterCreationData(pub(super) Arc<CharacterCreationData>);

#[derive(Clone)]
pub(super) struct Credentials {
    pub(super) username: String,
    pub(super) password: String,
    pub(super) cookie: bool,
}

#[derive(Default, Resource)]
pub(super) struct ActiveLoginCredentials {
    pub(super) credentials: Option<Credentials>,
}

#[derive(Resource)]
pub(super) struct PendingLogin {
    pub(super) credentials: Option<Credentials>,
    pub(super) note: String,
}

impl PendingLogin {
    pub(super) fn from_environment(config: &ClientConfig) -> Self {
        let username = config
            .login_user
            .clone()
            .or_else(|| env::var("FFONE_USERNAME").ok())
            .filter(|value| !value.is_empty());
        let Some(username) = username else {
            return Self {
                credentials: None,
                note: "Enter your account name and password.".to_owned(),
            };
        };
        match env::var(&config.password_env) {
            Ok(password) if !password.is_empty() => Self {
                credentials: Some(Credentials { username, password, cookie: false }),
                note: "OpenFusion auto-login queued".to_owned(),
            },
            _ => Self {
                credentials: None,
                note: format!("Enter the password for {username}."),
            },
        }
    }
}

/// Maps only lifecycle state that production actually owns. Clean
/// `CnLoginMode.AutoLogin` sets `bAutoLogin` before submitting saved
/// credentials; FFOne's startup `PendingLogin` is the sole corresponding
/// automatic-credential context. Clean WebLogin requires its browser cookie
/// callback and WarpShard requires `ReceiveWarpShard`; neither producer exists
/// in the native production lifecycle, so those standalone UI branches remain
/// deliberately unreachable here.
pub(super) fn login_surface_for_startup_context(pending: &PendingLogin) -> LoginSurface {
    if pending.credentials.is_some() {
        LoginSurface::AutoLogin
    } else {
        LoginSurface::Manual
    }
}

pub(super) fn begin_login(
    config: Res<ClientConfig>,
    mut pending: ResMut<PendingLogin>,
    mut active_credentials: ResMut<ActiveLoginCredentials>,
    bridge: Res<NetworkBridge>,
    mut runtime: ResMut<RuntimeStatus>,
    mut login_ui: ResMut<LoginUiModel>,
    mut loading: ResMut<GameplayLoadingState>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    loading.begin(ResourceLoadingScope::Login);
    runtime.message.clone_from(&pending.note);
    login_ui.surface = login_surface_for_startup_context(&pending);
    login_ui.visible = true;
    login_ui.status.clone_from(&pending.note);
    next_state.set(ClientState::Login);
    let Some(credentials) = pending.credentials.take() else {
        login_ui.username = config
            .login_user
            .clone()
            .or_else(|| env::var("FFONE_USERNAME").ok())
            .unwrap_or_default();
        login_ui.busy = false;
        return;
    };
    login_ui.username.clone_from(&credentials.username);
    active_credentials.credentials = Some(credentials.clone());
    login_ui.busy = true;
    login_ui.status = format!("Connecting to {}...", config.login_address);
    if let Err(error) = bridge.send(NetworkCommand::Login {
        login_address: config.login_address.clone(),
        username: credentials.username,
        password: credentials.password,
    }) {
        runtime.message.clone_from(&error);
        login_ui.status = error;
        login_ui.busy = false;
    }
}

pub(super) fn gate_login_loading(
    state: Res<State<ClientState>>,
    assets: Res<LoginUiAssetStatus>,
    mut loading: ResMut<GameplayLoadingState>,
) {
    if *state.get() != ClientState::Login || loading.scope != Some(ResourceLoadingScope::Login) {
        return;
    }
    match &*assets {
        LoginUiAssetStatus::Failed { path, error } => {
            loading.block(format!("login asset {path} failed: {error}"));
        }
        LoginUiAssetStatus::Loading { completed, total } => {
            let progress = *completed as f32 / (*total).max(1) as f32;
            loading.loading(progress, progress);
        }
        LoginUiAssetStatus::Ready => {
            if loading.settle_render_presentation() {
                loading.finish();
            }
        }
    }
}

pub(super) fn handle_login_ui_requests(
    mut config: ResMut<ClientConfig>,
    bridge: Res<NetworkBridge>,
    mut browser: ResMut<LoginBrowser>,
    mut active_credentials: ResMut<ActiveLoginCredentials>,
    mut outbox: ResMut<LoginUiOutbox>,
    mut runtime: ResMut<RuntimeStatus>,
    mut login_ui: ResMut<LoginUiModel>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    for request in outbox.drain() {
        login_ui.surface = LoginSurface::Manual;
        runtime.message = format!("Connecting to {}...", browser.selected_address());
        login_ui.status.clone_from(&runtime.message);
        browser.start_login(request.username, request.password);
    }
    let Some(result) = browser.take_login() else { return; };
    let (address, username, password) = match result {
        Ok(wire) => wire,
        Err(error) => {
            runtime.message = format!("Login failed: {error}");
            login_ui.status.clone_from(&runtime.message);
            login_ui.busy = false;
            return;
        }
    };
    config.login_address = address;
    next_state.set(ClientState::Login);
    active_credentials.credentials = Some(Credentials { username: username.clone(), password: password.clone(), cookie: browser.servers[browser.selected].api.is_some() });
    let command = if browser.servers[browser.selected].api.is_some() {
        NetworkCommand::LoginCookie { login_address: config.login_address.clone(), username, cookie: password }
    } else { NetworkCommand::Login { login_address: config.login_address.clone(), username, password } };
    if let Err(error) = bridge.send(command) {
        runtime.message = format!("Login failed: {error}");
        login_ui.status.clone_from(&runtime.message);
        login_ui.busy = false;
        browser.finish_login(false);
    }
}

pub(super) const LOGIN_REGISTRATION_MESSAGE_REQUEST_ID: u64 = 0x4c4f_4749_4e52_4547;
pub(super) const LOGIN_EXTERNAL_LINK_ERROR_REQUEST_ID: u64 = 0x4c4f_4749_4e55_524c;
pub(super) const LOGIN_EXTERNAL_LINK_ERROR_KEY: &str = "ui.login.external_link_error";
pub(super) const LOGIN_EXTERNAL_LINK_ERROR_FALLBACK: &str = "Unable to open the community page.\n{error}";

pub(super) fn localized_login_external_link_error(error: impl Into<String>) -> LocalizedText {
    LocalizedText::new(
        LOGIN_EXTERNAL_LINK_ERROR_KEY,
        LOGIN_EXTERNAL_LINK_ERROR_FALLBACK,
    )
    .with_arg("error", error)
}

pub(super) fn consume_login_ui_effects(
    mut outbox: ResMut<LoginUiEffectOutbox>,
    mut system_messages: ResMut<SystemMessageUiModel>,
    localization: Res<Localization>,
    language: Res<Language>,
) {
    for effect in outbox.drain() {
        match effect {
            LoginUiEffect::OpenCommunity { url } => {
                if let Err(error) = open_external_url(url) {
                    system_messages.push(SystemMessageRequest::new_localized(
                        LOGIN_EXTERNAL_LINK_ERROR_REQUEST_ID,
                        localized_login_external_link_error(error),
                        SystemMessageButtonType::Ok,
                    ));
                }
            }
            LoginUiEffect::ShowRegistrationInstructions { message } => {
                let message = localization.text(
                    &language,
                    &LocalizedText::new("ui.login.registration_instructions", message),
                );
                system_messages.push(SystemMessageRequest::new(
                    LOGIN_REGISTRATION_MESSAGE_REQUEST_ID,
                    message,
                    SystemMessageButtonType::Ok,
                ));
            }
        }
    }
}

pub(super) fn sync_login_ui(
    mut browser: ResMut<LoginBrowser>,
    runtime: Res<RuntimeStatus>,
    state: Res<State<ClientState>>,
    system_messages: Res<SystemMessageUiModel>,
    mut model: ResMut<LoginUiModel>,
) {
    let visible = *state.get() == ClientState::Login;
    if *state.get() == ClientState::CharacterSelect { browser.finish_login(true); }
    if model.visible != visible {
        model.visible = visible;
        if !visible {
            model.password.clear();
            model.busy = false;
            // The native plugin survives state changes, unlike the clean
            // LoginMode scene object. Restore that object's serialized owner
            // defaults before any later production re-entry.
            model.surface = LoginSurface::Manual;
        }
    }
    if runtime.is_changed() {
        model.status.clone_from(&runtime.message);
        if runtime.message.starts_with("Network error")
            || runtime.message.starts_with("Login failed")
            || runtime.message.starts_with("Disconnected")
            || runtime.message.starts_with("Offline")
        {
            model.busy = false;
            browser.finish_login(false);
        }
    }
    model.system_popup_active = system_messages.current().is_some();
}
