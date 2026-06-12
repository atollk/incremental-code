Here's a fresh take with the 5-tier S split, tighter cost pacing (3× current max — about three runs per upgrade, which
suits the gentler curve), and the Approach 2 budget tier that would naturally sit alongside each step.

| #  | Upgrade                        |  max before |   cost (3×) |                       max after | Approach 2 budget |
|----|--------------------------------|------------:|------------:|--------------------------------:|------------------:|
| 1  | L: 1→2                         |           1 |           3 |                               2 |                10 |
| 2  | L: 2→4                         |           2 |           6 |                               4 |                10 |
| 3  | UnlockSleep                    |           4 |          12 |                               4 |                10 |
| 4  | W: 5→10                        |           4 |          12 |                               4 |                10 |
| 5  | L: 4→"5" *(cosmetic)*          |           4 |          12 |                               4 |                10 |
| 6  | L: "5"→6                       |           4 |          12 |                               6 |                10 |
| 7  | E: 0→1 (lit 2)                 |           6 |          18 |                               6 |               100 |
| 8  | L: 6→"7" *(cosmetic)*          |           6 |          18 |                               6 |               100 |
| 9  | E: 1→2 (lit ≤5)                |           6 |          18 |                               6 |               100 |
| 10 | L: "7"→8                       |           6 |          18 |                               8 |               100 |
| 11 | **S: 0→1 — single loop**       |           8 |          24 |                            ~380 |               1 K |
| 12 | E: 2→3 (lit ≤10)               |         380 |       1.1 K |                           2.2 K |              10 K |
| 13 | E: 3→4 (string literal)        |         380 |       1.1 K |                           2.2 K |              10 K |
| 14 | UnlockPrint                    |             |             |                                 |                   |
| 15 | W: 10→15                       |       2.2 K |       6.6 K |                           1.6 M |              10 M |
| 16 | E: 3→4 (lit ≤100)              |       1.6 M |       4.7 M |                           290 M |               1 B |
| 17 | L: 8→10                        |       290 M |       870 M |                           480 M |               1 B |
| 18 | L: 10→"15" *(cosmetic)*        |       480 M |       1.4 B |                           480 M |               1 B |
| 19 | L: "15"→20                     |       480 M |       1.4 B |                           1.4 B |              10 B |
| 20 | **S: 1→2 — nested loops**      |       1.4 B |       4.3 B |                      7.4 × 10¹⁶ |              10¹⁷ |
| 21 | W: 15→30                       |  7.4 × 10¹⁶ |  2.2 × 10¹⁷ |                      6.6 × 10³⁶ |              10³⁷ |
| 22 | **S: 2→3 — def, no recursion** |  6.6 × 10³⁶ |  2.0 × 10³⁷ |    ~1.3 × 10³⁷ *(line savings)* |              10³⁷ |
| 23 | UnlockBrk                      |             |             |                                 |                   |
| 24 | L: 20→30                       |  1.3 × 10³⁷ |  4.0 × 10³⁷ |                      2.7 × 10⁷² |              10⁷³ |
| 25 | W: 30→50                       |  2.7 × 10⁷² |  8.1 × 10⁷² |                     2.0 × 10¹¹⁹ |             10¹²⁰ |
| 26 | L: 30→40                       | 2.0 × 10¹¹⁹ | 6.0 × 10¹¹⁹ |                     2.0 × 10¹⁴⁹ |             10¹⁵⁰ |
| 27 | W: 50→80                       | 2.0 × 10¹⁴⁹ | 6.0 × 10¹⁴⁹ |                     1.5 × 10²⁴⁹ |             10²⁵⁰ |
| 28 | **S: 3→4 — linear recursion**  | 1.5 × 10²⁴⁹ | 4.5 × 10²⁴⁹ | ~1.5 × 10²⁴⁹ *(stepping stone)* |             10²⁵⁰ |
| 29 | **S: 4→5 — tree recursion**    | 1.5 × 10²⁴⁹ | 4.5 × 10²⁴⁹ |                budget-saturated |             10⁴⁰⁰ |
| 30 | E: 4→5 (lit ≤255)              |         sat |      varies |    sat *(unlocks print of 255)* |             10⁵⁰⁰ |
| 31 | E: 5→6 (empty string)          |         sat |      varies |     sat *(empty-string flavor)* |                 ∞ |

A couple of things worth flagging in this layout that differ from the previous one:

**The cliffs are now spread over two distinct shoulders.** The post-loops shoulder runs from row 11 (single loop unlock)
through row 18 (nesting unlock) — five upgrades climbing 8 → ~10¹⁷, with the W and E knobs doing visible work between
the two S steps. The post-recursion shoulder runs from row 18 through row 26 — eight upgrades climbing 10¹⁷ → saturated,
with W/L/E driving most of it and S doing the structural cliffs at either end. This is the smoothing you bought from the
split: every five-or-so upgrades you get a real S step, instead of a 30-upgrade gap with two huge cliffs.

**Rows 20 and 25 (S=2→3 and S=3→4) are honest stepping stones.** Linear recursion with nested loops inside its body has
the same asymptotic growth as plain nested loops with the same line budget — the extra +1 effective depth from recursion
is paid for by the function scaffold, so the two cancel. `def`-without-recursion gives a small line-saving boost (~2×)
when there's a body worth factoring out, but no new growth class. They're priced at the standard 3× because they're
gates to S=5, not power purchases — and the budget column treats them the same way (budget doesn't tier up across them).

**The budget column reads as the smooth axis underneath the structural one.** Tiers are roughly powers of 100, sometimes
10 where the structural curve is climbing slowly. Each tier corresponds to one Approach 2 upgrade — about 12 budget
purchases across the run, comfortably interleavable between the structural ones. After row 26 (tree recursion unlocked),
the structural max is "however much budget you have", so the last two structural upgrades (E 4→5 and E 5→6, which don't
help max even in principle since `99` already dominates `255`) ride the budget column for their progression value. If
you want those last two upgrades to *also* matter for max, that's where adding a third literal slot — e.g., string
concatenation as a unit of compute, or a printable-character literal — would naturally slot in.