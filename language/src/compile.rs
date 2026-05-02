use crate::parser::{NotPythonExpr, NotPythonExprOp, NotPythonProgram, NotPythonStmt};
use anyhow::{anyhow, bail};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::ops::Deref;

pub trait CompilingMetadata {
    fn log_zero_instruction(&mut self) -> anyhow::Result<()>;
    fn log_atomic_instruction(&mut self) -> anyhow::Result<()>;
}

impl CompilingMetadata for () {
    fn log_zero_instruction(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn log_atomic_instruction(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Validates and executes a parsed [`NotPythonProgram`], returning an error if execution fails.
pub fn compile(program: &NotPythonProgram) -> anyhow::Result<()> {
    compile_with_meta(program, &mut ())
}

/// Like [`compile`], but calls back into `meta` at each zero-cost and atomic instruction
/// so callers can measure or profile execution.
pub fn compile_with_meta(
    program: &NotPythonProgram,
    meta: &mut impl CompilingMetadata,
) -> anyhow::Result<()> {
    let mut state = ProgramExecutionState {
        control_flow: ProgramExecutionControlFlow::Normal,
        call_stack: vec![ProgramExecutionCallState::default()],
    };
    compile_stmt(&program.statement, &mut state, meta)
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

enum Callable<'a> {
    PredefinedFunction {
        body: Box<dyn Fn(Vec<ProgramValue>) -> anyhow::Result<ProgramValue>>,
        params: Vec<String>,
    },
    UserFunction(&'a NotPythonStmt),
}

impl<'a> Callable<'a> {
    fn call(
        &self,
        name: &str,
        args: &[NotPythonExpr],
        state: &mut ProgramExecutionState<'a>,
        meta: &mut impl CompilingMetadata,
    ) -> anyhow::Result<ProgramValue> {
        let predefined_function_body = if let Callable::PredefinedFunction { params, body } = self {
            Some(|args| body(args))
        } else {
            None
        };
        let user_function_body =
            if let Callable::UserFunction(NotPythonStmt::Function { params, body, .. }) = self {
                let f = |args: Vec<ProgramValue>| -> anyhow::Result<ProgramValue> {
                    let mut frame = ProgramExecutionCallState::default();
                    for (param, val) in params.into_iter().zip(args) {
                        frame.variables.insert(param.as_str(), val);
                    }
                    state.call_stack.push(frame);

                    // Run function & return
                    compile_stmt(body, state, meta)?;
                    state.call_stack.pop();
                    let return_value = match std::mem::replace(
                        &mut state.control_flow,
                        ProgramExecutionControlFlow::Normal,
                    ) {
                        ProgramExecutionControlFlow::Return(v) => v,
                        _ => ProgramValue::None,
                    };
                    Ok(return_value)
                };
                Some(f)
            } else {
                None
            };
        let body_function = match (predefined_function_body, user_function_body) {
            (Some(_), Some(_)) => unreachable!(),
            (Some(f), None) => &f as &(dyn FnMut(&[ProgramValue]) -> anyhow::Result<ProgramValue>),
            (None, Some(f)) => &f,
            (None, None) => bail!("'{}' is not a function", name),
        };
        let arg_values: Vec<ProgramValue> = args
            .iter()
            .map(|a| eval_expr(a, state, meta))
            .collect::<anyhow::Result<_>>()?;
        body_function(&arg_values)
    }
}

struct ProgramExecutionState<'a> {
    control_flow: ProgramExecutionControlFlow,
    call_stack: Vec<ProgramExecutionCallState<'a>>,
}

impl<'a> ProgramExecutionState<'a> {
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

    fn get_function(&self, name: &str) -> anyhow::Result<Callable> {
        for call_state in self.call_stack.iter().rev() {
            if let Some(&stmt) = call_state.functions.get(name) {
                return Ok(Callable::UserFunction(stmt));
            }
        }
        Err(anyhow!("Function {} not found", name))
    }
}

fn compile_stmt<'a>(
    stmt: &'a NotPythonStmt,
    state: &mut ProgramExecutionState<'a>,
    meta: &mut impl CompilingMetadata,
) -> anyhow::Result<()> {
    if matches!(state.control_flow, ProgramExecutionControlFlow::Return(_)) {
        meta.log_zero_instruction()?;
        return Ok(());
    }
    match stmt {
        NotPythonStmt::Block(stmts) => {
            for stmt in stmts {
                compile_stmt(stmt, state, meta)?;
            }
        }
        NotPythonStmt::Pass => meta.log_atomic_instruction()?,
        NotPythonStmt::Break => {
            state.control_flow = ProgramExecutionControlFlow::Break;
            meta.log_atomic_instruction()?;
        }
        NotPythonStmt::Continue => {
            state.control_flow = ProgramExecutionControlFlow::Continue;
            meta.log_atomic_instruction()?;
        }
        NotPythonStmt::Return(expr) => {
            state.control_flow = ProgramExecutionControlFlow::Return(
                expr.as_ref()
                    .map(|e| eval_expr(e, state, meta))
                    .unwrap_or(Ok(ProgramValue::None))?,
            );
            meta.log_atomic_instruction()?;
        }
        NotPythonStmt::Decl(name, expr) => {
            let expr = eval_expr(expr, state, meta)?;
            state.decl_variable(name, expr);
            meta.log_atomic_instruction()?;
        }
        NotPythonStmt::Assign(name, expr) => {
            state.get_variable(name)?;
            let expr = eval_expr(expr, state, meta)?;
            state.assign_variable(name, expr)?;
            meta.log_atomic_instruction()?;
        }
        NotPythonStmt::If {
            condition,
            then,
            else_,
        } => match eval_expr(condition, state, meta)? {
            ProgramValue::Hashable(HashableProgramValue::Bool(b)) => {
                if b {
                    compile_stmt(then, state, meta)?;
                    meta.log_atomic_instruction()?;
                } else if let Some(else_) = else_ {
                    compile_stmt(else_, state, meta)?;
                    meta.log_atomic_instruction()?;
                } else {
                    meta.log_atomic_instruction()?;
                }
            }
            _ => bail!("Condition expression is not a boolean"),
        },
        NotPythonStmt::Loop(body) => {
            state.call_stack.last_mut().unwrap().loop_nesting += 1;
            meta.log_atomic_instruction()?;
            loop {
                compile_stmt(body, state, meta)?;
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
        }
        NotPythonStmt::Function {
            name,
            params: _,
            body: _,
        } => {
            state.decl_function(name, stmt);
            meta.log_atomic_instruction()?;
        }
        NotPythonStmt::Call(name, args) => {
            let func = state.get_function(name)?;
            func.call(name, args, state, meta)?;
            meta.log_atomic_instruction()?;
        }
    };
    Ok(())
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
    meta: &mut impl CompilingMetadata,
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
                .map(|ex| eval_expr(ex, state, meta))
                .collect::<anyhow::Result<_>>()?,
        )),
        NotPythonExpr::Dict(d) => {
            let mut map = HashMap::new();
            for (k, v) in d {
                let key = eval_expr(k, state, meta)?;
                let val = eval_expr(v, state, meta)?;
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
            | NotPythonExprOp::In(lhs, rhs) => eval_binary_op(
                eval_expr(lhs, state, meta)?,
                eval_expr(rhs, state, meta)?,
                o,
                state,
            ),
            NotPythonExprOp::Neg(val) | NotPythonExprOp::Not(val) => {
                eval_unary_op(eval_expr(val, state, meta)?, o, state)
            }
        },
        NotPythonExpr::Call(name_expr, args) => {
            let name = match name_expr.deref() {
                NotPythonExpr::Identifier(n) => n.as_str(),
                _ => return Err(anyhow!("Can only call named functions")),
            };
            let func = state.get_function(name)?;
            func.call(name, args, state, meta)
        }
        NotPythonExpr::Index(lhs, rhs) => {
            let lhs = eval_expr(lhs, state, meta)?;
            let rhs = eval_expr(rhs, state, meta)?;
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
    use crate::parser::parse_program;

    impl CompilingMetadata for u32 {
        fn log_zero_instruction(&mut self) -> anyhow::Result<()> {
            Ok(())
        }
        fn log_atomic_instruction(&mut self) -> anyhow::Result<()> {
            *self += 1;
            Ok(())
        }
    }

    fn compiled(src: &str) -> u32 {
        let mut count: u32 = 0;
        compile_with_meta(&parse_program(src).unwrap(), &mut count).unwrap();
        count
    }

    fn compiled_err(src: &str) -> anyhow::Error {
        let program = parse_program(src).unwrap();
        let mut state = ProgramExecutionState {
            control_flow: ProgramExecutionControlFlow::Normal,
            call_stack: vec![ProgramExecutionCallState::default()],
        };
        compile_stmt(&program.statement, &mut state, &mut ()).unwrap_err()
    }

    // -------------------------------------------------------------------------
    // instruction_count
    // -------------------------------------------------------------------------

    #[test]
    fn single_pass_counts_one() {
        assert_eq!(compiled("pass;"), 1);
    }

    #[test]
    fn two_stmts_count_two() {
        assert_eq!(compiled("pass;\npass;\n"), 2);
    }

    #[test]
    fn execution_time_positive() {
        assert!(compiled("pass;") > 0);
    }

    #[test]
    fn later_stmts_have_lower_time() {
        assert!(compiled("pass;\npass;\n") > compiled("pass;"));
    }

    // -------------------------------------------------------------------------
    // Variables
    // -------------------------------------------------------------------------

    #[test]
    fn decl_and_assign() {
        assert_eq!(compiled("x := 42;\nx = 1;\n"), 2);
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
        assert_eq!(compiled("x := 1 + 2 * 3;\n"), 1);
    }

    #[test]
    fn negation_in_decl() {
        assert_eq!(compiled("x := -5;\n"), 1);
    }

    #[test]
    fn string_concat() {
        assert_eq!(compiled("x := \"hello\" + \" world\";\n"), 1);
    }

    // -------------------------------------------------------------------------
    // Conditionals
    // -------------------------------------------------------------------------

    #[test]
    fn if_true_branch_executes() {
        assert_eq!(compiled("if True:\npass;\nend\n"), 2);
    }

    #[test]
    fn if_false_branch_skipped() {
        assert_eq!(compiled("if False:\npass;\nend\n"), 1);
    }

    #[test]
    fn if_else_false_takes_else() {
        assert_eq!(compiled("if False:\npass;\nelse:\nbreak;\nend\n"), 2);
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
        assert_eq!(compiled("loop:\nbreak;\nend\n"), 2);
    }

    #[test]
    fn loop_body_executes_multiple_times() {
        let src = "x := 3;\nloop:\nif x == 1:\nbreak;\nend\nx = x + -1;\nend\n";
        assert!(compiled(src) > 3);
    }

    #[test]
    fn loop_adds_more_instructions_than_sequential() {
        let looped = compiled("loop:\npass;\npass;\nbreak;\nend\n");
        let sequential = compiled("pass;\npass;\n");
        assert!(looped > sequential);
    }

    // -------------------------------------------------------------------------
    // Functions
    // -------------------------------------------------------------------------

    #[test]
    fn function_def_counts_one() {
        assert_eq!(compiled("def foo():\npass;\nend\n"), 1);
    }

    #[test]
    fn function_call_executes_body() {
        // def (1) + call stmt (1 overhead + body pass 1) = 3
        assert_eq!(compiled("def foo():\npass;\nend\nfoo();\n"), 3);
    }

    #[test]
    fn function_with_params() {
        assert_eq!(compiled("def add(x, y):\npass;\nend\nadd(1, 2);\n"), 3);
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
        assert_eq!(compiled("x := [1, 2, 3];\n"), 1);
    }

    #[test]
    fn list_index() {
        assert_eq!(compiled("x := [10, 20, 30];\ny := x[1];\n"), 2);
    }

    #[test]
    fn dict_literal_in_decl() {
        assert_eq!(compiled("x := {1: \"one\", 2: \"two\"};\n"), 1);
    }

    #[test]
    fn dict_index() {
        assert_eq!(compiled("x := {1: \"one\"};\ny := x[1];\n"), 2);
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
