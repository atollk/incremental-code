# Upgrades & Language Reference

Self-contained summary so future work on the "max instructions per upgrade
combination" estimator does not need to re-read source. Sources of truth at
the time of writing:

- `src/game_state/upgrades.rs`
- `src/game_state/program.rs`
- `language/src/{lexer,parser,compile}.rs`

---

## 1. Instruction model (what we are estimating)

The estimator's job: given a configuration of `Upgrades`, find the maximum
number of *atomic instructions* a program can execute.

### 1.1 What counts as an instruction

`CompilingMetadata` has two callbacks; only `log_atomic_instruction`
increments the counter and the limit. `log_zero_instruction` is a no-op for
the counter.

In `compile.rs`, `compile_stmt` calls `log_atomic_instruction` exactly once
per statement of these kinds:

| Statement              | Instructions logged                                                  |
| ---------------------- | -------------------------------------------------------------------- |
| `Pass`                 | 1                                                                    |
| `Break`                | 1 (sets control-flow to Break)                                       |
| `Continue`             | 1 (sets control-flow to Continue)                                    |
| `Return(expr?)`        | 1 (after evaluating `expr` if present)                               |
| `Decl(name, expr)`     | 1 (after evaluating RHS)                                             |
| `Assign(name, expr)`   | 1 (after evaluating RHS)                                             |
| `If { cond, then, else_ }` | 1, in addition to whatever the taken branch logs               |
| `Loop(body)`           | 1 on entry, then body executes repeatedly (each pass logs its own)   |
| `Function { ... }`     | 1 (definition only — registers the function)                         |
| `Call(name, args)`     | 1 *plus* the entire callee body's instructions                       |
| `Block(stmts)`         | 0 (transparent; children log)                                        |

Notes / subtleties:

- Expression evaluation is **free**. Arithmetic, comparison, list/dict
  literals, indexing, function calls inside expressions — none of these
  log atomic instructions on their own. Only the enclosing statement does.
- However, an `Expr::Call` *inside* an expression still goes through
  `Callable::call`, which executes the callee body. That body's statements
  log normally. So `x := foo();` costs `1 (decl) + body_cost(foo)`.
- A `Call` statement (`foo();`) costs `1 + body_cost(foo)`. The `+1` is
  separate from any instruction the body logs.
- After a `Return` is in flight, `compile_stmt` short-circuits at the top
  with a `log_zero_instruction` (no atomic increment). Subsequent statements
  in the same block contribute 0.
- `If` always logs 1, regardless of which branch (or none) is taken. The
  taken branch then logs additionally on top of that.
- `Loop` entry logs 1; each iteration's body logs its own instructions.
  `break`/`continue` inside a loop log 1 (as statements) — they only affect
  control flow, the limit applies the same.

---

## 2. Language (NotPython)

### 2.2 Tokens (full set)

Comments (`# ...`), Int, Float, StringLiteral, Newline, LineContinuation
(`\` + newline), LexError.

Keywords: `False`, `None`, `True`, `and`, `break`, `continue`, `def`,
`elif`, `else`, `if`, `in`, `not`, `or`, `pass`, `return`, `loop`, `end`.

Operators: `==`, `!=`, `<=`, `>=`, `:=`, `<`, `>`, `+`, `-`, `*`, `/`,
`%`, `=`.

Delimiters: `( ) [ ] { } , : . ;`.

Whitespace: horizontal whitespace skipped; newlines are tokens.

### 2.3 Grammar (informal, from `parser.rs`)

```
program     := stmt*
stmt        := pass | break | continue | return | decl | assign
             | call_stmt | if_stmt | loop_stmt | func_stmt
pass        := "pass" ";" line_end
break       := "break" ";" line_end
continue    := "continue" ";" line_end
return      := "return" expr? ";" line_end
decl        := IDENT ":=" expr ";" line_end
assign      := IDENT "=" expr ";" line_end
call_stmt   := IDENT "(" args? ")" ";" line_end
if_stmt     := "if" expr ":" NL block ("elif" expr ":" NL block)*
               ("else" ":" NL block)? "end" line_end
loop_stmt   := "loop" ":" NL block "end" line_end
func_stmt   := "def" IDENT "(" params? ")" ":" NL block "end" line_end
block       := stmt*               # (no separator; each stmt has terminator)
line_end    := NEWLINE | <eoi>
```

Note that most statements require a Newline to follow, so you cannot stack them in one line.

Expression precedence (low → high binding):
`or` < `and` < `not` < `in` < (`==` `!=` `<` `>` `<=` `>=`) < `%` <
(`+` `-`) < (`*` `/`) < unary `-` < atoms / calls / indexing / lists /
dicts.

`elif` chains desugar at parse time into nested `If` statements in the
else branch.

### 2.4 AST node summary

`NotPythonStmt`: `Call`, `Pass`, `Block`, `Decl`, `Assign`, `If`, `Loop`,
`Return`, `Break`, `Continue`, `Function { name, params, body }`.

`NotPythonExpr`: `Int(i64)`, `Float(f64)`, `String`, `Boolean`, `None`,
`Identifier`, `List`, `Dict`, `Op(NotPythonExprOp)`, `Call(name, args)`,
`Index(coll, idx)`.

`NotPythonExprOp`: arithmetic (Add/Sub/Mul/Div/Mod/Neg), boolean
(And/Or/Not), comparison (Equal/NotEqual/Greater/Less/GreaterEqual/LessEqual),
membership (In).

### 2.5 Runtime values

```
ProgramValue = Hashable(Int|String|Bool) | Float(f64) | None
             | List(Vec<ProgramValue>) | Dict(HashMap<Hashable,ProgramValue>)
```

Type rules (compile-time errors / runtime aborts):

- `+` numeric+numeric (int promotes to float on mix), or string+string.
- `- * /` numeric only; `/` errors on int÷0.
- `%` ints only; errors on mod 0.
- `and / or / not` booleans only.
- Comparisons (`< > <= >=`) numeric only. `== !=` work across hashables,
  floats, and `None == None`; otherwise return `false` (`!=` returns `true`).
- `in` requires list (any element) or dict (hashable key).
- `if` condition must be a `Bool`.
- `Index` works on lists (int index, range-checked) and dicts (hashable key).
- Variables: lookup walks call stack from top (innermost) outward. `Decl`
  inserts into the topmost frame; `Assign` updates the innermost frame
  that already binds the name (errors if no binding exists).
- Functions: `def` registers in the topmost frame (so nested `def` is
  scoped). `Call` first searches the call stack for user functions, then
  falls back to `predefined_functions` provided by the embedder.
- Function call: arg count must match params; a fresh frame is pushed
  with params bound; body runs; frame popped; `Return(v)` is consumed
  and returned (default `None`).
- Control flow tokens (`Break` / `Continue` / `Return`) are stored on the
  shared `state.control_flow` and propagate up through `compile_stmt`
  until the appropriate construct catches them. Once `Return` is set,
  every further statement in the function short-circuits as a zero-cost
  no-op until the call frame is popped.

`Loop` is the **only** unbounded construct. There are no for-loops, no
recursion limit other than the instruction cap, no time limit other than
the cap.

---

## 3. Upgrades

### 3.2 Group 1 — Compile/run mechanics + first language slice

| Upgrade                     | Levels (value, cost-to-next)                                                                                       |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| CodeLineWidth               | 5, 10, 15, 30, 50, 80                                                                                              |
| CodeLineCount               | 1, 2, 4, 4, 6, 6, 8, 10, 10, 20, 30, 40 (display labels include "5", "7", "15"; underlying values as listed)       |
| CodeExpressionLiterals      | (), 7 levels labeled "0, 1", "2", "3, 4, 5", "6-10", "numbers to 100", "numbers to 255", "empty strings"           |

Estimator-relevant from group 1:

- **CodeLineWidth / CodeLineCount** bound the program's source size in
  characters × lines — an upper bound on AST size, which combined with
  the limited literal vocabulary, bounds how complex a program can be.
- **CodeExpressionLiterals** restricts which numeric/string literals are
  *allowed in the editor*. By level, available literals are cumulative:
  L0: `0, 1` · L1: + `2` · L2: + `3, 4, 5` · L3: + `6..=10` · L4: +
  numbers up to 100 · L5: + numbers up to 255 · L6: + empty strings (`""`).
  No literal above 255 is ever available; floats and non-empty string
  literals are never available. This shapes how counters / indices in
  generated programs can be initialized.

### 3.3 Group 2 — Resource gain + statements

| Upgrade               | Levels                                                                                                                           |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| CodeStatements        | 3 levels: base (no extra), "loops", "functions"                                                                                  |

Estimator-relevant:

- **CodeStatements** is the big one for max-instruction shape:
  - Level 0: base statement set only (pass, decl, assign, if/elif/else,
    return, break, continue, function calls — but no `def` and no `loop`
    until later levels gate them).
  - Level 1 ("loops"): `loop:` statement becomes available.
  - Level 2 ("functions"): `def` statement and user-defined functions
    available.

  Without loops, max instructions ≤ source-size-bounded constant (no way
  to spend instructions in a loop). Without functions but with loops,
  programs can still hit the cap trivially (`loop: pass; end`) — so once
  loops are unlocked, the cap is the answer for any nontrivial layout.
  Without loops *and* without functions, the answer is the count of
  statements actually expressible in the editor (bounded by line width
  × line count and by the literal set).

### 3.4 Group 3 — Auto-compile, print, currency for prints

| Upgrade                 | Levels                                                                                            |
| ----------------------- | ------------------------------------------------------------------------------------------------- |
| UnlockPrint             | locked → unlocked — adds `print` predefined function (player-visible)                             |

`UnlockPrint` is the only one here that may add a builtin (`print`) to the
predefined functions map. `print` itself, if present, is a `Call` whose
body runs the host-side function — that's *one* atomic increment for the
call statement, plus whatever the host function reports via
`log_atomic_instruction` (today, none — the host `PredefinedFunction` is
free to log nothing). For the estimator this means a `print(...)` call
costs 1 (Call statement) + 0 (call into host).
