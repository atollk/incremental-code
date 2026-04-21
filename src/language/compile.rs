use crate::language::parser::{NotPythonExpr, NotPythonProgram, NotPythonStmt};
use anyhow::anyhow;
use chumsky::prelude::todo;
use egui::ahash::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CompiledProgram {
    pub execution_time: f64,
    pub instruction_count: u128,
}

pub fn compile(program: &NotPythonProgram) -> CompiledProgram {
    let mut state = ProgramExecutionState {
        execution_speed: 1.0,
    };
    compile_stmt(&program.statement, &mut state)
}

enum ProgramValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    None,
    List(Vec<ProgramValue>),
    Dict(HashMap<ProgramValue, ProgramValue>),
}

struct ProgramExecutionState {
    // Execution state
    variables_stack: Vec<HashMap<String, ProgramValue>>,
    functions: HashMap<String, NotPythonStmt>,
    // Counting compilation data
    instruction_speedup_count: u128,
    loop_nesting_count: u32,
    function_nesting_count: u32,
}

impl ProgramExecutionState {
    fn statement_execution_time(&self) -> f64 {
        1.0 / self.instruction_speedup_count as f64
    }

    fn get_variable(&self, name: &str) -> Option<ProgramValue> {
        for map in self.variables_stack.iter().rev() {
            if let Some(value) = map.get(name) {
                return Some(*value);
            }
        }
        None
    }

    fn decl_variable(&mut self, name: String, value: ProgramValue) {
        self.variables_stack.last_mut().unwrap().insert(name, value);
    }

    fn assign_variable(&mut self, name: &str, value: ProgramValue) -> anyhow::Result<()> {
        for map in self.variables_stack.iter().rev() {
            if let Some(v) = map.get_mut(name) {
                *v = value;
                return Ok(());
            }
        }
        Err(anyhow!("Variable {} not found", name))
    }
}

fn compile_stmt(
    stmt: &NotPythonStmt,
    state: &mut ProgramExecutionState,
) -> anyhow::Result<CompiledProgram> {
    let atom_compile = CompiledProgram {
        execution_time: state.statement_execution_time(),
        instruction_count: 1,
    };
    state.instruction_speedup_count += 1;
    match stmt {
        NotPythonStmt::Block(stmts) => stmts.iter().map(|s| compile_stmt(s, state)).fold(
            Ok(CompiledProgram {
                execution_time: 0.0,
                instruction_count: 0,
            }),
            |acc, b| {
                let acc = acc?;
                let b = b?;
                Ok(CompiledProgram {
                    execution_time: acc.execution_time + b.execution_time,
                    instruction_count: acc.instruction_count + b.instruction_count,
                })
            },
        ),
        NotPythonStmt::Pass => Ok(atom_compile),
        NotPythonStmt::Break => {}
        NotPythonStmt::Continue => {}
        NotPythonStmt::Decl(name, expr) => {
            state.decl_variable(*name, eval_expr(expr, state));
            Ok(atom_compile)
        }
        NotPythonStmt::Assign(name, expr) => {
            if state.get_variable(name).is_none() {
                todo!("Error")
            }
            state.assign_variable(*name, eval_expr(expr, state));
            Ok(atom_compile)
        }
        NotPythonStmt::If { .. } => {}
        NotPythonStmt::Loop(_) => {}
        NotPythonStmt::Function { .. } => {}
        NotPythonStmt::Call(function, args) => {}
    }
}

fn eval_expr(expr: &NotPythonExpr, state: &mut ProgramExecutionState) -> ProgramValue {
    todo!()
}
