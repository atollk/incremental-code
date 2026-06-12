# `pdcheat` programs by progression stage

This document sketches what an *optimal* program looks like at each meaningful
progression point, so the `pdcheat` cheat can be implemented to emit it.

It is a design doc, not a spec — the literal line widths, line counts, and
`max_int_lit` values used in examples come from the row in `upgrades.md` that
makes the unlock relevant; concrete budgets in the implementation will pull
from `current_upgrades` directly.

## Core resource cheat-sheet

The numbers a run can produce are (from `src/game_state/program.rs`):

- `bronze ≈ instructions ^ (1 + bronze_exp)` — driven by **total instructions
  executed**. Loops multiply this; each `pass;` in a body of depth-d nested
  loops over counter `N` contributes `N^d` instructions.
- `silver = Σ tᵢ · silver_per_sec` — driven by **total seconds passed to
  `sleep`**. Each `sleep(t)` inside a loop multiplies its `t` by the
  iteration count.
- `gold` — driven by the length of the **last** `print(s)`'s argument.
- `diamond = log2(min(b,s,g)) · brk_relatives · diamond_per_brk`, where
  `brk_relatives = Π(1 − bucket_size_i / total)`. Brks gate diamond, but
  early brks (before the loop) keep `brk_relatives` close to 1.

Five recurring design moves drop out of those formulas:

1. **One nested-loop scaffold per unlock tier** — keep multiplying total
   instructions. Each loop costs ~7 lines.
2. **`sleep(...)` lives inside the deepest body**, so its argument is
   multiplied by all surrounding loop counts. Sleep does not affect runtime
   directly; it only feeds silver and splits the instruction bucket.
3. **String accumulation pattern** — declare `s:="";` once, append a
   width-filling literal to it in the deepest body, `print(s);` after the
   loops. `print_len` then equals (iterations) × (literal length), not the
   line-width cap.
4. **`brk();` is the FIRST statement** (or first few). Pre-brk bucket has
   zero instructions, so it doesn't pull `brk_relatives` down. `brk_slowdown`
   also raises the per-instruction floor — useful once the speed exponent
   has gone negative.
5. **Loop counters use chained max-int multiplication** (e.g.,
   `i:=99*99*99;`) to fit as much iteration into one line as the literal
   limit and line width allow. Reuse the existing `counter_expr` helper.

The remainder of the document walks each progression point and shows the
program the cheat should emit.

---

## Stage A — pre-loops, no built-ins

State: `statements = None`, `max_int_lit ∈ {1, 2, 5}`, `lines ∈ {1, 2, 4}`,
no `unlock_print`, `unlock_sleep`, or `unlock_brk`.

```
pass;
pass;
pass;
pass;
```

**Idea.** No structural multiplier yet, no built-ins to call. Bronze is the
only currency, and it grows linearly with the line budget. Just spend every
line on a pass.

---

## Stage B — `SimpleLoops` unlocked

State: `CodeStatements = SimpleLoops`, `lines ≥ 8`, `width ≥ 9`,
`max_int_lit ≤ 10`. No built-ins.

```
i:=6*6;
loop:
if i==0:
break;
end
pass;
i=i-1;
end
```

**Idea.** First multiplier. One loop turns ~5 lines of body into
`N · body_lines` instructions where `N` is the counter. The counter init
uses `counter_expr` to chain as many max-int literals as fit on a line —
each `*` doubles down on the iteration count.

---

## Stage C — `NestedLoops`

State: `CodeStatements ∈ {NestedLoops, Functions, SingleRecursion}`,
`lines ≥ 15`, `width ≥ 12`, `max_int_lit = 10`.

```
i:=10*10*10;
loop:
if i==0:
break;
end
j:=10*10*10;
loop:
if j==0:
break;
end
pass;
j=j-1;
end
i=i-1;
end
```

**Idea.** Stack the multiplier. Depth grows linearly with line budget
(7 lines per loop level + 1 for the deepest body). `body_passes` should be
≥ 1 so the inner loop has something to do — every additional body pass
is another full `N^depth` worth of bronze.

---

## Stage D — `UnlockSleep` (Level 3)

State: `unlock_sleep = true`. Strings and `unlock_print` still locked.

```
i:=10*10*10;
loop:
if i==0:
break;
end
j:=10*10*10;
loop:
if j==0:
break;
end
sleep(10*10*10);
j=j-1;
end
i=i-1;
end
```

**Idea.** Swap one body `pass;` for `sleep(<big chained literal>);`. Silver
per run jumps from 0 to roughly `N_outer · N_inner · sleep_arg ·
silver_per_sec`, with no real-time cost (sleep doesn't add to execution
wall time — it only fills `sleep_calls`). The remaining body pass still
contributes bronze; if the line budget only affords one body slot, sleep
takes it.

---

## Stage E — Strings + `UnlockPrint` (Level 4)

State: `unlock_print = true` AND `literals.value().0 == true` (strings
unlocked at literals level 4). `unlock_sleep = true`.

```
s:="";
i:=10*10*10;
loop:
if i==0:
break;
end
j:=10*10*10;
loop:
if j==0:
break;
end
s=s+"aaaaaaaaaa";
sleep(10*10*10);
j=j-1;
end
i=i-1;
end
print(s);
```

**Idea.** Build the print payload, don't quote it. `s:="";` and `print(s);`
bracket the loops; inside the deepest body, `s=s+"aaaa…";` appends a
width-filling literal once per iteration, so the final `print_len` ends up
at `N_outer · N_inner · literal_len` — orders of magnitude larger than what
fits in a single `print("…");` line. The `print` call lives outside the
loops; only the last call matters, so calling it once is correct.

If `unlock_print` is on but strings are still locked (possible while
literals is below level 4), skip the print machinery entirely — the
language won't accept a string literal yet.

---

## Stage F — `MultiRecursion`

State: `CodeStatements = MultiRecursion`. Sleep + print typically already
unlocked at this point.

```
s:="";
def f(n):
if n==0:
return 0;
end
s=s+"aaaaaaa";
sleep(10*10);
f(n-1);
f(n-1);
end
f(10*10);
print(s);
```

**Idea.** Replace the nested-loop scaffold with tree recursion: instruction
count grows like `branching ^ depth`. The body is the same two
side-effects as the loop case (`s=s+"…";` then `sleep(…);`); they execute
once per recursive call, so the multiplier carries through. `s` is
declared in the top-level frame; appends inside `f` resolve up the call
stack (per `assign_variable` in `compile.rs`), so the accumulator survives
across calls.

`f(n-1); f(n-1);` is the cheapest tree-recursion shape; with more lines in
the function body you can add more `f(n-1);` calls to increase the branch
factor.

---

## Stage G — `UnlockBrk` (Level 5)

State: `unlock_brk = true`. Strings, print, sleep typically all on.

```
brk();
s:="";
i:=10*10*10;
loop:
if i==0:
break;
end
j:=10*10*10;
loop:
if j==0:
break;
end
s=s+"aaaaaaaaaa";
sleep(10*10*10);
j=j-1;
end
i=i-1;
end
print(s);
```

**Idea.** A single `brk();` at the very top. Two effects:

1. The pre-brk bucket has zero instructions, so it doesn't pull
   `brk_relatives` down — diamond stays close to its max.
2. Every subsequent instruction runs at `brk_slowdown^1` ≥ 2× the speed-floor
   cost. Once `instruction_execution_speed` exponent goes negative, that
   slowdown delays when per-instruction time saturates at
   `min_instruction_duration`, letting the program run more instructions
   before hitting that floor — net bronze gain in the late game.

Multiple leading `brk();` would push both effects further, at the cost of
more bronze tax early. The cheat defaults to one.

---

## Stage H — endgame combined

State: late literals (`max_int_lit = 99` or `255`), `width = 50`–`80`,
`lines = 30`–`40`, multi-recursion, all built-ins, brk slowdown high.

```
brk();
s:="";
def f(n):
if n==0:
return 0;
end
s=s+"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
sleep(99*99*99*99*99*99*99*99*99*99*99*99*99*99);
f(n-1);
f(n-1);
f(n-1);
end
f(99*99*99*99);
print(s);
```

**Idea.** All five moves from the cheat-sheet stacked. The function body
spends three lines on `f(n-1);` calls so the branching factor is 3 — line
budget will allow more or fewer. Counter, sleep argument, and string
literal each saturate the line width via `counter_expr` / padding. Brk
stays at the top. Print stays at the bottom.

---

## Implementation hooks

The cheat command sits in
`src/game_scenes/home_terminal/commands/cheat_pdcode_command.rs`. The
existing helpers `counter_expr`, `nested_loops_code`, `build_nested_loop`,
and `multi_recursion_code` already do the structural work. The new logic
slots in around them:

- A **leading** region (`brk();`, then `s:="";`) prepended once.
- A **body-extras** list (`s=s+"…";`, `sleep(…);`) that the structural
  helpers splice into the deepest body, each consuming one body-pass slot
  and reducing depth if there's no room.
- A **trailing** region (`print(s);`) appended once.

Each region is gated independently on its unlock, so the stage transitions
above fall out automatically as upgrades come online.
