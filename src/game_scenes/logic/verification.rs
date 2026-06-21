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

fn verify_expr(expr: &NotPythonExpr, strings_allowed: bool, max_int: i64) -> anyhow::Result<()> {
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
        NotPythonExpr::Float(_)
        | NotPythonExpr::Boolean(_)
        | NotPythonExpr::None
        | NotPythonExpr::Variable(_) => {}
        NotPythonExpr::List(elems) => {
            for e in elems {
                verify_expr(e, strings_allowed, max_int)?;
            }
        }
        NotPythonExpr::Dict(pairs) => {
            for (k, v) in pairs {
                verify_expr(k, strings_allowed, max_int)?;
                verify_expr(v, strings_allowed, max_int)?;
            }
        }
        NotPythonExpr::BinaryOp(_, a, b) => {
            verify_expr(a, strings_allowed, max_int)?;
            verify_expr(b, strings_allowed, max_int)?;
        }
        NotPythonExpr::UnaryOp(_, e) => verify_expr(e, strings_allowed, max_int)?,
        NotPythonExpr::Call(_, args) => {
            for a in args {
                verify_expr(a, strings_allowed, max_int)?;
            }
        }
        NotPythonExpr::Index(_, idx) => {
            verify_expr(idx, strings_allowed, max_int)?;
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
                verify_expr(a, strings_allowed, max_int)?;
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
            verify_expr(expr, strings_allowed, max_int)?;
        }
        NotPythonStmt::If {
            condition,
            then,
            else_,
        } => {
            verify_expr(condition, strings_allowed, max_int)?;
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
                verify_expr(e, strings_allowed, max_int)?;
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
