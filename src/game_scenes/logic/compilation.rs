use crate::game_state::{CompiledProgram, with_game_state, with_game_state_mut};
use anyhow::{anyhow, bail};
use language::{
    CompilingMetadata, PredefinedFunction, ProgramValue, compile_with_meta, parse_program,
};
use std::collections::HashMap;
use std::sync::Mutex;

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

struct WipCompilingProgram {
    program: CompiledProgram,
    cancelled: bool,
}

impl WipCompilingProgram {
    fn check_cancel(&self) -> anyhow::Result<()> {
        // TODO
        Ok(())
    }
}

impl CompilingMetadata for WipCompilingProgram {
    fn log_zero_instruction(&mut self) -> anyhow::Result<()> {
        self.check_cancel()?;
        self.program.log_zero_instruction()
    }

    fn log_atomic_instruction(&mut self) -> anyhow::Result<()> {
        self.check_cancel()?;
        self.program.log_atomic_instruction()
    }
}

pub fn compile_game_state() -> anyhow::Result<()> {
    let parse_result_run_result = with_game_state(|game_state| -> anyhow::Result<_> {
        let parsed = parse_program(&game_state.program_code);
        match parsed {
            Ok(parsed) => {
                let mut compiling_program = WipCompilingProgram {
                    program: CompiledProgram::new(),
                    cancelled: false,
                };
                let run_result =
                    compile_with_meta(&parsed, predefined_functions(), &mut compiling_program);
                if compiling_program.cancelled {
                    bail!("Compilation was cancelled")
                } else {
                    Ok(match run_result {
                        Ok(()) => Ok(compiling_program.program),
                        Err(e) => {
                            Err((e.to_string(), compiling_program.program.instruction_counts))
                        }
                    })
                }
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

struct CompileThread {}

enum CompileThreadStatus {}

static COMPILE_THREAD: Mutex<CompileThread> = Mutex::new(CompileThread {});

pub fn compile_game_state_thread() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    use std::thread;
    #[cfg(target_arch = "wasm32")]
    use wasm_thread as thread;

    let t = thread::spawn(|| {
        let compile = compile_game_state();
        todo!();
    });
    todo!();
}
