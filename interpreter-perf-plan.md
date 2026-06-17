# Interpreter performance plan

Baseline: `stage_d_gains_silver` runs in ~23.5 s release-mode on macOS arm64. It executes ~1 B atomic instructions against `instruction_limit = 999_999_999` set in `cheat_pdcode_command.rs:441`. Flame graph (samply, `/tmp/stage_d.profile.json`) shows `eval_binary_op` as the dominant self-time function.

The generated stage-D body is a 2-deep nested loop where each inner iteration runs roughly:

```
if i==0: break; sleep(9*9*9*9*9*9); i=i-1;
```

So per iteration the interpreter does ~7 binary ops, one HashMap-backed identifier load for `i`, and one atomic instruction count update. All of these are amenable to attack.

The plan below works in order of expected ROI. Each step lists what to change, where, and a verification approach.

---

## Step 1 — Constant-fold pure literal subtrees

**Why first:** `9*9*9*9*9*9` is re-evaluated every inner iteration. With ~200 M iterations, those 5 multiplications account for the majority of `eval_binary_op` time. This is the smallest change with the largest payoff.

**Where:** new pass in `language/src/`, invoked from `parse_program` in `language/src/parser.rs` before returning the AST, or wired into `compile_with_meta` in `language/src/compile.rs:37` as a preprocessing step.

**What:**
- Add `fn fold_constants(expr: &mut NotPythonExpr)` that recurses post-order.
- For each `NotPythonExpr::Op(op)` where both operand subtrees fold to literals (`Int`, `Float`, `Bool`), compute the result at compile time and replace the node with the corresponding literal.
- Cover at minimum: `Add`, `Sub`, `Mul`, `Div`, `Mod` over `(Int, Int)` and `(Float, *)`; `And`/`Or` over `(Bool, Bool)`; numeric comparisons; `Neg`/`Not`.
- Leave division by zero untouched (let runtime error fire as today) — do *not* fold `a / 0`.
- Run the pass once on `program.statement` after parse.

**Risk:** none for arithmetic; for `Add` on strings, only fold when both sides are string literals (cheap and matches runtime semantics).

**Verify:** rerun `cargo test --features tui --release stage_d_gains_silver`. Add a parser/compile unit test that asserts the AST after folding contains a single `Int(531441)` in place of the multiplication chain.

**Expected impact:** 40–60% wall-clock reduction on `stage_d_gains_silver`.

---

## Step 2 — Strip per-instruction overhead in `WipCompilingProgram`

**Why second:** independent of Step 1 and small. Every atomic instruction pays for: a virtual call through `Rc<RefCell<dyn FnMut() -> bool>>`, a `RefCell` borrow check, two `last_mut().unwrap()` calls inside `CompiledProgram::log_atomic_instruction`. At 10⁹ instructions, those constants add up.

**Where:**
- `src/game_scenes/logic/compilation.rs:113` (`check_cancel`, `log_atomic_instruction`)
- `src/game_state/program.rs:162` (`CompiledProgram::log_atomic_instruction`)

**What:**
1. **Debounce `check_cancel`.** Hold a small counter in `WipCompilingProgram`; only invoke the closure every N (e.g. 4096) atomic instructions. The existing pattern in `compile_thread::compile` (`compilation.rs:236`) already debounces; mirror it here so the debounce is enforced even when callers pass non-debouncing closures (as the test does).
2. **Cache a pointer to the active counter cell.** Replace `instruction_counts.last_mut().unwrap().last_mut().unwrap() += 1` per call with a `*mut u64` (or, more idiomatically, store an index pair and dereference once). Refresh the pointer only when `sleep`/`brk` mutate `instruction_counts`.
3. **`#[inline]`** `WipCompilingProgram::log_atomic_instruction`, `CompiledProgram::log_atomic_instruction`, and `check_cancel`.
4. Optional: split `log_atomic_instruction` so the cancel-check fast path is a single decrement and branch.

**Risk:** the raw-pointer cache must be invalidated whenever `instruction_counts` is reshaped — i.e. in `sleep_calls.push` / `instruction_counts.push` paths inside `predefined_function_sleep` and `predefined_function_brk` (`compilation.rs:28`, `:52`). Easier alternative: store `(brk_idx, sleep_idx)` and write through those without re-walking the vec — single bounds-check unwrap instead of two.

**Verify:** rerun the test; also run `stage_c_loops_more_bronze_than_a` and `stage_a_gains_bronze` to make sure counts are unchanged.

**Expected impact:** 1.3–1.8× on what remains after Step 1.

---

## Step 3 — Indexed variable slots instead of HashMap lookups

**Why third:** larger refactor than Steps 1–2 but still very localized; addresses a hot path that Steps 1–2 leave untouched (`get_variable` for `i==0` and `i=i-1`).

**Where:** `language/src/compile.rs` — `ProgramExecutionCallState.variables` (line 77), `eval_expr::Identifier` arm (line 515), `decl_variable`/`assign_variable`/`get_variable` (lines 191–235).

**What:**
- Run a resolver pass over the AST that, per function scope, assigns each declared name a `u16` slot index. Store these resolved indices in the AST nodes (extend `NotPythonExpr::Identifier` and the assign/decl statements to carry an `OnceCell<u16>` or change the AST to a separate "resolved" representation).
- Replace `HashMap<&'a str, ProgramValue>` per frame with `Vec<ProgramValue>` of fixed length per scope.
- For declarations, push; for assigns/loads, index by slot.
- Keep `HashMap` as a fallback for the outer (global) scope if cross-frame lookup semantics complicate slot resolution — the current `get_variable` walks the call stack, so the resolver needs to record whether a name is local or captured. In practice the current pure/non-pure split keeps things simple: non-pure locals are per-frame, globals are essentially the bottom frame.

**Risk:** the AST is currently shared by `compile_thread` and tests via `parse_program`. If `OnceCell` mutation is undesirable, return a separate `ResolvedProgram` from a new `resolve(program)` step and have `compile_with_meta` take that.

**Verify:** run the full `cargo test --features tui` to catch any regressions in name resolution edge cases (pure/non-pure boundaries, shadowing in nested loops).

**Expected impact:** 1.5–3× on `stage_d_gains_silver` after Steps 1–2.

---

## Step 4 — Make hot `ProgramValue` cases `Copy`

**Why fourth:** the largest refactor, with diminishing returns once Steps 1–3 land. `eval_expr` on `Identifier` does `.cloned()` on a 50+-byte enum because `ProgramValue` includes `List(Vec<…>)` and `Dict(HashMap<…>)`. For the Int-heavy workload this means a `memcpy` per identifier load.

**Where:** `language/src/compile.rs:51` and every site that holds or matches a `ProgramValue` (most of `compile.rs`).

**What:**
- Split `ProgramValue` into a small `ScalarValue: Copy` plus a heap-shared `HeapValue` behind `Rc`:

  ```rust
  #[derive(Copy, Clone, PartialEq, Debug)]
  enum ScalarValue {
      Int(i64),
      Float(f64),
      Bool(bool),
      None,
  }

  #[derive(Clone, Debug)]
  enum HeapValue {
      String(Rc<String>),
      List(Rc<Vec<ProgramValue>>),
      Dict(Rc<HashMap<HashableProgramValue, ProgramValue>>),
  }

  #[derive(Clone, Debug)]
  enum ProgramValue {
      Scalar(ScalarValue),
      Heap(HeapValue),
  }
  ```
- Adjust `HashableProgramValue` to share with `ScalarValue` where possible (Int, Bool become scalars; String stays in the heap layer with shared ownership).
- Migrate `eval_expr`, `eval_binary_op`, `eval_unary_op` to consume scalars by value and heap values by `Rc::clone`. The cost of `cloned()` on the hot path becomes a register copy.
- For `Add` on lists/strings (currently `a + &b`), the `Rc` shape means you'll `Rc::make_mut` or build a fresh `Rc<Vec<…>>`; semantics stay identical.

**Risk:** broad mechanical change. Tests in `language/src/compile.rs` (under `mod tests`) cover most surface area; rely on them. Watch out for `Hash`/`PartialEq` derivations on `HashableProgramValue`.

**Verify:** full `cargo test --features tui`. Re-profile to confirm `eval_expr` self-time drops and that `Rc::clone` does not become the new dominant cost (if it does, consider `Rc<str>` for short-lived strings or interning).

**Expected impact:** 10–25% on `stage_d_gains_silver` after Steps 1–3; the larger benefit is for tests with bigger lists/dicts where cloning is unbounded today.

---

## Suggested rollout

| Step | Effort | Risk | Stop & re-measure |
|------|--------|------|-------------------|
| 1 — Constant folding | small | low | yes |
| 2 — Per-instruction overhead | small | low | yes |
| 3 — Indexed variable slots | medium | medium | yes |
| 4 — Copy-friendly `ProgramValue` | large | medium | yes |

After each step, rerun `samply record` on the test binary and compare:

```
RUSTFLAGS="-C debuginfo=2" cargo test --features tui --release --no-run
cargo bin samply record --save-only -o /tmp/stage_d.profile.json -- \
    ./target/release/deps/incremental_code-<hash> \
    stage_d_gains_silver --nocapture --test-threads=1
cargo bin samply load /tmp/stage_d.profile.json
```

Stop as soon as the test drops below a target (e.g. <5 s) — further steps only pay off if other tests / real gameplay still feel slow.
