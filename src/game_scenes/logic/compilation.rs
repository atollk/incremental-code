use crate::game_state::{CompiledProgram, with_game_state, with_game_state_mut};
use anyhow::anyhow;
use language::{PredefinedFunction, ProgramValue, compile_with_meta, parse_program};
use std::collections::HashMap;

fn predefined_function_print(
    _meta: &mut CompiledProgram,
    _args: Vec<ProgramValue>,
) -> anyhow::Result<ProgramValue> {
    todo!()
}

fn predefined_function_sleep(
    _meta: &mut CompiledProgram,
    _args: Vec<ProgramValue>,
) -> anyhow::Result<ProgramValue> {
    todo!()
}

fn predefined_function_brk(
    _meta: &mut CompiledProgram,
    _args: Vec<ProgramValue>,
) -> anyhow::Result<ProgramValue> {
    todo!()
}

fn predefined_functions() -> HashMap<&'static str, &'static PredefinedFunction<CompiledProgram>> {
    let (unlock_print, unlock_sleep, unlock_brk) = with_game_state(|game_state| {
        (
            game_state.upgrades.unlock_print.value(),
            game_state.upgrades.unlock_sleep.value(),
            game_state.upgrades.unlock_brk.value(),
        )
    });
    let mut functions = HashMap::new();

    if unlock_print {
        functions.insert(
            "print",
            &predefined_function_print as &'static PredefinedFunction<CompiledProgram>,
        );
    }

    if unlock_sleep {
        functions.insert(
            "sleep",
            &predefined_function_sleep as &'static PredefinedFunction<CompiledProgram>,
        );
    }

    if unlock_brk {
        functions.insert(
            "brk",
            &predefined_function_brk as &'static PredefinedFunction<CompiledProgram>,
        );
    }

    functions
}

pub fn compile_game_state() -> anyhow::Result<()> {
    let parse_result_run_result = with_game_state(|game_state| -> anyhow::Result<_> {
        let parsed = parse_program(&game_state.program_code);
        match parsed {
            Ok(parsed) => {
                let mut compiled = CompiledProgram::new();
                let run_result = compile_with_meta(&parsed, predefined_functions(), &mut compiled);
                Ok(match run_result {
                    Ok(()) => Ok(compiled),
                    Err(e) => Err((e.to_string(), compiled.instruction_counts)),
                })
            }
            Err(richs) => Err(anyhow!(
                richs
                    .into_iter()
                    .map(|rich| format!("{rich}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )),
        }
    });
    let run_result = parse_result_run_result?;
    with_game_state_mut(|game_state| {
        game_state.compiled_program = Some(run_result);
    });
    Ok(())
}
