use crate::game_scenes::base::SceneSwitch;
use crate::game_state::{CodeStatementLevels, Upgrades, with_game_state_mut};
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use itertools::Itertools;
use ratatui_widgets::paragraph::Paragraph;
use std::iter;

// Adds predefined code snippets as the current program code.
pub(super) fn cheat_pdcode_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    with_game_state_mut(|game_state| {
        game_state.program_code = get_predefined_code(&game_state.upgrades)
    });
    Box::new(ParagraphCmd::new(Paragraph::new("Overwrote program code")))
}

fn get_predefined_code(current_upgrades: &Upgrades) -> String {
    let width = current_upgrades.code_line_width.value() as usize;
    let lines = current_upgrades.code_line_count.value() as usize;
    let max_int_lit = current_upgrades.literals.value().1;

    match current_upgrades.statements.value() {
        CodeStatementLevels::None => {
            let line = "pass;";
            iter::repeat(line).take(lines).join("\n")
        }
        CodeStatementLevels::SimpleLoops => nested_loops_code(width, lines, max_int_lit, 1),
        CodeStatementLevels::NestedLoops
        | CodeStatementLevels::Functions
        | CodeStatementLevels::PureFunctions
        | CodeStatementLevels::SingleRecursion => {
            nested_loops_code(width, lines, max_int_lit, usize::MAX)
        }
        CodeStatementLevels::MultiRecursion => multi_recursion_code(width, lines, max_int_lit),
    }
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

fn nested_loops_code(width: usize, lines: usize, max_int_lit: u8, max_depth: usize) -> String {
    if lines < 8 || width < 9 {
        return iter::repeat("pass;").take(lines).join("\n");
    }
    let depth = std::cmp::min(max_depth, (lines - 1) / 7);
    let body_passes = lines - 7 * depth;
    let vars = ["i", "j", "k", "l", "m", "n", "o", "p", "q", "r"];
    let expr = counter_expr(width - 4, max_int_lit);
    build_nested_loop(&vars[..depth], &expr, body_passes)
}

fn build_nested_loop(vars: &[&str], expr: &str, body_passes: usize) -> String {
    if vars.is_empty() {
        return iter::repeat("pass;").take(body_passes).join("\n");
    }
    let var = vars[0];
    format!(
        "{var}:={expr};\nloop:\nif {var}==0:\nbreak;\nend\n{}\n{var}={var}-1;\nend",
        build_nested_loop(&vars[1..], expr, body_passes)
    )
}

fn multi_recursion_code(width: usize, lines: usize, max_int_lit: u8) -> String {
    if lines < 8 || width < 9 {
        return nested_loops_code(width, lines, max_int_lit, usize::MAX);
    }
    let k = std::cmp::max(2, lines - 6);
    let expr = counter_expr(width - 4, max_int_lit);
    let calls = iter::repeat("f(n-1);").take(k).join("\n");
    format!("def f(n):\nif n==0:\nreturn 0;\nend\n{calls}\nend\nf({expr});")
}
