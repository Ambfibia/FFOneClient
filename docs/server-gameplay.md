# Server and gameplay

Target `../RustyFusion` with one FFOne / Retrobution 0104 protocol. `../OpenFusion`
is legacy reference, not the destination for new server behavior. Migration completion
must be checked against the affected code/tests and RustyFusion's
`docs/openfusion-migration-audit.md`, not inferred from the server choice.

Preserve wire sizes/packing, ordering, explicit IDs and authoritative post-state.
UI may submit typed intent, but cannot speculate inventory, currency, rewards or
other server-owned state. Retain malformed-frame handling and correlated reply/timeout
ownership when changing a transaction.

Nano ID, acquisition prerequisites and progression effects are independent.
Progression Nano rewards may raise level; other quest/item rewards may require a
level without raising it. Never derive level from Nano ID or remove all Nano level-ups.
Accepted identities: current tables plus `FusionForge/recipes/native/identities/accepted-native-identities.json`.

Check only affected protocol/net/gameplay tests and the relevant server round trip.
Use [packet inventory](protocol-0104-inventory.md) for wire work and
[character-creation contract](protocol-0104-character-creation.md) for that flow.
