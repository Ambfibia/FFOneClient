use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Pc2pcSessionIdentity0104 {
    pub pair: Pc2pcPair0104,
    pub local_pc_id: i32,
    pub remote_pc_id: i32,
    pub direction: Pc2pcOfferDirection0104,
}

impl Pc2pcSessionIdentity0104 {
    pub fn new(
        pair: Pc2pcPair0104,
        local_pc_id: i32,
        direction: Pc2pcOfferDirection0104,
    ) -> Result<Self, Pc2pcIdentityError0104> {
        let Some(remote_pc_id) = pair.other(local_pc_id) else {
            return Err(Pc2pcIdentityError0104::LocalOutsidePair { pair, local_pc_id });
        };
        let expected = if local_pc_id == pair.from_pc_id {
            Pc2pcOfferDirection0104::Outgoing
        } else {
            Pc2pcOfferDirection0104::Incoming
        };
        if direction != expected {
            return Err(Pc2pcIdentityError0104::DirectionMismatch {
                local_pc_id,
                expected,
                actual: direction,
            });
        }
        Ok(Self {
            pair,
            local_pc_id,
            remote_pc_id,
            direction,
        })
    }

    pub fn validate_envelope(
        self,
        envelope: Pc2pcEnvelope0104,
    ) -> Result<Pc2pcParticipant0104, Pc2pcCorrelationError0104> {
        if envelope.pair != self.pair {
            return Err(Pc2pcCorrelationError0104::PairMismatch {
                expected: self.pair,
                actual: envelope.pair,
            });
        }
        if envelope.requester_pc_id == self.local_pc_id {
            Ok(Pc2pcParticipant0104::Local)
        } else if envelope.requester_pc_id == self.remote_pc_id {
            Ok(Pc2pcParticipant0104::Remote)
        } else {
            Err(Pc2pcCorrelationError0104::RequesterOutsidePair {
                pair: self.pair,
                requester_pc_id: envelope.requester_pc_id,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pc2pcSessionEndReason0104 {
    OfferCancelled,
    OfferRefused,
    OfferAborted,
    ConfirmCancelled,
    ConfirmAborted,
    CombatStarted,
}

pub(super) fn project_pc2pc_icon(
    item: ItemBase0104,
    catalog: &impl UserEquipItemCatalog,
) -> UserEquipProjectedIcon {
    if InventoryRuntime0104::item_is_empty(item) {
        return UserEquipProjectedIcon::Empty;
    }
    match UserEquipCatalogQuery::from_non_empty_item(item) {
        Err(UserEquipCatalogQueryError::MalformedIdentity { item_type, item_id }) => {
            UserEquipProjectedIcon::MissingChecker(UserEquipMissingIconReason::MalformedIdentity {
                item_type,
                item_id,
            })
        }
        Err(UserEquipCatalogQueryError::EmptyItem { .. }) => UserEquipProjectedIcon::Empty,
        Ok(query) if query.kind.legacy_returns_null() => {
            UserEquipProjectedIcon::MissingChecker(UserEquipMissingIconReason::QuestLegacyNull {
                query,
            })
        }
        Ok(query) => {
            match catalog.resolve_icon(query) {
                Some(icon) => UserEquipProjectedIcon::Resolved(icon),
                None => UserEquipProjectedIcon::MissingChecker(
                    UserEquipMissingIconReason::CatalogMiss { query },
                ),
            }
        }
    }
}
