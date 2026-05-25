use crate::game_scenes::base::SceneSwitch;
use crate::game_state::{CodeStatementLevels, Upgrades, with_game_state_mut};
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use itertools::Itertools;
use ratatui_widgets::paragraph::Paragraph;
use std::iter;

// Adds predefined code snippets as the current program code.
pub(super) fn cheat_pdcode_cmd(height: u16) -> Box<dyn RunningCommand<SceneSwitch>> {
    with_game_state_mut(|game_state| {
        game_state.program_code = get_predefined_code(&game_state.upgrades)
    });
    Box::new(ParagraphCmd::new(Paragraph::new("Overwrote program code")))
}

fn get_predefined_code(current_upgrades: &Upgrades) -> String {
    match current_upgrades.statements.value() {
        CodeStatementLevels::None => {
            let line = "pass;";
            iter::repeat(line)
                .take(current_upgrades.code_line_count.value() as usize)
                .join("\n")
                .collect()
        }
        CodeStatementLevels::SimpleLoops => {
            todo!()
        }
        CodeStatementLevels::NestedLoops => {
            todo!()
        }
        CodeStatementLevels::Functions => {
            todo!()
        }
        CodeStatementLevels::SingleRecursion => {
            todo!()
        }
        CodeStatementLevels::MultiRecursion => {
            todo!()
        }
    }
}

/*
I'll write programs for the rows where the program shape actually changes (a fresh `pass`-spam isn't worth its own block). Each shows the optimal program at that state along with the resulting count.

### Row 10 — max = 8

State: `W=10, L=8, E=2, S=0`. No loops yet.

```
pass;
pass;
pass;
pass;
pass;
pass;
pass;
pass;
```

8 statements × 1 instruction each = **8**.

### Row 11 — max ≈ 380 (single loop just unlocked)

State: `W=10, L=8, E=2, S=1`. The minimal counted-loop scaffold is 7 lines; one extra `pass;` fits in the body.

```
i:=5*5*5;
loop:
if i==0:
break;
end
pass;
i=i-1;
end
```

`N = 5³ = 125`. Per iter (i ≠ 0): `if` (1) + `pass` (1) + `i=i-1` (1) = 3. Final iter (i = 0): `if` (1) + `break` (1) = 2. Total: 1 (decl) + 1 (loop entry) + 125·3 + 2 = **379**.

### Row 14 — max ≈ 2.9×10⁸ (W=15, E=4 — `99` is the dominant literal)

State: `W=15, L=8, E=4, S=1`. `i:=99*99*99*99;` is exactly 15 chars.

```
i:=99*99*99*99;
loop:
if i==0:
break;
end
pass;
i=i-1;
end
```

`N = 99⁴ = 96,059,601`. Per iter: 3. Total: 1 + 1 + 96,059,601·3 + 2 = **288,178,807**.

### Row 17 — max ≈ 1.44×10⁹ (L=20 still single loop)

State: `W=15, L=20, E=4, S=1`. Same `N`, but 13 extra `pass;` lines now fit in the body.

```
i:=99*99*99*99;
loop:
if i==0:
break;
end
pass;
pass;
pass;
pass;
pass;
pass;
pass;
pass;
pass;
pass;
pass;
pass;
pass;
i=i-1;
end
```

Per iter: 1 + 13 + 1 = 15. Total: 2 + 9.6×10⁷·15 + 2 ≈ **1.44×10⁹**.

### Row 18 — max ≈ 7.4×10¹⁶ (S=1→2, nested loops)

State: `W=15, L=20, E=4, S=2`. The 13 body lines become an inner loop scaffold (7) + 6 inner-body passes.

```
i:=99*99*99*99;
loop:
if i==0:
break;
end
j:=99*99*99*99;
loop:
if j==0:
break;
end
pass;
pass;
pass;
pass;
pass;
pass;
j=j-1;
end
i=i-1;
end
```

Innermost iter cost: 1 + 6 + 1 = 8. Innermost iter count: `N² ≈ 9.2×10¹⁵`. Total ≈ **7.4×10¹⁶**.

### Row 19 — max ≈ 6.6×10³⁶ (W=30 lets `N` jump to 99⁹)

State: `W=30, L=20, E=4, S=2`. Same shape, longer literal expressions.

```
i:=99*99*99*99*99*99*99*99*99;
loop:
if i==0:
break;
end
j:=99*99*99*99*99*99*99*99*99;
loop:
if j==0:
break;
end
pass;
pass;
pass;
pass;
pass;
pass;
j=j-1;
end
i=i-1;
end
```

`N = 99⁹ ≈ 9.1×10¹⁷`. Innermost iters: `N² ≈ 8.3×10³⁵`. Total ≈ **6.6×10³⁶**.

### Row 21 — max ≈ 2.7×10⁷² (L=30, four-deep nesting)

State: `W=30, L=30, E=4, S=3`. Four loop scaffolds (28 lines) + 2 innermost passes. `S=3` is irrelevant — functions don't beat plain nesting at this line budget, so the optimal program ignores `def`.

```
i:=99*99*99*99*99*99*99*99*99;
loop:
if i==0:
break;
end
j:=99*99*99*99*99*99*99*99*99;
loop:
if j==0:
break;
end
k:=99*99*99*99*99*99*99*99*99;
loop:
if k==0:
break;
end
l:=99*99*99*99*99*99*99*99*99;
loop:
if l==0:
break;
end
pass;
pass;
l=l-1;
end
k=k-1;
end
j=j-1;
end
i=i-1;
end
```

Innermost iter cost: 4. Innermost iters: `N⁴ ≈ 6.86×10⁷¹`. Total ≈ **2.7×10⁷²**.

### Row 24 — max ≈ 2×10²⁴⁹ (W=80, L=40 — the polynomial ceiling)

State: `W=80, L=40, E=4, S=4`. Five loop scaffolds (35 lines) + 5 innermost passes. Each counter is 25 factors of `99` (78 chars), `N = 99²⁵ ≈ 7.7×10⁴⁹`. Linear recursion (S=4) doesn't beat this — it costs lines for the function scaffold without buying extra depth.

```
i:=99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99;
loop:
if i==0:
break;
end
j:=99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99;
loop:
if j==0:
break;
end
k:=99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99;
loop:
if k==0:
break;
end
l:=99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99;
loop:
if l==0:
break;
end
m:=99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99;
loop:
if m==0:
break;
end
pass;
pass;
pass;
pass;
pass;
m=m-1;
end
l=l-1;
end
k=k-1;
end
j=j-1;
end
i=i-1;
end
```

Innermost iter cost: 7. Innermost iters: `N⁵ ≈ 2.7×10²⁴⁸`. Total ≈ **1.9×10²⁴⁹**.

### Row 26 — saturated (S=5, tree recursion)

State: `W=80, L=40, E=4, S=5`. Doubling recursion plus a width-stuffed argument blows past anything realistic.

```
def f(n):
if n==0:
return 0;
end
f(n-1);
f(n-1);
end
f(99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99*99);
```

8 lines, 32 spare. `N = 99²⁵ ≈ 7.7×10⁴⁹`. Calls: `~2^N`, so instructions on the order of `5·2^(7.7×10⁴⁹)`. **Saturated** at whatever the runtime cap is — and with 32 lines unused, the player can graduate to Ackermann (`def A(m,n): if m==0: return n+1; end if n==0: return A(m-1,1); end return A(m-1,A(m,n-1)); end A(4,99);`, ~10 lines, `W=23` minimum) which saturates faster than doubling but doesn't change the practical outcome.

A small note on what's *not* shown: rows where I expected meaningful program changes but they don't actually appear in the optimal program. Rows 20 (`def`, no recursion) and 25 (linear recursion) have the same optimal program as the nesting that came before, because at those line budgets the function scaffold costs as much as it saves. The player who hits S=3 or S=4 *can* write functions — they're useful for code organisation and they'll be needed to climb to S=5 — but the busy-beaver-maximising program at that exact row doesn't use them.
 */
