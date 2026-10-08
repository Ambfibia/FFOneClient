//! Native overrides keep mail and Nano-Com independent when both channels are used.
use super::*;
pub(super) const PAIRS: &[(&str,&str,&str,&str,&str)] = &[
    ("start","m_iSTMessageType","m_iSTMessageTextID","m_iSTEmailTextID","m_iSTEmailSendNPC"),
    ("success","m_iSUMessageType","m_iSUMessagetextID","m_iSUEmailTextID","m_iSUEmailSendNPC"),
    ("failure","m_iFMessageType","m_iFMessageTextID","m_iFEmailTextID","m_iFEmailSendNPC"),
];
pub(super) fn is_override(field:&str)->bool {PAIRS.iter().any(|(_,_,_,text,npc)|field==*text||field==*npc)}
pub(super) fn email_field<'a>(row:&Value,field:&'a str)->&'a str {
    PAIRS.iter().find(|(_,_,old,text,_)|field==*old||field==*text)
        .filter(|(_,route,_,_,_)|row[*route].as_i64().unwrap_or(0)&6==6).map(|(_,_,_,text,_)|*text).unwrap_or(field)
}
