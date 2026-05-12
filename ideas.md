- 'speed up' function -> erhoeht multiplier fuer alle ressourcen aber muss oft hintereinander aufgerufen werden fuer
  echten mehrwert

| #  | Upgrade                        |   cost (3×) |                       max after | Approach 2 budget | Price |
|----|--------------------------------|------------:|--------------------------------:|------------------:|-------|
| 1  | L: 1→2                         |           3 |                               2 |                10 | 10 B  |
| 2  | L: 2→4                         |           6 |                               4 |                10 | B     |
| 3  | UnlockPrint                    |          12 |                               4 |                10 | B     |
| 4  | W: 5→10                        |          12 |                               4 |                10 | B     |
| 5  | L: 4→"5" *(cosmetic)*          |          12 |                               4 |                10 | B     |
| 6  | L: "5"→6                       |          12 |                               6 |                10 | B     |
| 7  | E: 0→1 (lit 2)                 |          18 |                               6 |               100 | B     |
| 8  | L: 6→"7" *(cosmetic)*          |          18 |                               6 |               100 | B     |
| 9  | E: 1→2 (lit ≤5)                |          18 |                               6 |               100 | B     |
| 10 | L: "7"→8                       |          18 |                               8 |               100 | B     |
| 11 | **S: 0→1 — single loop**       |          24 |                            ~380 |               1 K | S     |
| 12 | E: 2→3 (lit ≤10)               |       1.1 K |                           2.2 K |              10 K | S     |
| 13 | W: 10→15                       |       6.6 K |                           1.6 M |              10 M | S     |
| 14 | E: 3→4 (lit ≤100)              |       4.7 M |                           290 M |               1 B | S     |
| 15 | L: 8→10                        |       870 M |                           480 M |               1 B | S     |
| 16 | L: 10→"15" *(cosmetic)*        |       1.4 B |                           480 M |               1 B | G     |
| 17 | L: "15"→20                     |       1.4 B |                           1.4 B |              10 B | G     |
| 18 | **S: 1→2 — nested loops**      |       4.3 B |                      7.4 × 10¹⁶ |              10¹⁷ | G     |
| 19 | W: 15→30                       |  2.2 × 10¹⁷ |                      6.6 × 10³⁶ |              10³⁷ | G     |
| 20 | **S: 2→3 — def, no recursion** |  2.0 × 10³⁷ |    ~1.3 × 10³⁷ *(line savings)* |              10³⁷ | G     |
| 21 | L: 20→30                       |  4.0 × 10³⁷ |                      2.7 × 10⁷² |              10⁷³ | D     |
| 22 | W: 30→50                       |  8.1 × 10⁷² |                     2.0 × 10¹¹⁹ |             10¹²⁰ | D     |
| 23 | L: 30→40                       | 6.0 × 10¹¹⁹ |                     2.0 × 10¹⁴⁹ |             10¹⁵⁰ | D     |
| 24 | W: 50→80                       | 6.0 × 10¹⁴⁹ |                     1.5 × 10²⁴⁹ |             10²⁵⁰ | D     |
| 25 | **S: 3→4 — linear recursion**  | 4.5 × 10²⁴⁹ | ~1.5 × 10²⁴⁹ *(stepping stone)* |             10²⁵⁰ | D     |
| 26 | **S: 4→5 — tree recursion**    | 4.5 × 10²⁴⁹ |                budget-saturated |             10⁴⁰⁰ | *     |
| 27 | E: 4→5 (lit ≤255)              |      varies |    sat *(unlocks print of 255)* |             10⁵⁰⁰ | *     |
| 28 | E: 5→6 (empty string)          |      varies |     sat *(empty-string flavor)* |                 ∞ | *     |