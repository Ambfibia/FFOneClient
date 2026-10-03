use super::*;

pub const RACE_COPY_TIME: &str = "TIME";

pub const RACE_COPY_PODS: &str = "PODS";

pub const RACE_COPY_SCORE: &str = "SCORE";

pub const RACE_COPY_MY_BEST: &str = "MY BEST RECORD";

pub const RACE_COPY_REWARD: &str = "REWARD:";

pub const RACE_COPY_LEVEL: &str = "LEVEL";

pub const RACE_COPY_INVENTORY_FULL: &str = "INVENTORY FULL!";

pub const RACE_COPY_FUSION_MATTER: &str = "FUSION MATTER";

pub const RACE_COPY_ACCEPT: &str = "ACCEPT";

pub(super) fn write_i32(payload: &mut [u8], offset: usize, value: i32) {
    payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[must_use]
pub fn race_time_copy(total_seconds: i32) -> String {
    let hours = total_seconds / 3_600;
    let minutes = total_seconds / 60 % 60;
    let seconds = total_seconds % 60;
    format!("{hours}:{minutes:02}:{seconds:02}")
}

/// Matches the invariant `"{0:N}"` copy produced for an integer by the clean
/// client: comma groups and two decimal places.
#[must_use]
pub fn race_score_copy(value: i32) -> String {
    let negative = value < 0;
    let digits = i64::from(value).unsigned_abs().to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3 + 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(character);
    }
    if negative {
        grouped.insert(0, '-');
    }
    grouped.push_str(".00");
    grouped
}

#[must_use]
pub const fn race_rank_copy(rank: i32) -> &'static str {
    match race_filled_star_count(rank) {
        5 => "Genius!",
        4 => "Awesome!",
        3 => "Good",
        2 => "Not Bad",
        1 => "Bleh!",
        _ => "",
    }
}
