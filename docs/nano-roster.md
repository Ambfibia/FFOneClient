# Native Nano identities

The client and server must use the same Nano and tuning tables. Each active Nano
has a positive ID in `assets/game/data/tables/xdt.json`; gallery entries
use those IDs and never create display-only ownership records.

| Nano | ID | Model | Tuning rows | Skill IDs |
| --- | ---: | --- | --- | --- |
| Flapjack | 37 | `nano_flapjack` | 198, 199, 200 | 198, 199, 200 |
| Johnny Bravo | 38 | `nano_johnnybravo` | 201, 202, 203 | 13, 4, 22 |
| Unstable Nano | 41 | `nano_holonano` | 285, 286, 287 | 210, 211, 212 |
| Coop | 67 | `nano_coop` | 210, 211, 212 | 210, 211, 212 |
| Ben Tennyson | 68 | `nano_ben` | 285, 286, 287 | 210, 211, 212 |
| Ghostfreak | 69 | `nano_ghostfreak` | 285, 286, 287 | 210, 211, 212 |
| Upgrade | 70 | `nano_upgrade` | 285, 286, 287 | 210, 211, 212 |

Mission 841 (The Unstable Nano), tasks 5213–5215, continues to refer to Nano 41.
Existing ownership of ID 41 therefore belongs to the restored Unstable Nano;
Coop must be acquired separately as ID 67. Saved player records are not rewritten.

IDs 52 and 66 remain separate accepted variants sharing the Van Kleiss model.
Their names, skills and ownership are unchanged. ID 52 shares the display name
“Unstable Nano” with ID 41 but uses its own portrait; names are not identity keys.

Ben, Ghostfreak and Upgrade use the owner-selected Unstable Nano powers: bonus
Taros, additional Fusion Matter, and cone stun. Coop retains the same effects
with his own power names and descriptions. Ghostfreak and Upgrade use dedicated normal and ready icons; their animated
models are independent and available in gameplay.

The gallery contains 67 active identities in a five-column scroll view. Updating
the server XDT requires restarting that server; changing client files alone does
not change an already running server's in-memory tables.

Manual summon requires strictly more than 20% of the Nano's maximum stamina
(`m_iNanoBattery1`). With a maximum of 150, stamina 30 is blocked and 31 is
allowed. A blocked attempt plays the summon-failure sound and sends no activation
request. Recalling an already active Nano remains allowed at any stamina.
An active Nano may continue below the summon threshold; stamina and depletion
remain server-authoritative.
