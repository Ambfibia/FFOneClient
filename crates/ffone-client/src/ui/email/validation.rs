use super::*;

pub const EMAIL_REQ_UPDATE_CHECK_ID: u32 = 0x1300_007B;

pub const EMAIL_REQ_UPDATE_CHECK_SIZE: usize = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmailComposeError {
    MissingRecipient,
    NegativeCash(i32),
    InsufficientCash { available: i32, required: i32 },
    SubjectWireOverflow,
    ContentWireOverflow,
}

impl fmt::Display for EmailComposeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRecipient => formatter.write_str("email recipient is not selected"),
            Self::NegativeCash(value) => {
                write!(formatter, "email cash attachment is negative: {value}")
            }
            Self::InsufficientCash {
                available,
                required,
            } => write!(
                formatter,
                "email requires {required} Taros but only {available} are available"
            ),
            Self::SubjectWireOverflow => {
                formatter.write_str("email subject exceeds the UTF-16 wire field")
            }
            Self::ContentWireOverflow => {
                formatter.write_str("email content exceeds the UTF-16 wire field")
            }
        }
    }
}

impl Error for EmailComposeError {}
