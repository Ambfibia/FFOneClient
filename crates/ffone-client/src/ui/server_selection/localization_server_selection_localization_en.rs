use super::*;

pub const SERVER_SELECTION_US_LOCALE_VALUE: i32 = 0;

pub const SERVER_SELECTION_KOREA_LOCALE_VALUE: i32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerSelectionLocalizationEntry {
    pub key: &'static str,
    pub en: &'static str,
    pub ru: &'static str,
}

/// Bundle entries for every string painted by the reachable clean `OnGUI`.
/// Packet diagnostics, event-scene identifiers and system-message 250 remain
/// outside this presentation tree and are intentionally not treated as copy.
pub const SERVER_SELECTION_LOCALIZATION_ENTRIES: [ServerSelectionLocalizationEntry; 14] = [
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_HEADER_SERVER_CHANNEL_KEY,
        en: SERVER_SELECTION_COPY_HEADER_SERVER_CHANNEL,
        ru: "Сервер / Канал",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_HEADER_STATUS_KEY,
        en: SERVER_SELECTION_COPY_HEADER_STATUS,
        ru: "Статус",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_SERVER_COLLAPSED_KEY,
        en: SERVER_SELECTION_COPY_SERVER_COLLAPSED,
        ru: "+ Сервер {server} ----------------",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_SERVER_EXPANDED_KEY,
        en: SERVER_SELECTION_COPY_SERVER_EXPANDED,
        ru: "- Сервер {server} ----------------",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_CHANNEL_ROW_KEY,
        en: SERVER_SELECTION_COPY_CHANNEL_ROW,
        ru: "- Канал {channel}  -------",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_STATUS_NONE_KEY,
        en: "",
        ru: "",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_STATUS_CLOSED_KEY,
        en: SERVER_SELECTION_COPY_STATUS_CLOSED,
        ru: "ЗАКРЫТ",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_STATUS_EMPTY_KEY,
        en: SERVER_SELECTION_COPY_STATUS_EMPTY,
        ru: "ПУСТО",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_STATUS_NORMAL_KEY,
        en: SERVER_SELECTION_COPY_STATUS_NORMAL,
        ru: "НОРМА",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_STATUS_BUSY_KEY,
        en: SERVER_SELECTION_COPY_STATUS_BUSY,
        ru: "ЗАНЯТ",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_CONNECT_KEY,
        en: SERVER_SELECTION_COPY_CONNECT,
        ru: "Подключиться",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_MY_ACCOUNT_KEY,
        en: SERVER_SELECTION_COPY_MY_ACCOUNT,
        ru: "Мой аккаунт",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_HOMEPAGE_KEY,
        en: SERVER_SELECTION_COPY_HOMEPAGE,
        ru: "Главная",
    },
    ServerSelectionLocalizationEntry {
        key: SERVER_SELECTION_QUIT_KEY,
        en: SERVER_SELECTION_COPY_QUIT,
        ru: "Выйти",
    },
];

#[must_use]
pub fn server_selection_server_heading_localized(expanded: bool, server: u8) -> LocalizedText {
    let (key, fallback) = if expanded {
        (
            SERVER_SELECTION_SERVER_EXPANDED_KEY,
            SERVER_SELECTION_COPY_SERVER_EXPANDED,
        )
    } else {
        (
            SERVER_SELECTION_SERVER_COLLAPSED_KEY,
            SERVER_SELECTION_COPY_SERVER_COLLAPSED,
        )
    };
    LocalizedText::new(key, fallback).with_arg("server", server.to_string())
}

#[must_use]
pub fn server_selection_channel_localized(channel: u8) -> LocalizedText {
    LocalizedText::new(
        SERVER_SELECTION_CHANNEL_ROW_KEY,
        SERVER_SELECTION_COPY_CHANNEL_ROW,
    )
    .with_arg("channel", channel.to_string())
}

pub(super) fn initial_server_selection_localized(role: ServerSelectionUiTextRole) -> LocalizedText {
    match role {
        ServerSelectionUiTextRole::ServerHeading => {
            server_selection_server_heading_localized(false, 1)
        }
        ServerSelectionUiTextRole::ServerStatus => ServerSelectionPopulation::Empty.localized(),
        ServerSelectionUiTextRole::ShardStatus(_) => ServerSelectionPopulation::Closed.localized(),
    }
}

pub(super) fn localized_fallback(localized: &LocalizedText) -> String {
    localized
        .args
        .iter()
        .fold(localized.fallback.clone(), |text, (name, value)| {
            text.replace(&format!("{{{name}}}"), value)
        })
}
