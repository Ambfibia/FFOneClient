use super::*;

pub const RACE_RANK_GET_LIST_REQUEST_SIZE: usize = 4;

pub const RACE_RANK_GET_DETAIL_REQUEST_SIZE: usize = 4;

pub const RACE_RANK_GET_PC_INFO_REQUEST_SIZE: usize = 56;

pub const RACE_RANK_GET_LIST_REQUEST_ABI: [RaceRankDeclaredAbiField; 1] =
    [RaceRankDeclaredAbiField {
        clean_name: "iRankListPageNum",
        offset: 0,
        byte_width: 4,
    }];

pub const RACE_RANK_GET_DETAIL_REQUEST_ABI: [RaceRankDeclaredAbiField; 1] =
    [RaceRankDeclaredAbiField {
        clean_name: "iEP_ID",
        offset: 0,
        byte_width: 4,
    }];

pub const RACE_RANK_GET_PC_INFO_REQUEST_ABI: [RaceRankDeclaredAbiField; 3] = [
    RaceRankDeclaredAbiField {
        clean_name: "iEP_ID",
        offset: 0,
        byte_width: 4,
    },
    RaceRankDeclaredAbiField {
        clean_name: "szFirstName UTF-16[9]",
        offset: 4,
        byte_width: 18,
    },
    RaceRankDeclaredAbiField {
        clean_name: "szLastName UTF-16[17]",
        offset: 22,
        byte_width: 34,
    },
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RaceRankHttpIntent {
    pub request_id: u64,
    pub url: String,
    pub pcuid: i32,
    pub ep_id: i32,
}

impl RaceRankHttpIntent {
    #[must_use]
    pub fn ordered_form_fields(&self) -> [(&'static str, String); 2] {
        [
            (RACE_RANK_HTTP_FIELD_PCUID, self.pcuid.to_string()),
            (RACE_RANK_HTTP_FIELD_EP_ID, self.ep_id.to_string()),
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceRankUiCommand {
    SelectLocationSlot(usize),
    PreviousPage,
    NextPage,
    SelectPeriod(RaceRankPeriod),
    Close,
}

#[derive(Debug, Default, Resource)]
pub struct RaceRankUiCommandOutbox(pub(super) VecDeque<RaceRankUiCommand>);

impl RaceRankUiCommandOutbox {
    pub fn push(&mut self, command: RaceRankUiCommand) {
        self.0.push_back(command);
    }

    pub fn pop(&mut self) -> Option<RaceRankUiCommand> {
        self.0.pop_front()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
