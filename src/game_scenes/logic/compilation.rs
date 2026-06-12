use crate::game_state::{CompiledProgram, with_game_state};
use anyhow::{anyhow, bail};
use itertools::Itertools;
use language::{
    CompilingMetadata, HashableProgramValue, NotPythonProgram, PredefinedFunction, ProgramValue,
    compile_with_meta,
};
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::rc::Rc;

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
    meta.program.print_len = Some(max(s.len() as u64, meta.program.print_len.unwrap_or(0)));
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
    meta.program
        .instruction_counts
        .last_mut()
        .expect("instruction vectors to never be empty")
        .push(0);
    Ok(ProgramValue::None)
}

fn predefined_function_brk(
    meta: &mut WipCompilingProgram,
    args: Vec<ProgramValue>,
) -> anyhow::Result<ProgramValue> {
    if !args.is_empty() {
        bail!("brk takes no arguments")
    }
    meta.program.instruction_counts.push(vec![0]);
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

#[derive(Clone)]
struct WipCompilingProgram {
    program: CompiledProgram,
    is_cancelled: Rc<RefCell<dyn FnMut() -> bool + 'static>>,
    left_to_instruction_limit: u64,
}

impl WipCompilingProgram {
    fn instruction_limit() -> u64 {
        with_game_state(|game_state| game_state.upgrades.max_instructions.value())
    }

    fn new(is_cancelled: impl FnMut() -> bool + 'static) -> Self {
        Self {
            program: CompiledProgram::new(),
            is_cancelled: Rc::new(RefCell::new(is_cancelled)),
            left_to_instruction_limit: Self::instruction_limit(),
        }
    }

    fn check_cancel(&mut self) -> anyhow::Result<()> {
        if (self.is_cancelled.borrow_mut())() {
            bail!("Cancelling logic program");
        }
        Ok(())
    }
}

struct WipCompilingProgramDiff {
    program: <CompiledProgram as CompilingMetadata>::Diff,
    instruction_count: i64,
}

impl CompilingMetadata for WipCompilingProgram {
    type Diff = WipCompilingProgramDiff;

    fn log_zero_instruction(&mut self) -> anyhow::Result<()> {
        self.check_cancel()?;
        self.program.log_zero_instruction()
    }

    fn log_atomic_instruction(&mut self) -> anyhow::Result<()> {
        self.check_cancel()?;
        self.left_to_instruction_limit -= 1;
        if self.left_to_instruction_limit == 0 {
            bail!("Reached instruction limit. Stopping execution to prevent overheating.")
        }
        self.program.log_atomic_instruction()
    }

    fn diff(&self, other: &Self) -> anyhow::Result<Self::Diff> {
        Ok(WipCompilingProgramDiff {
            program: self.program.diff(&other.program),
            instruction_count: (other.left_to_instruction_limit - self.left_to_instruction_limit)
                as i64,
        })
    }

    fn add_assign(&mut self, diff: &Self::Diff) -> anyhow::Result<()> {
        self.program.add_assign(&diff.program)?;
        self.left_to_instruction_limit = self
            .left_to_instruction_limit
            .saturating_sub_signed(diff.instruction_count);
        if self.left_to_instruction_limit == 0 {
            bail!("Reached instruction limit. Stopping execution to prevent overheating.")
        } else {
            Ok(())
        }
    }
}

fn parse_code(program_code: &str) -> anyhow::Result<NotPythonProgram> {
    match language::parse_program(program_code) {
        Ok(parsed) => Ok(parsed),
        Err(richs) => Err(anyhow!(
            richs
                .into_iter()
                .map(|rich| format!("{rich}"))
                .collect::<Vec<_>>()
                .join("\n")
        )),
    }
}

/// Compiles the given code and returns compile errors in the outer result, or runtime errors in the inner result.
fn compile_code(
    parsed_program: &NotPythonProgram,
    is_cancelled: impl FnMut() -> bool + 'static,
) -> anyhow::Result<Result<CompiledProgram, (String, Vec<Vec<u64>>)>> {
    let mut compiling_program = WipCompilingProgram::new(is_cancelled);
    let run_result = compile_with_meta(
        &parsed_program,
        predefined_functions(),
        &mut compiling_program,
    );
    if (compiling_program.is_cancelled.borrow_mut())() {
        bail!("Compilation was cancelled")
    } else {
        Ok(match run_result {
            Ok(()) => Ok(compiling_program.program),
            Err(e) => Err((e.to_string(), compiling_program.program.instruction_counts)),
        })
    }
}

pub mod compile_thread {
    use crate::game_scenes::logic::compilation::{compile_code, parse_code};
    use crate::game_scenes::logic::verification::verify_unlocks;
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
                let get_compile_result = || -> anyhow::Result<_> {
                    let program_code =
                        with_game_state(|game_state| game_state.program_code.clone());
                    let parsed_code = parse_code(&program_code)?;
                    verify_unlocks(&program_code, &parsed_code)?;
                    compile_code(&parsed_code, is_cancelled)
                };
                let thread_result = match get_compile_result() {
                    Ok(compile_result) => {
                        with_game_state_mut(|game_state| {
                            game_state.compiled_program = Some(compile_result);
                        });
                        Ok(())
                    }
                    Err(e) => Err(e.to_string()),
                };
                *status.lock().unwrap() = CompileThreadStatus::Idle(thread_result);
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
