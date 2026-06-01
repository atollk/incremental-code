use crate::game_state::{CompiledProgram, with_game_state};
use anyhow::{anyhow, bail};
use language::{
    CompilingMetadata, PredefinedFunction, ProgramValue, compile_with_meta, parse_program,
};
use std::collections::HashMap;

fn predefined_function_print<T>(
    _meta: &mut T,
    _args: Vec<ProgramValue>,
) -> anyhow::Result<ProgramValue> {
    todo!()
}

fn predefined_function_sleep<T>(
    _meta: &mut T,
    _args: Vec<ProgramValue>,
) -> anyhow::Result<ProgramValue> {
    todo!()
}

fn predefined_function_brk<T>(
    _meta: &mut T,
    _args: Vec<ProgramValue>,
) -> anyhow::Result<ProgramValue> {
    todo!()
}

fn predefined_functions<T>() -> HashMap<&'static str, PredefinedFunction<T>> {
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
            predefined_function_print::<T> as PredefinedFunction<T>,
        );
    }

    if unlock_sleep {
        functions.insert(
            "sleep",
            predefined_function_sleep::<T> as PredefinedFunction<T>,
        );
    }

    if unlock_brk {
        functions.insert("brk", predefined_function_brk::<T> as PredefinedFunction<T>);
    }

    functions
}

struct WipCompilingProgram<F: Fn() -> bool> {
    program: CompiledProgram,
    is_cancelled: F,
}

impl<F: Fn() -> bool> WipCompilingProgram<F> {
    fn check_cancel(&self) -> anyhow::Result<()> {
        if (self.is_cancelled)() {
            bail!("Cancelling logic program");
        }
        Ok(())
    }
}

impl<F: Fn() -> bool> CompilingMetadata for WipCompilingProgram<F> {
    fn log_zero_instruction(&mut self) -> anyhow::Result<()> {
        self.check_cancel()?;
        self.program.log_zero_instruction()
    }

    fn log_atomic_instruction(&mut self) -> anyhow::Result<()> {
        self.check_cancel()?;
        self.program.log_atomic_instruction()
    }
}

/// Compiles the given code and returns compile errors in the outer result, or runtime errors in the inner result.
fn compile_code(
    program_code: &str,
    is_cancelled: impl Fn() -> bool,
) -> anyhow::Result<Result<CompiledProgram, (String, Vec<u64>)>> {
    let parsed = parse_program(program_code);
    match parsed {
        Ok(parsed) => {
            let mut compiling_program = WipCompilingProgram {
                program: CompiledProgram::new(),
                is_cancelled,
            };
            let run_result =
                compile_with_meta(&parsed, predefined_functions(), &mut compiling_program);
            if (compiling_program.is_cancelled)() {
                bail!("Compilation was cancelled")
            } else {
                Ok(match run_result {
                    Ok(()) => Ok(compiling_program.program),
                    Err(e) => Err((e.to_string(), compiling_program.program.instruction_counts)),
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
}

pub mod compile_thread {
    use crate::game_scenes::logic::compilation::compile_code;
    use crate::game_state::{with_game_state, with_game_state_mut};
    use crate::global_variable;
    use std::sync::{Arc, Mutex};
    #[cfg(not(target_arch = "wasm32"))]
    use std::thread;
    #[cfg(target_arch = "wasm32")]
    use wasm_thread as thread;

    #[derive(Debug, Clone)]
    pub enum CompileThreadStatus {
        Idle(Result<(), String>),
        Running,
        Cancelled,
    }

    global_variable!(compile_thread, CompileThread);

    pub struct CompileThread {
        status: Arc<Mutex<CompileThreadStatus>>,
        join_handle: Option<thread::JoinHandle<()>>,
    }

    impl CompileThread {
        /// Compiles the program code in the current game_state.
        /// If compilation fails, an Err is stored in `self.result`.
        /// If compilation succeeds, an Ok is stored in `self.result` and the compilation result is stored in the game_state.
        pub fn compile(&mut self) {
            let status = self.status.clone();
            let f = move || {
                *status.lock().unwrap() = CompileThreadStatus::Running;
                let is_cancelled =
                    || matches!(*status.lock().unwrap(), CompileThreadStatus::Cancelled);
                let parse_result_run_result = with_game_state(|game_state| -> anyhow::Result<_> {
                    compile_code(&game_state.program_code, is_cancelled)
                });
                let result = match parse_result_run_result {
                    Err(parse_err) => Err(parse_err),
                    Ok(run_result) => {
                        with_game_state_mut(|game_state| {
                            game_state.compiled_program = Some(run_result);
                        });
                        Ok(())
                    }
                };
                *status.lock().unwrap() =
                    CompileThreadStatus::Idle(result.map_err(|e| e.to_string()));
            };
            let t = thread::spawn(f);
            self.join_handle = Some(t);
        }

        pub fn status(&self) -> CompileThreadStatus {
            self.status.lock().unwrap().clone()
        }

        pub fn cancel(&mut self) {
            let mut lock = self.status.lock().unwrap();
            if matches!(*lock, CompileThreadStatus::Running) {
                *lock = CompileThreadStatus::Cancelled
            }
        }
    }

    impl Default for CompileThread {
        fn default() -> Self {
            CompileThread {
                status: Arc::new(Mutex::new(CompileThreadStatus::Idle(Ok(())))),
                join_handle: None,
            }
        }
    }
}
