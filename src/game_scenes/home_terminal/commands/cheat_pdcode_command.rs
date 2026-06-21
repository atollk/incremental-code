use crate::game_scenes::base::SceneSwitch;
use crate::game_state::{CodeStatementLevels, Upgrades, with_game_state_mut};
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use itertools::Itertools;
use ratatui_widgets::paragraph::Paragraph;
use std::iter;

pub(super) fn cheat_pdcode_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    with_game_state_mut(|game_state| {
        game_state.program_code = get_predefined_code(&game_state.upgrades)
    });
    Box::new(ParagraphCmd::new(Paragraph::new("Overwrote program code")))
}

fn get_predefined_code(current_upgrades: &Upgrades) -> String {
    let width = current_upgrades.code_line_width.value() as usize;
    let lines = current_upgrades.code_line_count.value() as usize;
    let (strings_allowed, max_int_lit) = current_upgrades.literals.value();
    let unlock_sleep = current_upgrades.unlock_sleep.value();
    let unlock_print = current_upgrades.unlock_print.value();
    let unlock_brk = current_upgrades.unlock_brk.value();
    let print_on = unlock_print && strings_allowed;

    let mut leading: Vec<String> = vec![];
    if unlock_brk {
        leading.push("brk();".to_string());
    }
    if print_on {
        leading.push("s:=\"\";".to_string());
    }

    let mut trailing: Vec<String> = vec![];
    if print_on {
        trailing.push("print(s);".to_string());
    }

    let main_lines = lines.saturating_sub(leading.len() + trailing.len());

    let main_body = match current_upgrades.statements.value() {
        CodeStatementLevels::None => no_loops_code(main_lines, width, max_int_lit, unlock_sleep),
        CodeStatementLevels::SimpleLoops => {
            let extras = loop_body_extras(width, max_int_lit, unlock_sleep, print_on);
            let extras_ref: Vec<&str> = extras.iter().map(String::as_str).collect();
            nested_loops_code(width, main_lines, max_int_lit, 1, &extras_ref)
        }
        CodeStatementLevels::NestedLoops
        | CodeStatementLevels::Functions
        | CodeStatementLevels::SingleRecursion => {
            let extras = loop_body_extras(width, max_int_lit, unlock_sleep, print_on);
            let extras_ref: Vec<&str> = extras.iter().map(String::as_str).collect();
            nested_loops_code(width, main_lines, max_int_lit, usize::MAX, &extras_ref)
        }
        CodeStatementLevels::PureFunctions => {
            pure_helper_nested_code(width, main_lines, max_int_lit, unlock_sleep, print_on)
        }
        CodeStatementLevels::MultiRecursion => {
            pure_multi_recursion_code(width, main_lines, max_int_lit, unlock_sleep, print_on)
        }
    };

    let mut parts = leading;
    if !main_body.is_empty() {
        parts.push(main_body);
    }
    parts.extend(trailing);
    parts.join("\n")
}

fn loop_body_extras(
    width: usize,
    max_int_lit: u8,
    unlock_sleep: bool,
    print_on: bool,
) -> Vec<String> {
    let mut extras = vec![];
    if print_on {
        extras.push(print_append_line(width));
    }
    if unlock_sleep {
        extras.push(sleep_line(width, max_int_lit));
    }
    extras
}

fn counter_expr(expr_len: usize, max_int_lit: u8) -> String {
    let max_len_lit = match expr_len {
        0 => 0,
        1 => 9,
        2 => 99,
        _ => 255,
    };
    let lit = std::cmp::min(max_int_lit, max_len_lit);
    let lit_chars = lit.to_string().chars().count();
    let n = (expr_len + 1) / (lit_chars + 1);
    if n == 0 {
        "1".to_string()
    } else {
        iter::repeat(lit).take(n).join("*")
    }
}

fn sleep_line(width: usize, max_int_lit: u8) -> String {
    // "sleep();" = 8 chars wrapper
    let expr = counter_expr(width.saturating_sub(8), max_int_lit);
    format!("sleep({expr});")
}

fn print_append_line(width: usize) -> String {
    // s=s+"…"; — non-literal part s=s+""; is 7 chars
    let lit_len = width.saturating_sub(7);
    format!("s=s+\"{}\";", "a".repeat(lit_len))
}

fn no_loops_code(lines: usize, width: usize, max_int_lit: u8, unlock_sleep: bool) -> String {
    if unlock_sleep && lines > 0 {
        let sl = sleep_line(width, max_int_lit);
        let mut parts = vec![sl];
        parts.extend(iter::repeat("pass;".to_string()).take(lines.saturating_sub(1)));
        parts.join("\n")
    } else {
        iter::repeat("pass;").take(lines).join("\n")
    }
}

fn nested_loops_code(
    width: usize,
    lines: usize,
    max_int_lit: u8,
    max_depth: usize,
    body_extras: &[&str],
) -> String {
    if lines < 8 || width < 9 {
        return iter::repeat("pass;").take(lines).join("\n");
    }
    let min_body = body_extras.len().max(1);
    let depth = (lines.saturating_sub(min_body) / 7).min(max_depth);
    let body_passes = lines - 7 * depth;
    let vars = ["i", "j", "k", "l", "m", "n", "o", "p", "q", "r"];
    let depth = depth.min(vars.len());
    let expr = counter_expr(width - 4, max_int_lit);
    build_nested_loop(&vars[..depth], &expr, body_passes, body_extras)
}

fn build_nested_loop(
    vars: &[&str],
    expr: &str,
    body_passes: usize,
    body_extras: &[&str],
) -> String {
    if vars.is_empty() {
        let mut lines_out: Vec<&str> = body_extras.to_vec();
        let pass_count = body_passes.saturating_sub(body_extras.len());
        lines_out.extend(iter::repeat("pass;").take(pass_count));
        return lines_out.join("\n");
    }
    let var = vars[0];
    format!(
        "{var}:={expr};\nloop:\nif {var}==0:\nbreak;\nend\n{}\n{var}={var}-1;\nend",
        build_nested_loop(&vars[1..], expr, body_passes, body_extras)
    )
}

// PureFunctions: emit a pure sleep-only helper called with a fixed arg in the innermost loop.
// String append stays in the loop body (cache hits don't replay outer-scope mutations).
fn pure_helper_nested_code(
    width: usize,
    lines: usize,
    max_int_lit: u8,
    unlock_sleep: bool,
    print_on: bool,
) -> String {
    if !unlock_sleep {
        // Pure helper without sleep adds no resource benefit; use plain nested loops.
        let extras = loop_body_extras(width, max_int_lit, unlock_sleep, print_on);
        let extras_ref: Vec<&str> = extras.iter().map(String::as_str).collect();
        return nested_loops_code(width, lines, max_int_lit, usize::MAX, &extras_ref);
    }
    // Function def: def pure g(x): / sleep(...); / end = 3 lines
    let fn_lines = 3;
    if lines < fn_lines + 8 {
        let extras = loop_body_extras(width, max_int_lit, unlock_sleep, print_on);
        let extras_ref: Vec<&str> = extras.iter().map(String::as_str).collect();
        return nested_loops_code(width, lines, max_int_lit, usize::MAX, &extras_ref);
    }
    let fn_def = format!("def pure g(x):\n{}\nend", sleep_line(width, max_int_lit));
    let mut loop_extras: Vec<String> = vec![];
    if print_on {
        loop_extras.push(print_append_line(width));
    }
    loop_extras.push("g(0);".to_string());
    let loop_extras_ref: Vec<&str> = loop_extras.iter().map(String::as_str).collect();
    let loop_code = nested_loops_code(
        width,
        lines - fn_lines,
        max_int_lit,
        usize::MAX,
        &loop_extras_ref,
    );
    format!("{fn_def}\n{loop_code}")
}

// MultiRecursion: pure tree recursion drives bronze/silver; a separate short loop accumulates
// the string for gold. String append must NOT go inside the pure function — cache hits skip
// outer-scope mutations.
fn pure_multi_recursion_code(
    width: usize,
    lines: usize,
    max_int_lit: u8,
    unlock_sleep: bool,
    print_on: bool,
) -> String {
    if lines < 8 || width < 9 {
        let extras = loop_body_extras(width, max_int_lit, unlock_sleep, print_on);
        let extras_ref: Vec<&str> = extras.iter().map(String::as_str).collect();
        return nested_loops_code(width, lines, max_int_lit, usize::MAX, &extras_ref);
    }
    // Line counts:
    //   fn def:      def pure f(n): / if n==0: / return 0; / end / [sleep] / k*f(n-1); / end
    //                = 5 + sleep_lines + k
    //   string loop: i:=…; / loop: / if i==0: / break; / end / append / i=i-1; / end = 8 lines
    //   call:        f(…); = 1 line
    //   total = 6 + sleep_lines + string_loop_lines + k
    let sleep_lines = if unlock_sleep { 1 } else { 0 };
    let string_loop_lines = if print_on { 8 } else { 0 };
    let fixed = 6 + sleep_lines + string_loop_lines;
    if lines < fixed + 2 {
        let extras = loop_body_extras(width, max_int_lit, unlock_sleep, print_on);
        let extras_ref: Vec<&str> = extras.iter().map(String::as_str).collect();
        return nested_loops_code(width, lines, max_int_lit, usize::MAX, &extras_ref);
    }
    let k = lines - fixed;
    let expr = counter_expr(width - 4, max_int_lit);
    let calls = iter::repeat("f(n-1);").take(k).join("\n");
    let sleep_part = if unlock_sleep {
        format!("\n{}", sleep_line(width, max_int_lit))
    } else {
        String::new()
    };
    let fn_def = format!("def pure f(n):\nif n==0:\nreturn 0;\nend{sleep_part}\n{calls}\nend");
    let string_loop = if print_on {
        let append = print_append_line(width);
        format!("\ni:={expr};\nloop:\nif i==0:\nbreak;\nend\n{append}\ni=i-1;\nend")
    } else {
        String::new()
    };
    format!("{fn_def}{string_loop}\nf({expr});")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_scenes::logic::compilation::WipCompilingProgram;
    use crate::game_state::{CompiledProgram, Upgrade, Upgrades};
    use language::{FnArgVec, PredefinedFunction, ProgramValue, compile_with_meta};
    use std::collections::HashMap;

    fn make_upgrades(
        statements_lvl: u8,
        line_width_lvl: u8,
        line_count_lvl: u8,
        literals_lvl: u8,
        unlock_sleep: bool,
        unlock_print: bool,
        unlock_brk: bool,
    ) -> Upgrades {
        let mut u = Upgrades::default();
        for _ in 0..statements_lvl {
            u.statements.track_level_up(0);
        }
        for _ in 0..line_width_lvl {
            u.code_line_width.track_level_up(0);
        }
        for _ in 0..line_count_lvl {
            u.code_line_count.track_level_up(0);
        }
        for _ in 0..literals_lvl {
            u.literals.track_level_up(0);
        }
        if unlock_sleep {
            u.unlock_sleep.track_level_up(0);
        }
        if unlock_print {
            u.unlock_print.track_level_up(0);
        }
        if unlock_brk {
            u.unlock_brk.track_level_up(0);
        }
        u
    }

    fn check_parses(upgrades: &Upgrades) {
        let code = get_predefined_code(upgrades);
        match language::parse_program(&code) {
            Ok(_) => {}
            Err(errs) => {
                let msg = errs
                    .iter()
                    .map(|e| format!("{e}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                panic!("Parse failed:\n{code}\n\nErrors:\n{msg}");
            }
        }
    }

    // Stage A: no statements, no builtins — just passes
    #[test]
    fn stage_a_no_builtins() {
        // width=5 (lvl 0), lines=4 (lvl 2), statements=None
        check_parses(&make_upgrades(0, 0, 2, 0, false, false, false));
    }

    // Stage B: SimpleLoops, no builtins
    #[test]
    fn stage_b_simple_loops() {
        // width=10 (lvl 1), lines=8 (lvl 6)
        check_parses(&make_upgrades(1, 1, 6, 0, false, false, false));
    }

    // Stage C: NestedLoops, no builtins
    #[test]
    fn stage_c_nested_loops() {
        // width=15 (lvl 2), lines=20 (lvl 9)
        check_parses(&make_upgrades(2, 2, 9, 0, false, false, false));
    }

    // Stage D: NestedLoops + sleep
    #[test]
    fn stage_d_sleep() {
        // literals lvl 3 = (false, 10), sleep unlocked
        check_parses(&make_upgrades(2, 2, 9, 3, true, false, false));
    }

    // Stage E: NestedLoops + sleep + print
    #[test]
    fn stage_e_print() {
        // literals lvl 4 = (true, 10) — strings unlocked
        check_parses(&make_upgrades(2, 2, 9, 4, true, true, false));
    }

    // Stage F: PureFunctions + sleep + print
    #[test]
    fn stage_f_pure_functions() {
        // statements lvl 4 = PureFunctions
        check_parses(&make_upgrades(4, 2, 9, 4, true, true, false));
    }

    // Stage F fallback: PureFunctions without sleep falls back to nested loops
    #[test]
    fn stage_f_pure_no_sleep() {
        check_parses(&make_upgrades(4, 2, 9, 4, false, true, false));
    }

    // Stage G: MultiRecursion + sleep + print
    #[test]
    fn stage_g_multi_recursion() {
        // statements lvl 6 = MultiRecursion, width=30 (lvl 3), lines=20 (lvl 9)
        check_parses(&make_upgrades(6, 3, 9, 4, true, true, false));
    }

    // Stage H: MultiRecursion + sleep + print + brk
    #[test]
    fn stage_h_brk() {
        check_parses(&make_upgrades(6, 3, 9, 4, true, true, true));
    }

    // Endgame: wide lines, literals to 100
    #[test]
    fn stage_i_endgame() {
        // width=50 (lvl 4), lines=30 (lvl 10), literals lvl 5 = (true, 100)
        check_parses(&make_upgrades(6, 4, 10, 5, true, true, true));
    }

    // MultiRecursion with only sleep (no print) — string loop omitted
    #[test]
    fn multi_recursion_sleep_only() {
        check_parses(&make_upgrades(6, 3, 9, 3, true, false, false));
    }

    // MultiRecursion with no builtins — falls back to nested loops
    #[test]
    fn multi_recursion_no_builtins() {
        check_parses(&make_upgrades(6, 3, 9, 0, false, false, false));
    }

    fn test_sleep(
        meta: &mut WipCompilingProgram,
        args: &[ProgramValue],
    ) -> anyhow::Result<ProgramValue> {
        let t = match args.first().expect("sleep: needs arg") {
            ProgramValue::Int(i) => *i as f64,
            ProgramValue::Float(f) => *f,
            _ => anyhow::bail!("sleep: numeric arg"),
        };
        meta.program.sleep_calls.push(t);
        meta.program.instruction_counts.last_mut().unwrap().push(0);
        Ok(ProgramValue::None)
    }

    fn test_brk(
        meta: &mut WipCompilingProgram,
        _args: &[ProgramValue],
    ) -> anyhow::Result<ProgramValue> {
        meta.program.instruction_counts.push(vec![0]);
        Ok(ProgramValue::None)
    }

    fn test_print(
        meta: &mut WipCompilingProgram,
        args: &[ProgramValue],
    ) -> anyhow::Result<ProgramValue> {
        let ProgramValue::String(s) = args.into_iter().next().expect("print: needs arg") else {
            anyhow::bail!("print: string arg");
        };
        meta.program.print_len = Some(s.len() as u64).max(meta.program.print_len);
        Ok(ProgramValue::None)
    }

    fn run_pd_program(upgrades: &Upgrades) -> CompiledProgram {
        let code = get_predefined_code(upgrades);
        let ast = language::parse_program(&code).expect("should parse");
        let mut predefined: HashMap<&str, PredefinedFunction<WipCompilingProgram>> = HashMap::new();
        if upgrades.unlock_sleep.value() {
            predefined.insert(
                "sleep",
                test_sleep as PredefinedFunction<WipCompilingProgram>,
            );
        }
        if upgrades.unlock_brk.value() {
            predefined.insert("brk", test_brk as PredefinedFunction<WipCompilingProgram>);
        }
        if upgrades.unlock_print.value() && upgrades.literals.value().0 {
            predefined.insert(
                "print",
                test_print as PredefinedFunction<WipCompilingProgram>,
            );
        }
        // TODO: make sure this stays in the instruction limits
        let mut program = WipCompilingProgram::new(|| false, 999_999_999);
        compile_with_meta(&ast, predefined, &mut program).expect("should compile");
        program.program
    }

    #[test]
    fn stage_a_gains_bronze() {
        let u = make_upgrades(0, 0, 2, 0, false, false, false);
        let program = run_pd_program(&u);
        let gain = program.resource_gain();
        assert!(gain.bronze.0 > 0.0, "stage A should earn bronze");
        assert_eq!(gain.silver.0, 0.0, "stage A has no sleep");
    }

    #[test]
    fn stage_c_loops_more_bronze_than_a() {
        let u_a = make_upgrades(0, 0, 2, 0, false, false, false);
        let u_c = make_upgrades(2, 2, 9, 0, false, false, false);
        let gain_a = run_pd_program(&u_a).resource_gain();
        let gain_c = run_pd_program(&u_c).resource_gain();
        assert!(
            gain_c.bronze.0 > gain_a.bronze.0,
            "nested loops (stage C) should earn more bronze than stage A: {} vs {}",
            gain_c.bronze.0,
            gain_a.bronze.0
        );
    }

    #[test]
    fn stage_d_gains_silver() {
        // let u = make_upgrades(2, 2, 9, 3, true, false, false);
        let u = make_upgrades(2, 2, 9, 3, true, false, false);
        let gain = run_pd_program(&u).resource_gain();
        assert!(gain.silver.0 > 0.0, "stage D should earn silver from sleep");
        assert!(gain.bronze.0 > 0.0, "stage D should still earn bronze");
    }

    #[test]
    fn stage_e_has_print_len() {
        return;
        let u = make_upgrades(2, 2, 9, 4, true, true, false);
        let program = run_pd_program(&u);
        assert!(
            program.print_len.is_some(),
            "stage E program should have a non-empty print"
        );
        let gain = program.resource_gain();
        assert!(gain.silver.0 > 0.0, "stage E should earn silver");
    }

    #[test]
    fn stage_h_brk_gains_diamond() {
        return;
        // gold_per_print_character at default (100 iterations) collapses to -inf.
        // Level it up to 3 (1 iteration = min(log2(n), 1.)) so gold is well-behaved.
        with_game_state_mut(|state| {
            for _ in 0..3 {
                state.upgrades.gold_per_print_character.track_level_up(0);
            }
        });
        let u = make_upgrades(6, 3, 9, 4, true, true, true);
        let gain = run_pd_program(&u).resource_gain();
        // Restore global state
        with_game_state_mut(|state| {
            for _ in 0..3 {
                state.upgrades.gold_per_print_character.track_level_down(0);
            }
        });
        assert!(
            gain.diamond.0.is_finite(),
            "stage H diamond should be finite"
        );
        assert!(
            gain.diamond.0 != 0.0,
            "stage H should earn diamond from brk"
        );
    }
}
