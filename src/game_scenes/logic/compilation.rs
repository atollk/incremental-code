use crate::game_state::{CompiledProgram, with_game_state};
use anyhow::{anyhow, bail};
use itertools::Itertools;
use language::{
    CompilingMetadata, HashableProgramValue, PredefinedFunction, ProgramValue, compile_with_meta,
    parse_program,
};
use std::collections::HashMap;

fn predefined_function_print(
    meta: &mut WipCompilingProgram,
    args: Vec<ProgramValue>,
) -> anyhow::Result<ProgramValue> {
    let arg = args
        .iter()
        .exactly_one()
        .map_err(|_| anyhow!("print takes exactly one argument"))?;
    let ProgramValue::Hashable(HashableProgramValue::String(s)) = arg else {
        bail!("print requires a string argument")
    };
    meta.program.print_calls.push(s.len() as u64);
    Ok(ProgramValue::None)
}

fn predefined_function_sleep(
    meta: &mut WipCompilingProgram,
    args: Vec<ProgramValue>,
) -> anyhow::Result<ProgramValue> {
    let arg = args
        .iter()
        .exactly_one()
        .map_err(|_| anyhow!("sleep takes exactly one argument"))?;
    let t = if let ProgramValue::Hashable(HashableProgramValue::Int(i)) = arg {
        *i as f64
    } else if let ProgramValue::Float(f) = arg {
        *f
    } else {
        bail!("sleep requires a numeric argument")
    };
    meta.program.sleep_calls.push(t);
    Ok(ProgramValue::None)
}

fn predefined_function_brk(
    meta: &mut WipCompilingProgram,
    args: Vec<ProgramValue>,
) -> anyhow::Result<ProgramValue> {
    if !args.is_empty() {
        bail!("brk takes no arguments")
    }
    meta.program.brk_calls += 1;
    Ok(ProgramValue::None)
}

fn predefined_functions() -> HashMap<&'static str, PredefinedFunction<WipCompilingProgram>> {
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
            predefined_function_print as PredefinedFunction<WipCompilingProgram>,
        );
    }

    if unlock_sleep {
        functions.insert(
            "sleep",
            predefined_function_sleep as PredefinedFunction<WipCompilingProgram>,
        );
    }

    if unlock_brk {
        functions.insert(
            "brk",
            predefined_function_brk as PredefinedFunction<WipCompilingProgram>,
        );
    }

    functions
}

struct WipCompilingProgram {
    program: CompiledProgram,
    is_cancelled: Box<dyn FnMut() -> bool + 'static>,
}

impl WipCompilingProgram {
    fn check_cancel(&mut self) -> anyhow::Result<()> {
        if (self.is_cancelled)() {
            bail!("Cancelling logic program");
        }
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

/// Compiles the given code and returns compile errors in the outer result, or runtime errors in the inner result.
fn compile_code(
    program_code: &str,
    is_cancelled: impl FnMut() -> bool + 'static,
) -> anyhow::Result<Result<CompiledProgram, (String, Vec<u64>)>> {
    let parsed = parse_program(program_code);
    match parsed {
        Ok(parsed) => {
            let mut compiling_program = WipCompilingProgram {
                program: CompiledProgram::new(),
                is_cancelled: Box::new(is_cancelled),
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
                // Debounce is_cancelled check to reduce Mutex locks.
                let mut is_cancelled_debounce = 0;
                let status_for_cancel = status.clone();
                let is_cancelled = move || {
                    is_cancelled_debounce = (is_cancelled_debounce + 1) % 100;
                    if is_cancelled_debounce == 0 {
                        matches!(
                            *status_for_cancel.lock().unwrap(),
                            CompileThreadStatus::Cancelled
                        )
                    } else {
                        false
                    }
                };

                // Compile
                let program_code = with_game_state(|game_state| game_state.program_code.clone());
                let parse_result_run_result = compile_code(&program_code, is_cancelled);

                // Extract result from compilation and set the necessary fields.
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
