use crate::language::parser::{NotPythonExpr, NotPythonProgram, NotPythonStmt};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Add;

#[derive(Serialize, Deserialize)]
pub struct CompiledProgram {
    pub execution_time: f64,
    pub instruction_count: u128,
}

impl Add for CompiledProgram {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        CompiledProgram {
            execution_time: self.execution_time + rhs.execution_time,
            instruction_count: self.instruction_count + rhs.instruction_count,
        }
    }
}

pub fn compile(program: &NotPythonProgram) -> CompiledProgram {
    let mut state = ProgramExecutionState {
        execution_speed: 1.0,
    };
    compile_stmt(&program.statement, &mut state)
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
enum HashableProgramValue {
    Int(i64),
    String(String),
    Bool(bool),
}

#[derive(PartialEq, Clone, Debug)]
enum ProgramValue {
    Hashable(HashableProgramValue),
    Float(f64),
    None,
    List(Vec<ProgramValue>),
    Dict(HashMap<HashableProgramValue, ProgramValue>),
}

#[derive(PartialEq, Clone, Debug)]
enum ProgramExecutionControlFlow {
    Normal,
    Continue,
    Break,
    Return(ProgramValue),
}

struct ProgramExecutionCallState<'a> {
    variables: HashMap<&'a str, ProgramValue>,
    functions: HashMap<&'a str, &'a NotPythonStmt>,
    loop_nesting: usize,
}

impl<'a> Default for ProgramExecutionCallState<'a> {
    fn default() -> Self {
        ProgramExecutionCallState {
            variables: HashMap::new(),
            functions: HashMap::new(),
            loop_nesting: 0,
        }
    }
}

struct ProgramExecutionState<'a> {
    // Execution state
    control_flow: ProgramExecutionControlFlow,
    call_stack: Vec<ProgramExecutionCallState<'a>>,
    // Counting compilation data
    instruction_speedup_count: u128,
    loop_nesting_count: u32,
    function_nesting_count: u32,
}

impl<'a> ProgramExecutionState<'a> {
    fn statement_execution_time(&self) -> f64 {
        1.0 / self.instruction_speedup_count as f64
    }

    fn get_variable(&self, name: &str) -> Option<&ProgramValue> {
        for call_state in self.call_stack.iter().rev() {
            if let Some(value) = call_state.variables.get(name) {
                return Some(value);
            }
        }
        None
    }

    fn decl_variable(&mut self, name: &'a str, value: ProgramValue) {
        self.call_stack
            .last_mut()
            .unwrap()
            .variables
            .insert(name, value);
    }

    fn assign_variable(&mut self, name: &str, value: ProgramValue) -> anyhow::Result<()> {
        for call_state in self.call_stack.iter_mut().rev() {
            if let Some(v) = call_state.variables.get_mut(name) {
                *v = value;
                return Ok(());
            }
        }
        Err(anyhow!("Variable {} not found", name))
    }

    fn decl_function(&mut self, name: &str, params: &[String], body: &NotPythonStmt) {
        todo!()
    }
}

fn compile_stmt<'a>(
    stmt: &'a NotPythonStmt,
    state: &mut ProgramExecutionState<'a>,
) -> anyhow::Result<CompiledProgram> {
    let zero_compile = CompiledProgram {
        execution_time: 0.0,
        instruction_count: 0,
    };
    let atom_compile = CompiledProgram {
        execution_time: state.statement_execution_time(),
        instruction_count: 1,
    };
    state.instruction_speedup_count += 1;
    if let NotPythonStmt::Call(_, _) = stmt {
    } else if let ProgramExecutionControlFlow::Return(_) = state.control_flow {
        return Ok(zero_compile);
    }
    match stmt {
        NotPythonStmt::Block(stmts) => {
            stmts
                .iter()
                .map(|s| compile_stmt(s, state))
                .fold(Ok(zero_compile), |acc, b| {
                    let acc = acc?;
                    let b = b?;
                    Ok(CompiledProgram {
                        execution_time: acc.execution_time + b.execution_time,
                        instruction_count: acc.instruction_count + b.instruction_count,
                    })
                })
        }
        NotPythonStmt::Pass => Ok(atom_compile),
        NotPythonStmt::Break => {
            state.control_flow = ProgramExecutionControlFlow::Break;
            Ok(atom_compile)
        }
        NotPythonStmt::Continue => {
            state.control_flow = ProgramExecutionControlFlow::Continue;
            Ok(atom_compile)
        }
        NotPythonStmt::Decl(name, expr) => {
            let expr = eval_expr(expr, state);
            state.decl_variable(name, expr);
            Ok(atom_compile)
        }
        NotPythonStmt::Assign(name, expr) => {
            if state.get_variable(name).is_none() {
                todo!("Error")
            }
            let expr = eval_expr(expr, state);
            state.assign_variable(name, expr)?;
            Ok(atom_compile)
        }
        NotPythonStmt::If {
            condition,
            then,
            else_,
        } => match eval_expr(&**condition, state) {
            ProgramValue::Hashable(HashableProgramValue::Bool(b)) => {
                let stmt = if b {
                    Some(&**then)
                } else {
                    else_.as_ref().map(|s| &**s)
                };
                let rec = if let Some(stmt) = stmt {
                    compile_stmt(&stmt, state)? + atom_compile
                } else {
                    atom_compile
                };
                Ok(rec)
            }
            _ => Err(anyhow!("Condition expression is not a boolean")),
        },
        NotPythonStmt::Loop(body) => {
            todo!()
        }
        NotPythonStmt::Function { name, params, body } => {
            state.decl_function(name, params, body);
            Ok(atom_compile)
        }
        NotPythonStmt::Call(function, args) => {
            todo!()
        }
    }
}

fn eval_expr(expr: &NotPythonExpr, state: &mut ProgramExecutionState) -> ProgramValue {
    todo!()
}
