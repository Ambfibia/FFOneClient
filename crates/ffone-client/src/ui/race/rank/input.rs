use super::*;

#[must_use]
pub fn parse_clean_rank_response(body: &str) -> Result<RaceRankScores, RaceRankParseError> {
    let mut scores = RaceRankScores::default();
    if !body.contains("SUCCESS") {
        return Ok(scores);
    }
    for (index, (personal_name, top_name)) in [
        ("myday", "day"),
        ("myweek", "week"),
        ("mymonth", "month"),
        ("myalltime", "alltime"),
    ]
    .into_iter()
    .enumerate()
    {
        let personal_section = section(body, personal_name)?;
        scores.personal[index] = score_tags(personal_section)?.into_iter().next();
        let top_section = section(body, top_name)?;
        for (slot, score) in score_tags(top_section)?
            .into_iter()
            .take(RACE_RANK_TOP_COUNT)
            .enumerate()
        {
            scores.top[index][slot] = Some(score);
        }
    }
    Ok(scores)
}

pub(super) fn find_score_open(value: &str) -> Option<usize> {
    let mut cursor = 0;
    while let Some(relative) = value[cursor..].find("<score") {
        let start = cursor + relative;
        let suffix = value.as_bytes().get(start + "<score".len()).copied();
        if matches!(suffix, Some(b' ' | b'\t' | b'\r' | b'\n' | b'>')) {
            return Some(start);
        }
        cursor = start + "<score".len();
    }
    None
}

pub(super) fn parse_score_tag(tag: &str) -> Result<RaceRankScore, RaceRankParseError> {
    let mut cursor = 0;
    let mut previous = "";
    let pcuid = ordered_attribute(tag, "PCUID", previous, &mut cursor)?;
    previous = "PCUID";
    let score = ordered_attribute(tag, "Score", previous, &mut cursor)?;
    previous = "Score";
    let rank = ordered_attribute(tag, "Rank", previous, &mut cursor)?;
    previous = "Rank";
    let first = ordered_attribute(tag, "FirstName", previous, &mut cursor)?;
    previous = "FirstName";
    let last = ordered_attribute(tag, "LastName", previous, &mut cursor)?;
    Ok(RaceRankScore {
        pcuid: parse_rank_integer("PCUID", pcuid)?,
        rank: parse_rank_integer("Rank", rank)?,
        player: format!("{first} {last}"),
        score: parse_rank_integer("Score", score)?,
    })
}

pub(super) fn parse_rank_integer(field: &'static str, value: &str) -> Result<i32, RaceRankParseError> {
    value
        .parse()
        .map_err(|_| RaceRankParseError::InvalidInteger {
            field,
            value: value.to_owned(),
        })
}
