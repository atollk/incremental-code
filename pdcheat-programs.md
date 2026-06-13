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
- `gold` — driven by the length of the **max** `print(s)` argument across all calls.
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

State: `CodeStatements ∈ {NestedLoops, Functions, PureFunctions, SingleRecursion, MultiRecursion}`,
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

## Stage F — `PureFunctions`

State: `CodeStatements = PureFunctions`. No recursion allowed yet. Sleep +
print typically already unlocked at this point.

```
s:="";
def pure g(x):
sleep(10*10*10);
end
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
g(0);
j=j-1;
end
i=i-1;
end
print(s);
```

**Idea.** Mark the sleep helper with `pure` and call it with a fixed
argument (`0`) inside the innermost loop. On the first iteration the
interpreter executes the body (the sleep call itself) and caches the meta
diff — instruction counts plus the sleep entry. Every subsequent iteration
with the same argument is a *cache hit*: the diff is replayed instantly,
adding the sleep contribution to the total without re-executing the body.
Bronze and silver accumulate just as if the call were inlined.

The string append `s=s+"...";` lives in the **loop body**, not inside the
pure function. Cache hit replay only restores the `meta` diff; it does
**not** re-execute any side effects on outer-scope variables. Putting the
append inside `g` would mean only the first iteration ever extends `s`,
and gold would collapse to a tiny value. Keeping the append in the loop
body ensures it runs every iteration.

At this stage the pure helper is a modest line-saving tool rather than a
multiplier. The real pay-off arrives in Stage G, where tree recursion
combined with `pure` lets each unique argument execute only once while
still accumulating the full exponential instruction and sleep count via
diff replay.

---

## Stage G — `MultiRecursion` (with `pure`)

State: `CodeStatements = MultiRecursion`. `PureFunctions` is a prerequisite
and is always available here. Sleep + print typically already unlocked.

```
s:="";
def pure f(n):
if n==0:
return 0;
end
sleep(10*10);
f(n-1);
f(n-1);
end
i:=10*10*10;
loop:
if i==0:
break;
end
s=s+"aaaaaaaaaa";
i=i-1;
end
f(10*10*10);
print(s);
```

**Idea.** Pure tree recursion is the bronze and silver multiplier; a
separate loop outside handles the string accumulator for gold.

`f` is marked `pure`: the interpreter executes each unique argument exactly
once. For a starting depth of `D`, it makes `D` unique body executions and
`2^D − D` cache-hit replays. Each replay applies the full sub-tree's diff
— instruction counts and sleep calls — without re-running the body. Bronze
grows like `2^D`; silver likewise. Because `D` can be enormous (the
interpreter only runs `D` bodies instead of `2^D`), setting `D` orders of
magnitude larger than feasible without pure is now practical.

The string-accumulator **cannot** go inside `f`. Cache hit replay does not
re-execute outer-scope mutations: `s=s+"...";` inside `f` would only run
on the `D` first-call bodies, not the `2^D` cache-hit replays. Gold would
barely grow. Instead, a separate short loop before `f(...)` accumulates
`s` the old way — using some of the line budget, but giving gold a proper
multiplier independently of the recursion depth.

`f(n-1); f(n-1);` is the cheapest tree-recursion shape. With more lines
in the function body you can add more `f(n-1);` calls for a higher branch
factor.

---

## Stage H — `UnlockBrk` (Level 5)

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

## Stage I — endgame combined

State: late literals (`max_int_lit = 99` or `255`), `width = 50`–`80`,
`lines = 30`–`40`, multi-recursion, all built-ins, brk slowdown high.

```
brk();
s:="";
i:=99*99*99*99*99*99*99*99*99*99*99*99*99*99;
loop:
if i==0:
break;
end
s=s+"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
i=i-1;
end
def pure f(n):
if n==0:
return 0;
end
sleep(99*99*99*99*99*99*99*99*99*99*99*99*99*99);
f(n-1);
f(n-1);
f(n-1);
end
f(99*99*99*99);
print(s);
```

**Idea.** All six moves from the cheat-sheet stacked. `brk();` is first.
A short loop accumulates `s` for gold — the string literal and loop
counter each saturate the line width. The pure recursive `f` drives bronze
and silver: branch factor 3 (add more `f(n-1);` lines if the budget
allows), sleep argument saturates the line width, `f(99*99*99*99)` sets an
enormous starting depth that the interpreter can handle because only
`99*99*99*99` unique bodies run instead of `3^(99^4)`. `print(s);` caps
the program; gold comes from the loop, not the recursion.

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

**Pure functions** (`CodeStatements >= PureFunctions`) change the recursion
template: the recursive function is emitted as `def pure f(n):` and the
initial call depth can be set much larger (e.g. `f(max_int^k)`). However,
the string-accumulator body extra (`s=s+"…";`) must **not** go inside the
pure function — cache hits skip outer-scope mutations, so gold would barely
grow. At the `PureFunctions` stage (no recursion), a pure sleep-only helper
can be called inside the innermost loop with a fixed argument; the
string-append extra stays in the loop body directly. At the
`MultiRecursion` stage, the string accumulation is moved to a separate
short loop outside the recursive function, and the recursive body contains
only `sleep(…);` and the `f(n-1);` calls.
