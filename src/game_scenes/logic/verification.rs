use crate::game_state::{CodeStatementLevels, with_game_state};
use anyhow::bail;
use language::{NotPythonExpr, NotPythonProgram, NotPythonStmt};

/// Checks whether the code only uses features that have been unlocked.
pub fn verify_unlocks(source: &str, parsed_program: &NotPythonProgram) -> anyhow::Result<()> {
    struct Upgrades {
        line_width: u8,
        line_count: u8,
        literals: (bool, u8), // strings unlocked ; max int
        statements: CodeStatementLevels,
    }
    let upgrades = with_game_state(|game_state| Upgrades {
        line_width: game_state.upgrades.code_line_width.value(),
        line_count: game_state.upgrades.code_line_count.value(),
        literals: game_state.upgrades.literals.value(),
        statements: game_state.upgrades.statements.value(),
    });

    // Source-level checks
    let non_empty_lines = source.lines().filter(|l| !l.trim().is_empty()).count();
    if non_empty_lines > upgrades.line_count as usize {
        bail!(
            "Too many lines: {} used, {} allowed",
            non_empty_lines,
            upgrades.line_count
        );
    }
    for (i, line) in source.lines().enumerate() {
        if line.len() > upgrades.line_width as usize {
            bail!(
                "Line {} is too long: {} chars, max {}",
                i + 1,
                line.len(),
                upgrades.line_width
            );
        }
    }

    // AST-level checks
    let (strings_allowed, max_int) = upgrades.literals;
    verify_stmt(
        &parsed_program.statement,
        strings_allowed,
        max_int as i64,
        &upgrades.statements,
        0,
        None,
    )
}

/// Number of decimal places float literals may have, given the integer limit:
/// unlocking integers up to 10^n also unlocks floats with precision 10^-n.
/// `None` means float literals are not unlocked yet.
pub fn float_decimals(max_int: i64) -> Option<u32> {
    if max_int < 10 {
        return None;
    }
    Some((max_int.ilog10()).min(2))
}

fn verify_expr(
    expr: &NotPythonExpr,
    strings_allowed: bool,
    max_int: i64,
    level: &CodeStatementLevels,
) -> anyhow::Result<()> {
    let allows_multiplication = matches!(
        level,
        CodeStatementLevels::Multiplication
            | CodeStatementLevels::PureFunctions
            | CodeStatementLevels::SingleRecursion
            | CodeStatementLevels::MultiRecursion
    );

    match expr {
        NotPythonExpr::Int(n) => {
            let abs = n.unsigned_abs();
            if abs > max_int as u64 {
                bail!("Integer literal {n} exceeds max allowed value of {max_int}");
            }
        }
        NotPythonExpr::String(_) => {
            if !strings_allowed {
                bail!("String literals are not unlocked yet");
            }
        }
        NotPythonExpr::Float(f) => {
            let Some(decimals) = float_decimals(max_int) else {
                bail!("Float literals are not unlocked yet");
            };
            let scale = 10f64.powi(decimals as i32);
            if (f * scale - (f * scale).round()).abs() > 1e-9 * scale.max(1.0) {
                bail!("Float literal {f} exceeds allowed precision of 10^-{decimals}");
            }
            if f.abs() > max_int as f64 {
                bail!("Float literal {f} exceeds max allowed value of {max_int}");
            }
        }
        NotPythonExpr::Boolean(_) | NotPythonExpr::None | NotPythonExpr::Variable(_) => {}
        NotPythonExpr::List(elems) => {
            for e in elems {
                verify_expr(e, strings_allowed, max_int, level)?;
            }
        }
        NotPythonExpr::Dict(pairs) => {
            for (k, v) in pairs {
                verify_expr(k, strings_allowed, max_int, level)?;
                verify_expr(v, strings_allowed, max_int, level)?;
            }
        }
        NotPythonExpr::BinaryOp(op, a, b) => {
            if matches!(op, language::BinaryOp::Mul | language::BinaryOp::Div)
                && !allows_multiplication
            {
                bail!("Multiplication/division is not unlocked yet");
            }
            verify_expr(a, strings_allowed, max_int, level)?;
            verify_expr(b, strings_allowed, max_int, level)?;
        }
        NotPythonExpr::UnaryOp(_, e) => verify_expr(e, strings_allowed, max_int, level)?,
        NotPythonExpr::Call(_, args) => {
            for a in args {
                verify_expr(a, strings_allowed, max_int, level)?;
            }
        }
        NotPythonExpr::Index(_, idx) => {
            verify_expr(idx, strings_allowed, max_int, level)?;
        }
    }
    Ok(())
}

fn count_direct_calls(stmt: &NotPythonStmt, fn_name: &str) -> usize {
    match stmt {
        NotPythonStmt::Call(name, _) if name == fn_name => 1,
        NotPythonStmt::Call(_, _)
        | NotPythonStmt::Pass
        | NotPythonStmt::Break
        | NotPythonStmt::Continue => 0,
        NotPythonStmt::Block(stmts) => stmts.iter().map(|s| count_direct_calls(s, fn_name)).sum(),
        NotPythonStmt::Decl(_, _) | NotPythonStmt::Assign(_, _) => 0,
        NotPythonStmt::If { then, else_, .. } => {
            count_direct_calls(then, fn_name)
                + else_
                    .as_deref()
                    .map_or(0, |e| count_direct_calls(e, fn_name))
        }
        NotPythonStmt::Loop(body) => count_direct_calls(body, fn_name),
        NotPythonStmt::Return(_) => 0,
        NotPythonStmt::Function { body, .. } => count_direct_calls(body, fn_name),
    }
}

fn verify_stmt(
    stmt: &NotPythonStmt,
    strings_allowed: bool,
    max_int: i64,
    level: &CodeStatementLevels,
    loop_depth: u32,
    current_fn: Option<&str>,
) -> anyhow::Result<()> {
    let allows_loops = !matches!(level, CodeStatementLevels::None);
    let allows_nested_loops = !matches!(
        level,
        CodeStatementLevels::None | CodeStatementLevels::SimpleLoops
    );
    let allows_functions = matches!(
        level,
        CodeStatementLevels::Functions
            | CodeStatementLevels::PureFunctions
            | CodeStatementLevels::SingleRecursion
            | CodeStatementLevels::MultiRecursion
    );
    let allows_pure_functions = matches!(
        level,
        CodeStatementLevels::PureFunctions
            | CodeStatementLevels::SingleRecursion
            | CodeStatementLevels::MultiRecursion
    );

    match stmt {
        NotPythonStmt::Call(_, args) => {
            for a in args {
                verify_expr(a, strings_allowed, max_int, level)?;
            }
        }
        NotPythonStmt::Pass | NotPythonStmt::Break | NotPythonStmt::Continue => {
            if matches!(stmt, NotPythonStmt::Break | NotPythonStmt::Continue) && !allows_loops {
                bail!("break/continue requires the loops upgrade");
            }
        }
        NotPythonStmt::Block(stmts) => {
            for s in stmts {
                verify_stmt(s, strings_allowed, max_int, level, loop_depth, current_fn)?;
            }
        }
        NotPythonStmt::Decl(_, expr) | NotPythonStmt::Assign(_, expr) => {
            verify_expr(expr, strings_allowed, max_int, level)?;
        }
        NotPythonStmt::If {
            condition,
            then,
            else_,
        } => {
            verify_expr(condition, strings_allowed, max_int, level)?;
            verify_stmt(
                then,
                strings_allowed,
                max_int,
                level,
                loop_depth,
                current_fn,
            )?;
            if let Some(e) = else_ {
                verify_stmt(e, strings_allowed, max_int, level, loop_depth, current_fn)?;
            }
        }
        NotPythonStmt::Loop(body) => {
            if !allows_loops {
                bail!("Loops are not unlocked yet");
            }
            if !allows_nested_loops && loop_depth >= 1 {
                bail!("Nested loops are not unlocked yet");
            }
            verify_stmt(
                body,
                strings_allowed,
                max_int,
                level,
                loop_depth + 1,
                current_fn,
            )?;
        }
        NotPythonStmt::Return(expr) => {
            if !allows_functions {
                bail!("return requires the functions upgrade");
            }
            if let Some(e) = expr {
                verify_expr(e, strings_allowed, max_int, level)?;
            }
        }
        NotPythonStmt::Function {
            name,
            body,
            is_pure,
            ..
        } => {
            if !allows_functions {
                bail!("Function definitions are not unlocked yet");
            }
            if *is_pure && !allows_pure_functions {
                bail!("Pure functions are not unlocked yet");
            }
            let self_calls = count_direct_calls(body, name);
            let max_recursive = match level {
                CodeStatementLevels::Functions | CodeStatementLevels::PureFunctions => 0,
                CodeStatementLevels::SingleRecursion => 1,
                CodeStatementLevels::MultiRecursion => usize::MAX,
                _ => 0,
            };
            if self_calls > max_recursive {
                if max_recursive == 0 {
                    bail!("Recursion is not unlocked yet (function '{name}' calls itself)");
                } else {
                    bail!(
                        "Function '{name}' has {self_calls} recursive calls, max allowed is {max_recursive}"
                    );
                }
            }
            verify_stmt(body, strings_allowed, max_int, level, 0, Some(name))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::{Upgrade, with_game_state_mut};
    use serial_test::serial;

    /// Levels up (and, after `f` runs, levels back down) the upgrades that
    /// `verify_unlocks` reads from global game state, then runs `f`.
    fn with_levels<T>(literals_level: u8, line_width_level: u8, f: impl FnOnce() -> T) -> T {
        with_levels_and_statements(literals_level, line_width_level, 0, f)
    }

    /// Same as `with_levels`, but also levels up the `statements` (`CodeStatementLevels`)
    /// upgrade by `statements_level` steps.
    fn with_levels_and_statements<T>(
        literals_level: u8,
        line_width_level: u8,
        statements_level: u8,
        f: impl FnOnce() -> T,
    ) -> T {
        with_game_state_mut(|state| {
            for _ in 0..literals_level {
                state.upgrades.literals.track_level_up(0);
            }
            for _ in 0..line_width_level {
                state.upgrades.code_line_width.track_level_up(0);
            }
            for _ in 0..statements_level {
                state.upgrades.statements.track_level_up(0);
            }
        });
        let result = f();
        with_game_state_mut(|state| {
            for _ in 0..literals_level {
                state.upgrades.literals.track_level_down(0);
            }
            for _ in 0..line_width_level {
                state.upgrades.code_line_width.track_level_down(0);
            }
            for _ in 0..statements_level {
                state.upgrades.statements.track_level_down(0);
            }
        });
        result
    }

    #[test]
    #[serial]
    fn folded_literal_product_exceeding_limit_is_allowed() {
        // Each raw literal (255) is within the max_int=255 limit even though the
        // folded product (65025) would exceed it. Verification must run on the
        // unfolded AST, so this must be accepted.
        with_levels_and_statements(6, 3, 4, || {
            let source = "x := 255 * 255;\n";
            let parsed = language::parse_program(source).unwrap();
            assert!(verify_unlocks(source, &parsed).is_ok());
        });
    }

    #[test]
    #[serial]
    fn raw_literal_exceeding_limit_is_still_rejected() {
        with_levels(6, 3, || {
            let source = "x := 999;\n";
            let parsed = language::parse_program(source).unwrap();
            assert!(verify_unlocks(source, &parsed).is_err());
        });
    }

    fn float_ok(literals_level: u8, source: &str) -> bool {
        with_levels(literals_level, 3, || {
            let parsed = language::parse_program(source).unwrap();
            verify_unlocks(source, &parsed).is_ok()
        })
    }

    #[test]
    fn float_decimals_follow_integer_limit() {
        assert_eq!(float_decimals(5), None);
        assert_eq!(float_decimals(10), Some(1));
        assert_eq!(float_decimals(100), Some(2));
        assert_eq!(float_decimals(255), Some(2));
    }

    #[test]
    #[serial]
    fn floats_unlock_with_integer_limit() {
        assert!(!float_ok(2, "x := 0.5;\n")); // max_int 5
        assert!(float_ok(3, "x := 0.5;\n")); // max_int 10: 1 decimal
        assert!(!float_ok(3, "x := 0.25;\n"));
        assert!(float_ok(5, "x := 0.25;\n")); // max_int 100: 2 decimals
        assert!(!float_ok(5, "x := 0.125;\n"));
    }

    #[test]
    #[serial]
    fn multiplication_is_rejected_when_not_unlocked() {
        with_levels(6, 3, || {
            let source = "x := 2 * 3;\n";
            let parsed = language::parse_program(source).unwrap();
            assert!(verify_unlocks(source, &parsed).is_err());
        });
    }

    #[test]
    #[serial]
    fn division_is_rejected_when_not_unlocked() {
        with_levels(6, 3, || {
            let source = "x := 6 / 2;\n";
            let parsed = language::parse_program(source).unwrap();
            assert!(verify_unlocks(source, &parsed).is_err());
        });
    }

    #[test]
    #[serial]
    fn multiplication_is_allowed_once_unlocked() {
        // statements level 4 == CodeStatementLevels::Multiplication.
        with_levels_and_statements(6, 3, 4, || {
            let source = "x := 2 * 3;\n";
            let parsed = language::parse_program(source).unwrap();
            assert!(verify_unlocks(source, &parsed).is_ok());
        });
    }

    #[test]
    #[serial]
    fn addition_and_subtraction_are_unaffected_by_multiplication_lock() {
        with_levels(6, 3, || {
            let source = "x := 2 + 3 - 1;\n";
            let parsed = language::parse_program(source).unwrap();
            assert!(verify_unlocks(source, &parsed).is_ok());
        });
    }
}
