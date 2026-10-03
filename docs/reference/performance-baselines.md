# Historical performance measurements

Retained observations from the supplied performance document, not a current benchmark
or default task. Current invariants: [performance](../performance.md).

## Retained observations and limitations

September 4 native consolidation removed 3,854 PNGs,added 336,rewrote 15,260 references in 638 documents
(including 410 GLBs),saving 18,926,944 encoded bytes while preserving 146,918 placements and binary
geometry/animation. These are historical library counts. September 12 audit found 24,996 PNGs,
21,680 unique payloads,3,316 redundant copies/45,788,636 bytes;12,012 GLBs were distinct. Whole-library
duplicates do not predict simultaneously resident resources or FPS.

The September 5 matrix below compares a coherent pre-final snapshot with adaptive uniform upload,
three HUD dirty guards and rig bitsets; both already contained sharing/mip/index improvements.
RTX 3060/DX 12,uncapped 1920×1080,eight quiet seconds before each attempt,no compiler during capture.
Sector V's 0.091 ms stationary difference is within variation. Frozen pool/Townsville retained counts
with normal foliage/raster differences. Traces showed UI layout≈3 ms; the layout-cache experiment failed.

Reentry originally added 6,966 nodes/48 roots per return. Production constructors now run once:
focused fixture retains 346 entity IDs across eight cycles. Full fixture baseline nodes
7,882→15,201→22,167;fixed 7,882→8,235→8,235 (selection first becomes resident). First-entry timing
variation is not a general FPS gain. This is offline lifecycle evidence, not live reconnect.

Streaming retirement benchmark 12,801 entities:401→51 frames,total CPU 5.9404→6.1864 ms,p 95 step
0.0186→0.1709 ms. Warm production transition 0.1461 s/35 frames,max 9 tiles,p 95=6.5993 ms; not cold I/O.
Later executable comparisons also included unrelated changes, so no streaming-only FPS attribution.

HUD material preparation fell 0.2002+0.1981→0.0027+0.0028 ms in traces; layout stayed≈3.2 ms.
The clean eight-run table below establishes no repeatable overall FPS improvement. The September 11
attribution found UI traversal, pool material/uniform work and rig/ground queries; plain-run variation
was unresolved. Clock sampling failed; no thermal diagnosis. A historical Bevy 0.17.3 traversal finding
must be remeasured on the current toolchain. Final report spikes are excluded from gameplay timing.

Rig CPU fixture 200 frames/52 roots/6,312 sources:30.3565→7.1383 ms with 62,400 exact matches.
Ground 4,000 queries/8,192 triangles:402.5264→11.2878 ms,checksum-167.1076555310865; full synthetic
preparation 12.3086 ms is spread over budgets in gameplay. Full-client results below do not prove a
general FPS improvement: Pool had no gain, Sector V mean gain was modest and within variation.
Nanomachine's outdated visual-sidecar hash was refreshed only after proving BIN and 43 textures equal,
149 JSON changes URI-only; captures before this catalog repair are invalid.

Material admission reduces resident counts in the final table. It preserves poses,placements,
images,CPU texture bytes and pass entities. Equal analytic GPU UV snapshots retain phase ownership.
Full-image equality is not claimed: baseline repeats also vary. Clean timing series were interrupted
by compilation; pool-clean-reused-1=16.349 ms is invalid. The single 11.918→11.621 ms pair does not prove
FPS gain. Complete reversed clean pairs before attributing one.

Rail captures at(-2445,-13,2580) and(-1995,26,2065) retained placement/asset/tile counts and restored
one visible mesh each (471→472,479→480);(-1991,28,2061) checked the opposite end. Pose error<0.00004.
Overlapping compilation invalidates timing, not the observed visibility correction.

Historical focused suites passed their stated domains, but broader suites had known stale fixtures:
localization source audits, nanocom audio/icon expectations, world catalog/hash fixtures and one
animation hash test. This document does not assert a currently green whole workspace. Run relevant
current tests. Original captures remain optional ignored target/performance artifacts; paths do not
imply that artifacts exist on a fresh checkout.

## Historical measurement tables

Dates and caveats above apply. These tables preserve measured values, not new claims.

### Measurements from the native GPU probe

| Scene | Placements (both) | Mesh entities (both) | Visible meshes (both) | Bound native materials, baseline → shared | Bound meshes, baseline → shared |
| --- | ---: | ---: | ---: | ---: | ---: |
| Future pool | 6,403 | 7,057 | 92 | 3,018–3,035 → 2,521 | 1,052 → 1,037 |
| Sector V | 1,594 | 2,060 | 201 | 509 → 440 | 378 → 377 |

### Final matched production-client comparison

| Camera | Location | Mean ms, baseline -> final | Derived FPS, baseline -> final | p95 ms, baseline -> final | Placements, both |
| --- | --- | ---: | ---: | ---: | ---: |
| Stationary | Pokey Oaks South pool | 22.066 -> 10.658 | 45.3 -> 93.8 | 23.532 -> 12.016 | 6,403 |
| Stationary | Sector V | 8.836 -> 8.927 | 113.2 -> 112.0 | 9.581 -> 9.730 | 1,594 |
| Stationary | Tech Square | 11.408 -> 11.079 | 87.7 -> 90.3 | 13.113 -> 12.012 | 13,173 |
| Stationary | Townsville Center | 11.893 -> 11.526 | 84.1 -> 86.8 | 12.970 -> 12.304 | 10,882 |
| 300-degree orbit | Pokey Oaks South pool | 22.034 -> 11.521 | 45.4 -> 86.8 | 41.194 -> 12.887 | 6,403 |
| 300-degree orbit | Sector V | 10.145 -> 10.141 | 98.6 -> 98.6 | 11.626 -> 11.459 | 1,594 |
| 300-degree orbit | Tech Square | 12.757 -> 11.951 | 78.4 -> 83.7 | 17.106 -> 13.823 | 13,173 |
| 300-degree orbit | Townsville Center | 16.299 -> 11.556 | 61.4 -> 86.5 | 26.689 -> 12.747 | 10,882 |

### Character-change regression fixed (September 7, 2026)

| Location | Entry | Mean ms, baseline -> final | p95 ms, baseline -> final | Derived FPS, baseline -> final |
| --- | ---: | ---: | ---: | ---: |
| Sector V | 1 | 16.475 -> 15.248 | 19.470 -> 17.947 | 60.7 -> 65.6 |
| Sector V | 2 | 21.506 -> 15.436 | 26.363 -> 17.885 | 46.5 -> 64.8 |
| Sector V | 3 | 26.567 -> 16.463 | 31.047 -> 19.103 | 37.6 -> 60.7 |
| Pokey Oaks South pool | 1 | 14.169 -> 12.734 | 16.590 -> 14.183 | 70.6 -> 78.5 |
| Pokey Oaks South pool | 2 | 18.614 -> 14.400 | 21.464 -> 16.698 | 53.7 -> 69.4 |
| Pokey Oaks South pool | 3 | 23.334 -> 14.481 | 26.054 -> 18.144 | 42.9 -> 69.1 |

### September 11, 2026: avoid redundant HUD asset and component writes

| Scene / snapshot | Mean ms (two runs) | FPS = 1000 / mean ms | p95 ms (two runs) |
| --- | --- | --- | --- |
| Pool before | 11.6265 / 11.7869 | 86.01 / 84.84 | 12.9157 / 13.0904 |
| Pool after | 11.6575 / 11.5276 | 85.78 / 86.75 | 12.8610 / 13.0138 |
| Sector V before | 13.0712 / 13.1128 | 76.50 / 76.26 | 13.9381 / 14.0136 |
| Sector V after | 12.9839 / 13.5384 | 77.02 / 73.86 | 13.9243 / 14.9382 |

### September 11, 2026: frame-drop attribution follow-up

| CPU system, average ms/frame | Pool, 14 NPCs | Sector V, 52 NPCs |
| --- | ---: | ---: |
| Bevy UI layout | 3.3043 | 3.3055 |
| Bevy UI stack | 0.6270 | 0.6311 |
| Bevy UI clipping | 0.4899 | 0.4789 |
| Rig material metadata validation | 0.1424 | 0.6897 |
| Rig readiness finalization | 0.0643 | 0.3488 |
| Authored ground resolution | 0.1674 | 0.8270 |
| Bevy animation targets | 0.3570 | 1.1641 |
| World material curve updates | 0.4824 | 0.0263 |
| Native GPU uniform uploads | 0.9790 | 0.0141 |

### September 11, 2026: frame-drop attribution follow-up

| Scene | Mean ms | FPS (1000 / mean) | p95 ms | p99 ms |
| --- | ---: | ---: | ---: | ---: |
| Pool | 13.5646 | 73.72 | 16.2189 | 17.8108 |
| Sector V | 19.3702 | 51.63 | 28.5959 | 45.4141 |
| Sector V repeat, process sampler enabled | 14.8576 | 67.31 | 19.4636 | 27.4373 |

### Rig query and NPC ground indices (September 2026)

| Scene/run | Baseline mean ms | Indexed mean ms | Baseline p95 ms | Indexed p95 ms |
| --- | ---: | ---: | ---: | ---: |
| Pool pair 1 | 11.9827 | 12.7003 | 13.9970 | 15.2265 |
| Pool pair 2 (reverse order) | 11.5564 | 11.6268 | 13.2431 | 13.1223 |
| Sector V pair 1 | 13.6436 | 12.9426 | 15.4197 | 15.0346 |
| Sector V pair 2 (reverse order) | 12.9075 | 12.8168 | 14.0511 | 14.5634 |

### Native asset and resident material audit (12 September 2026)

| World-only snapshot | Bound material IDs | Unique current values | Duplicate current-value IDs |
| --- | ---: | ---: | ---: |
| Pool frozen, cross-model sharing disabled | 2,944 | 501 | 2,443 |
| Pool frozen, sharing enabled | 2,521 | 501 | 2,020 |
| Sector V frozen, sharing disabled | 512 | 207 | 305 |
| Sector V frozen, sharing enabled | 440 | 207 | 233 |
| Pool animated, sharing enabled | 2,521 | 535 | 1,986 |
| Sector V animated, sharing enabled | 440 | 225 | 215 |

### Immutable model/effect admission and first-write world ownership (12 September 2026)

| Frozen scene | World probe bound material IDs, before / after | Full client resident material IDs, before / after |
| --- | ---: | ---: |
| Pool | 2,521 / 643 | 3,029 / 1,356 |
| Sector V | 440 / 352 | 634 / 546 |
