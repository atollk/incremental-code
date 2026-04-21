use crate::language::parser::{NotPythonExpr, NotPythonExprOp, NotPythonProgram, NotPythonStmt};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::ops::{Add, Deref};

#[derive(Serialize, Deserialize, Debug)]
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
        control_flow: ProgramExecutionControlFlow::Normal,
        call_stack: vec![ProgramExecutionCallState::default()],
        instruction_speedup_count: 1,
    };
    compile_stmt(&program.statement, &mut state).unwrap_or(CompiledProgram {
        execution_time: 0.0,
        instruction_count: 0,
    })
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

#[derive(Default)]
struct ProgramExecutionCallState<'a> {
    variables: HashMap<&'a str, ProgramValue>,
    functions: HashMap<&'a str, &'a NotPythonStmt>,
    loop_nesting: usize,
}

struct ProgramExecutionState<'a> {
    // Execution state
    control_flow: ProgramExecutionControlFlow,
    call_stack: Vec<ProgramExecutionCallState<'a>>,
    // Counting compilation data
    instruction_speedup_count: u128,
}

impl<'a> ProgramExecutionState<'a> {
    fn statement_execution_time(&self) -> f64 {
        1.0 / self.instruction_speedup_count as f64
    }

    fn get_variable(&self, name: &str) -> anyhow::Result<&ProgramValue> {
        for call_state in self.call_stack.iter().rev() {
            if let Some(value) = call_state.variables.get(name) {
                return Ok(value);
            }
        }
        Err(anyhow::format_err!("Variable {name} not found"))
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

    fn decl_function(&mut self, name: &'a str, stmt: &'a NotPythonStmt) {
        self.call_stack
            .last_mut()
            .unwrap()
            .functions
            .insert(name, stmt);
    }

    fn get_function(&self, name: &str) -> anyhow::Result<&'a NotPythonStmt> {
        for call_state in self.call_stack.iter().rev() {
            if let Some(&stmt) = call_state.functions.get(name) {
                return Ok(stmt);
            }
        }
        Err(anyhow!("Function {} not found", name))
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
                .try_fold(zero_compile, |acc, b| {
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
        NotPythonStmt::Return(expr) => {
            state.control_flow = ProgramExecutionControlFlow::Return(
                expr.as_ref()
                    .map(|e| eval_expr(e, state))
                    .unwrap_or(Ok(ProgramValue::None))?,
            );
            Ok(atom_compile)
        }
        NotPythonStmt::Decl(name, expr) => {
            let expr = eval_expr(expr, state)?;
            state.decl_variable(name, expr);
            Ok(atom_compile)
        }
        NotPythonStmt::Assign(name, expr) => {
            state.get_variable(name)?;
            let expr = eval_expr(expr, state)?;
            state.assign_variable(name, expr)?;
            Ok(atom_compile)
        }
        NotPythonStmt::If {
            condition,
            then,
            else_,
        } => match eval_expr(condition, state)? {
            ProgramValue::Hashable(HashableProgramValue::Bool(b)) => {
                let stmt = if b {
                    Some(&**then)
                } else {
                    else_.as_ref().map(|s| &**s)
                };
                let rec = if let Some(stmt) = stmt {
                    compile_stmt(stmt, state)? + atom_compile
                } else {
                    atom_compile
                };
                Ok(rec)
            }
            _ => Err(anyhow!("Condition expression is not a boolean")),
        },
        NotPythonStmt::Loop(body) => {
            state.call_stack.last_mut().unwrap().loop_nesting += 1;
            let mut total = atom_compile;
            loop {
                total = total + compile_stmt(body, state)?;
                match &state.control_flow {
                    ProgramExecutionControlFlow::Break => {
                        state.control_flow = ProgramExecutionControlFlow::Normal;
                        break;
                    }
                    ProgramExecutionControlFlow::Continue => {
                        state.control_flow = ProgramExecutionControlFlow::Normal;
                    }
                    ProgramExecutionControlFlow::Return(_) => break,
                    ProgramExecutionControlFlow::Normal => {}
                }
            }
            state.call_stack.last_mut().unwrap().loop_nesting -= 1;
            Ok(total)
        }
        NotPythonStmt::Function {
            name,
            params: _,
            body: _,
        } => {
            state.decl_function(name, stmt);
            Ok(atom_compile)
        }
        NotPythonStmt::Call(name, args) => {
            let func = state.get_function(name)?;

            // Map argument values to parameter names
            let (params, body) = match func {
                NotPythonStmt::Function { params, body, .. } => (params.as_slice(), &**body),
                _ => return Err(anyhow!("'{}' is not a function", name)),
            };
            let arg_values: Vec<ProgramValue> = args
                .iter()
                .map(|a| eval_expr(a, state))
                .collect::<anyhow::Result<_>>()?;
            if params.len() != arg_values.len() {
                return Err(anyhow!(
                    "Function '{}' expects {} arguments but got {}",
                    name,
                    params.len(),
                    arg_values.len()
                ));
            }

            // Create new call stack frame
            let mut frame = ProgramExecutionCallState::default();
            for (param, val) in params.iter().zip(arg_values) {
                frame.variables.insert(param.as_str(), val);
            }
            state.call_stack.push(frame);

            // Run function & return
            let result = compile_stmt(body, state);
            state.call_stack.pop();
            if let ProgramExecutionControlFlow::Return(_) = state.control_flow {
                state.control_flow = ProgramExecutionControlFlow::Normal;
            }
            Ok(result? + atom_compile)
        }
    }
}

fn eval_unary_op(
    val: ProgramValue,
    op: &NotPythonExprOp,
    _state: &mut ProgramExecutionState,
) -> anyhow::Result<ProgramValue> {
    match op {
        NotPythonExprOp::Neg(_) => match val {
            ProgramValue::Hashable(HashableProgramValue::Int(i)) => {
                Ok(ProgramValue::Hashable(HashableProgramValue::Int(-i)))
            }
            ProgramValue::Float(f) => Ok(ProgramValue::Float(-f)),
            _ => Err(anyhow!("Cannot negate non-numeric value")),
        },
        NotPythonExprOp::Not(_) => match val {
            ProgramValue::Hashable(HashableProgramValue::Bool(b)) => {
                Ok(ProgramValue::Hashable(HashableProgramValue::Bool(!b)))
            }
            _ => Err(anyhow!("Cannot apply 'not' to non-boolean value")),
        },
        _ => unreachable!("eval_unary_op called with binary op"),
    }
}

fn eval_binary_op(
    lhs: ProgramValue,
    rhs: ProgramValue,
    op: &NotPythonExprOp,
    _state: &mut ProgramExecutionState,
) -> anyhow::Result<ProgramValue> {
    use HashableProgramValue::*;
    use ProgramValue::*;

    match op {
        NotPythonExprOp::Add(_, _) => match (lhs, rhs) {
            (Hashable(Int(a)), Hashable(Int(b))) => Ok(Hashable(Int(a + b))),
            (Hashable(Int(a)), Float(b)) => Ok(Float(a as f64 + b)),
            (Float(a), Hashable(Int(b))) => Ok(Float(a + b as f64)),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            (Hashable(String(a)), Hashable(String(b))) => Ok(Hashable(String(a + &b))),
            _ => Err(anyhow!("'+' operands must be numeric or both strings")),
        },
        NotPythonExprOp::Sub(_, _) => match (lhs, rhs) {
            (Hashable(Int(a)), Hashable(Int(b))) => Ok(Hashable(Int(a - b))),
            (Hashable(Int(a)), Float(b)) => Ok(Float(a as f64 - b)),
            (Float(a), Hashable(Int(b))) => Ok(Float(a - b as f64)),
            (Float(a), Float(b)) => Ok(Float(a - b)),
            _ => Err(anyhow!("'-' operands must be numeric")),
        },
        NotPythonExprOp::Mul(_, _) => match (lhs, rhs) {
            (Hashable(Int(a)), Hashable(Int(b))) => Ok(Hashable(Int(a * b))),
            (Hashable(Int(a)), Float(b)) => Ok(Float(a as f64 * b)),
            (Float(a), Hashable(Int(b))) => Ok(Float(a * b as f64)),
            (Float(a), Float(b)) => Ok(Float(a * b)),
            _ => Err(anyhow!("'*' operands must be numeric")),
        },
        NotPythonExprOp::Div(_, _) => match (lhs, rhs) {
            (Hashable(Int(a)), Hashable(Int(b))) => {
                if b == 0 {
                    return Err(anyhow!("Division by zero"));
                }
                Ok(Hashable(Int(a / b)))
            }
            (Hashable(Int(a)), Float(b)) => Ok(Float(a as f64 / b)),
            (Float(a), Hashable(Int(b))) => Ok(Float(a / b as f64)),
            (Float(a), Float(b)) => Ok(Float(a / b)),
            _ => Err(anyhow!("'/' operands must be numeric")),
        },
        NotPythonExprOp::Mod(_, _) => match (lhs, rhs) {
            (Hashable(Int(a)), Hashable(Int(b))) => {
                if b == 0 {
                    return Err(anyhow!("Modulo by zero"));
                }
                Ok(Hashable(Int(a % b)))
            }
            _ => Err(anyhow!("'%' operands must be integers")),
        },
        NotPythonExprOp::And(_, _) => match (lhs, rhs) {
            (Hashable(Bool(a)), Hashable(Bool(b))) => Ok(Hashable(Bool(a && b))),
            _ => Err(anyhow!("'and' operands must be booleans")),
        },
        NotPythonExprOp::Or(_, _) => match (lhs, rhs) {
            (Hashable(Bool(a)), Hashable(Bool(b))) => Ok(Hashable(Bool(a || b))),
            _ => Err(anyhow!("'or' operands must be booleans")),
        },
        NotPythonExprOp::Equal(_, _) => match (lhs, rhs) {
            (Hashable(a), Hashable(b)) => Ok(Hashable(Bool(a == b))),
            (Float(a), Float(b)) => Ok(Hashable(Bool(a == b))),
            (None, None) => Ok(Hashable(Bool(true))),
            _ => Ok(Hashable(Bool(false))),
        },
        NotPythonExprOp::NotEqual(_, _) => match (lhs, rhs) {
            (Hashable(a), Hashable(b)) => Ok(Hashable(Bool(a != b))),
            (Float(a), Float(b)) => Ok(Hashable(Bool(a != b))),
            (None, None) => Ok(Hashable(Bool(false))),
            _ => Ok(Hashable(Bool(true))),
        },
        NotPythonExprOp::Greater(_, _) => match (lhs, rhs) {
            (Hashable(Int(a)), Hashable(Int(b))) => Ok(Hashable(Bool(a > b))),
            (Hashable(Int(a)), Float(b)) => Ok(Hashable(Bool((a as f64) > b))),
            (Float(a), Hashable(Int(b))) => Ok(Hashable(Bool(a > b as f64))),
            (Float(a), Float(b)) => Ok(Hashable(Bool(a > b))),
            _ => Err(anyhow!("'>' operands must be numeric")),
        },
        NotPythonExprOp::Less(_, _) => match (lhs, rhs) {
            (Hashable(Int(a)), Hashable(Int(b))) => Ok(Hashable(Bool(a < b))),
            (Hashable(Int(a)), Float(b)) => Ok(Hashable(Bool((a as f64) < b))),
            (Float(a), Hashable(Int(b))) => Ok(Hashable(Bool(a < b as f64))),
            (Float(a), Float(b)) => Ok(Hashable(Bool(a < b))),
            _ => Err(anyhow!("'<' operands must be numeric")),
        },
        NotPythonExprOp::GreaterEqual(_, _) => match (lhs, rhs) {
            (Hashable(Int(a)), Hashable(Int(b))) => Ok(Hashable(Bool(a >= b))),
            (Hashable(Int(a)), Float(b)) => Ok(Hashable(Bool((a as f64) >= b))),
            (Float(a), Hashable(Int(b))) => Ok(Hashable(Bool(a >= b as f64))),
            (Float(a), Float(b)) => Ok(Hashable(Bool(a >= b))),
            _ => Err(anyhow!("'>=' operands must be numeric")),
        },
        NotPythonExprOp::LessEqual(_, _) => match (lhs, rhs) {
            (Hashable(Int(a)), Hashable(Int(b))) => Ok(Hashable(Bool(a <= b))),
            (Hashable(Int(a)), Float(b)) => Ok(Hashable(Bool((a as f64) <= b))),
            (Float(a), Hashable(Int(b))) => Ok(Hashable(Bool(a <= b as f64))),
            (Float(a), Float(b)) => Ok(Hashable(Bool(a <= b))),
            _ => Err(anyhow!("'<=' operands must be numeric")),
        },
        NotPythonExprOp::In(_, _) => match rhs {
            List(l) => Ok(Hashable(Bool(l.contains(&lhs)))),
            Dict(d) => match lhs {
                Hashable(k) => Ok(Hashable(Bool(d.contains_key(&k)))),
                _ => Err(anyhow!("'in' for dicts requires a hashable key")),
            },
            _ => Err(anyhow!(
                "'in' requires a list or dict on the right-hand side"
            )),
        },
        _ => unreachable!("eval_binary_op called with unary op"),
    }
}

fn eval_expr<'a>(
    expr: &'a NotPythonExpr,
    state: &mut ProgramExecutionState<'a>,
) -> anyhow::Result<ProgramValue> {
    match expr {
        NotPythonExpr::Int(i) => Ok(ProgramValue::Hashable(HashableProgramValue::Int(*i))),
        NotPythonExpr::Float(f) => Ok(ProgramValue::Float(*f)),
        NotPythonExpr::String(s) => Ok(ProgramValue::Hashable(HashableProgramValue::String(
            s.clone(),
        ))),
        NotPythonExpr::Boolean(b) => Ok(ProgramValue::Hashable(HashableProgramValue::Bool(*b))),
        NotPythonExpr::None => Ok(ProgramValue::None),
        NotPythonExpr::Identifier(name) => state.get_variable(name).cloned(),
        NotPythonExpr::List(l) => Ok(ProgramValue::List(
            l.iter()
                .map(|ex| eval_expr(ex, state))
                .collect::<anyhow::Result<_>>()?,
        )),
        NotPythonExpr::Dict(d) => {
            let mut map = HashMap::new();
            for (k, v) in d {
                let key = eval_expr(k, state)?;
                let val = eval_expr(v, state)?;
                match key {
                    ProgramValue::Hashable(h) => {
                        map.insert(h, val);
                    }
                    _ => return Err(anyhow!("Dict keys must be hashable (int, string, or bool)")),
                }
            }
            Ok(ProgramValue::Dict(map))
        }
        NotPythonExpr::Op(o) => match o {
            NotPythonExprOp::Add(lhs, rhs)
            | NotPythonExprOp::Sub(lhs, rhs)
            | NotPythonExprOp::Mul(lhs, rhs)
            | NotPythonExprOp::Div(lhs, rhs)
            | NotPythonExprOp::Mod(lhs, rhs)
            | NotPythonExprOp::And(lhs, rhs)
            | NotPythonExprOp::Or(lhs, rhs)
            | NotPythonExprOp::Equal(lhs, rhs)
            | NotPythonExprOp::NotEqual(lhs, rhs)
            | NotPythonExprOp::Greater(lhs, rhs)
            | NotPythonExprOp::Less(lhs, rhs)
            | NotPythonExprOp::GreaterEqual(lhs, rhs)
            | NotPythonExprOp::LessEqual(lhs, rhs)
            | NotPythonExprOp::In(lhs, rhs) => {
                eval_binary_op(eval_expr(lhs, state)?, eval_expr(rhs, state)?, o, state)
            }
            NotPythonExprOp::Neg(val) | NotPythonExprOp::Not(val) => {
                eval_unary_op(eval_expr(val, state)?, o, state)
            }
        },
        NotPythonExpr::Call(name_expr, args) => {
            let name = match name_expr.deref() {
                NotPythonExpr::Identifier(n) => n.as_str(),
                _ => return Err(anyhow!("Can only call named functions")),
            };
            let func = state.get_function(name)?;

            // Map argument values to parameters
            let (params, body) = match func {
                NotPythonStmt::Function { params, body, .. } => (params.as_slice(), &**body),
                _ => return Err(anyhow!("'{}' is not a function", name)),
            };
            let arg_values: Vec<ProgramValue> = args
                .iter()
                .map(|a| eval_expr(a, state))
                .collect::<anyhow::Result<_>>()?;
            if params.len() != arg_values.len() {
                return Err(anyhow!(
                    "Function '{}' expects {} arguments but got {}",
                    name,
                    params.len(),
                    arg_values.len()
                ));
            }

            // Create new call stack frame
            let mut frame = ProgramExecutionCallState::default();
            for (param, val) in params.iter().zip(arg_values) {
                frame.variables.insert(param.as_str(), val);
            }
            state.call_stack.push(frame);

            // Run function & return
            compile_stmt(body, state)?;
            state.call_stack.pop();
            let return_value = match std::mem::replace(
                &mut state.control_flow,
                ProgramExecutionControlFlow::Normal,
            ) {
                ProgramExecutionControlFlow::Return(v) => v,
                _ => ProgramValue::None,
            };
            Ok(return_value)
        }
        NotPythonExpr::Index(lhs, rhs) => {
            let lhs = eval_expr(lhs, state)?;
            let rhs = eval_expr(rhs, state)?;
            match lhs {
                ProgramValue::List(l) => match rhs {
                    ProgramValue::Hashable(HashableProgramValue::Int(i)) => l
                        .get(i as usize)
                        .ok_or(anyhow!("Index {i} out of range"))
                        .cloned(),
                    _ => Err(anyhow!(
                        "Index operator on lists can only be used with integers."
                    )),
                },
                ProgramValue::Dict(d) => match rhs {
                    ProgramValue::Hashable(rhs) => {
                        d.get(&rhs).ok_or(anyhow!("Key not found in dict")).cloned()
                    }
                    _ => Err(anyhow!(
                        "Index operator on dicts can only be used with integers, bools, or strings."
                    )),
                },
                _ => Err(anyhow!(
                    "Index operator can only be used on lists or dicts."
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::parser::parse_program;

    fn compiled(src: &str) -> CompiledProgram {
        compile(&parse_program(src))
    }

    fn compiled_err(src: &str) -> anyhow::Error {
        // We need direct access to the Result for error tests.
        let program = parse_program(src);
        let mut state = ProgramExecutionState {
            control_flow: ProgramExecutionControlFlow::Normal,
            call_stack: vec![ProgramExecutionCallState::default()],
            instruction_speedup_count: 1,
        };
        compile_stmt(&program.statement, &mut state).unwrap_err()
    }

    // -------------------------------------------------------------------------
    // instruction_count
    // -------------------------------------------------------------------------

    #[test]
    fn single_pass_counts_one() {
        assert_eq!(compiled("pass;").instruction_count, 1);
    }

    #[test]
    fn two_stmts_count_two() {
        assert_eq!(compiled("pass;\npass;\n").instruction_count, 2);
    }

    #[test]
    fn execution_time_positive() {
        assert!(compiled("pass;").execution_time > 0.0);
    }

    #[test]
    fn later_stmts_have_lower_time() {
        // execution_time is 1/speedup; speedup grows each call, so each
        // successive statement contributes less time than the previous one.
        let p1 = compiled("pass;");
        let p2 = compiled("pass;\npass;\n");
        // Total time for two passes must be less than 2× the time of one pass,
        // because the second pass runs at a higher speedup factor.
        assert!(p2.execution_time < 2.0 * p1.execution_time);
    }

    // -------------------------------------------------------------------------
    // Variables
    // -------------------------------------------------------------------------

    #[test]
    fn decl_and_assign() {
        let c = compiled("x := 42;\nx = 1;\n");
        assert_eq!(c.instruction_count, 2);
    }

    #[test]
    fn undefined_variable_is_error() {
        let err = compiled_err("x = 1;\n");
        assert!(err.to_string().contains("not found"));
    }

    // -------------------------------------------------------------------------
    // Arithmetic & expressions
    // -------------------------------------------------------------------------

    #[test]
    fn arithmetic_in_decl() {
        // Just check it compiles without panic; the value is 7.
        let c = compiled("x := 1 + 2 * 3;\n");
        assert_eq!(c.instruction_count, 1);
    }

    #[test]
    fn negation_in_decl() {
        let c = compiled("x := -5;\n");
        assert_eq!(c.instruction_count, 1);
    }

    #[test]
    fn string_concat() {
        let c = compiled("x := \"hello\" + \" world\";\n");
        assert_eq!(c.instruction_count, 1);
    }

    // -------------------------------------------------------------------------
    // Conditionals
    // -------------------------------------------------------------------------

    #[test]
    fn if_true_branch_executes() {
        // then branch: pass (1 stmt) → 1 + 1 = 2 instructions (if + pass)
        let c = compiled("if True:\npass;\nend\n");
        assert_eq!(c.instruction_count, 2);
    }

    #[test]
    fn if_false_branch_skipped() {
        // condition is False, no else → only the if stmt itself counts
        let c = compiled("if False:\npass;\nend\n");
        assert_eq!(c.instruction_count, 1);
    }

    #[test]
    fn if_else_false_takes_else() {
        // else branch has break (1 stmt) → 1 (if) + 1 (break) = 2
        let c = compiled("if False:\npass;\nelse:\nbreak;\nend\n");
        assert_eq!(c.instruction_count, 2);
    }

    #[test]
    fn if_non_bool_condition_is_error() {
        let err = compiled_err("if 42:\npass;\nend\n");
        assert!(err.to_string().contains("not a boolean"));
    }

    // -------------------------------------------------------------------------
    // Loops
    // -------------------------------------------------------------------------

    #[test]
    fn loop_immediate_break() {
        // Loop runs once (break on first iteration): atom(loop) + break = 2
        let c = compiled("loop:\nbreak;\nend\n");
        assert_eq!(c.instruction_count, 2);
    }

    #[test]
    fn loop_body_executes_multiple_times() {
        // x counts down from 3; loop runs 3 times then breaks.
        let src = "x := 3;\nloop:\nif x == 1:\nbreak;\nend\nx = x + -1;\nend\n";
        let c = compiled(src);
        // More instructions than if there were no loop.
        assert!(c.instruction_count > 3);
    }

    #[test]
    fn loop_adds_more_instructions_than_sequential() {
        // A loop that runs twice should produce more instructions than two
        // inline pass statements.
        let looped = compiled("loop:\npass;\npass;\nbreak;\nend\n");
        let sequential = compiled("pass;\npass;\n");
        assert!(looped.instruction_count > sequential.instruction_count);
    }

    // -------------------------------------------------------------------------
    // Functions
    // -------------------------------------------------------------------------

    #[test]
    fn function_def_counts_one() {
        let c = compiled("def foo():\npass;\nend\n");
        assert_eq!(c.instruction_count, 1);
    }

    #[test]
    fn function_call_executes_body() {
        // def (1) + call stmt (1 overhead + body pass 1) = 3
        let c = compiled("def foo():\npass;\nend\nfoo();\n");
        assert_eq!(c.instruction_count, 3);
    }

    #[test]
    fn function_with_params() {
        let c = compiled("def add(x, y):\npass;\nend\nadd(1, 2);\n");
        assert_eq!(c.instruction_count, 3);
    }

    #[test]
    fn undefined_function_is_error() {
        let err = compiled_err("foo();\n");
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn wrong_arg_count_is_error() {
        let err = compiled_err("def foo(x):\npass;\nend\nfoo();\n");
        assert!(err.to_string().contains("expects 1 arguments but got 0"));
    }

    // -------------------------------------------------------------------------
    // Lists & dicts
    // -------------------------------------------------------------------------

    #[test]
    fn list_literal_in_decl() {
        let c = compiled("x := [1, 2, 3];\n");
        assert_eq!(c.instruction_count, 1);
    }

    #[test]
    fn list_index() {
        let c = compiled("x := [10, 20, 30];\ny := x[1];\n");
        assert_eq!(c.instruction_count, 2);
    }

    #[test]
    fn dict_literal_in_decl() {
        let c = compiled("x := {1: \"one\", 2: \"two\"};\n");
        assert_eq!(c.instruction_count, 1);
    }

    #[test]
    fn dict_index() {
        let c = compiled("x := {1: \"one\"};\ny := x[1];\n");
        assert_eq!(c.instruction_count, 2);
    }

    #[test]
    fn list_index_out_of_range_is_error() {
        let err = compiled_err("x := [1, 2];\ny := x[5];\n");
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn dict_missing_key_is_error() {
        let err = compiled_err("x := {1: \"a\"};\ny := x[99];\n");
        assert!(err.to_string().contains("Key not found"));
    }

    // -------------------------------------------------------------------------
    // Boolean & comparison ops
    // -------------------------------------------------------------------------

    #[test]
    fn boolean_ops() {
        compiled("x := True and False;\n");
        compiled("x := True or False;\n");
        compiled("x := not True;\n");
    }

    #[test]
    fn comparison_ops() {
        compiled("x := 1 == 1;\n");
        compiled("x := 1 != 2;\n");
        compiled("x := 1 < 2;\n");
        compiled("x := 2 > 1;\n");
        compiled("x := 1 <= 1;\n");
        compiled("x := 2 >= 2;\n");
    }

    #[test]
    fn in_operator_list() {
        compiled("x := 1 in [1, 2, 3];\n");
    }

    #[test]
    fn in_operator_dict() {
        compiled("x := 1 in {1: \"a\", 2: \"b\"};\n");
    }
}
