use crate::parser::{
    BinaryOp, NotPythonExpr, NotPythonExprVariable, NotPythonProgram, NotPythonStmt, UnaryOp,
};
use crate::string_hasher::HashedString;
use linear_map::LinearMap;
use smallvec::SmallVec;
use std::cmp::PartialEq;
use std::collections::HashMap;

#[derive(thiserror::Error, Debug, Clone)]
#[error("{msg}")]
pub struct CompileError {
    // Box the String to reduce size of this struct to one word
    #[allow(clippy::box_collection)]
    msg: Box<String>,
}

impl CompileError {
    pub fn new(msg: String) -> Self {
        Self { msg: Box::new(msg) }
    }
}

pub type CompileResult<T> = Result<T, CompileError>;

fn new_compile_err<T>(msg: String) -> CompileResult<T> {
    Err(CompileError::new(msg))
}

pub trait CompilingMetadata: Clone {
    type Diff;
    fn log_zero_instruction(&mut self) -> CompileResult<()>;
    fn log_atomic_instruction(&mut self) -> CompileResult<()>;
    fn diff(&self, other: &Self) -> CompileResult<Self::Diff>;
    fn add_assign(&mut self, diff: &Self::Diff) -> CompileResult<()>;
}

impl CompilingMetadata for () {
    type Diff = ();

    fn log_zero_instruction(&mut self) -> CompileResult<()> {
        Ok(())
    }

    fn log_atomic_instruction(&mut self) -> CompileResult<()> {
        Ok(())
    }

    fn diff(&self, _other: &Self) -> CompileResult<Self::Diff> {
        Ok(())
    }

    fn add_assign(&mut self, _diff: &Self::Diff) -> CompileResult<()> {
        Ok(())
    }
}

/// Like [`compile`], but calls back into `meta` at each zero-cost and atomic instruction
/// so callers can measure or profile execution.
pub fn compile_with_meta<Meta: CompilingMetadata>(
    program: &NotPythonProgram,
    predefined_functions: HashMap<&str, PredefinedFunction<Meta>>,
    meta: &mut Meta,
) -> CompileResult<()> {
    let variable_len = 100; // TODO
    let mut state = ProgramExecutionState::new(variable_len, predefined_functions);
    compile_stmt(&program.statement, &mut state, meta)
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum HashableProgramValue {
    Int(i64),
    String(u64),
    Bool(bool),
}

#[derive(PartialEq, Clone, Debug)]
pub enum ProgramValue {
    Int(i64),
    Float(f64),
    String(HashedString),
    Bool(bool),
    None,
    List(Box<Vec<ProgramValue>>),
    Dict(Box<HashMap<HashableProgramValue, ProgramValue>>),
}

fn to_hashable(v: &ProgramValue) -> Option<HashableProgramValue> {
    match v {
        ProgramValue::Int(i) => Some(HashableProgramValue::Int(*i)),
        ProgramValue::String(s) => Some(HashableProgramValue::String(s.hash())),
        ProgramValue::Bool(b) => Some(HashableProgramValue::Bool(*b)),
        _ => None,
    }
}

#[derive(PartialEq, Clone, Debug)]
enum ProgramExecutionControlFlow {
    Normal,
    Continue,
    Break,
    Return(ProgramValue),
}

struct ProgramExecutionCallState<'a> {
    variables: Vec<Option<ProgramValue>>,
    functions: LinearMap<&'a str, &'a NotPythonStmt>,
    loop_nesting: usize,
    is_pure: bool,
}

impl ProgramExecutionCallState<'_> {
    fn new(variable_len: usize) -> Self {
        Self {
            variables: vec![None; variable_len],
            functions: LinearMap::new(),
            loop_nesting: 0,
            is_pure: false,
        }
    }
}

pub type FnArgVec<T> = SmallVec<[T; 4]>;

pub type PredefinedFunction<Meta> = fn(&mut Meta, &[ProgramValue]) -> CompileResult<ProgramValue>;

enum Callable<'a, Meta: CompilingMetadata> {
    PredefinedFunction(PredefinedFunction<Meta>),
    UserFunction(&'a NotPythonStmt),
}

impl<'a, Meta: CompilingMetadata> Callable<'a, Meta> {
    fn call(
        self,
        name: &str,
        args: &[NotPythonExpr],
        state: &mut ProgramExecutionState<'a, Meta>,
        meta: &mut Meta,
    ) -> CompileResult<ProgramValue> {
        let arg_values = {
            let mut arg_values = FnArgVec::new();
            for arg in args {
                arg_values.push(eval_expr(arg, state, meta)?);
            }
            arg_values
        };
        match self {
            Callable::PredefinedFunction(body) => body(meta, &arg_values),
            Callable::UserFunction(NotPythonStmt::Function {
                params,
                body,
                is_pure,
                name: fn_name,
            }) => {
                if arg_values.len() != params.len() {
                    return new_compile_err(format!(
                        "'{}' expects {} arguments but got {}",
                        name,
                        params.len(),
                        arg_values.len()
                    ));
                }

                let cache_key = if *is_pure {
                    // Convert args to hashable cache keys; non-hashable args are an error.
                    let cache_key: Vec<HashableProgramValue> = arg_values
                        .iter()
                        .map(|v| {
                            to_hashable(v).ok_or_else(|| {
                                CompileError::new(format!(
                                    "Pure function '{}' received non-hashable argument",
                                    name
                                ))
                            })
                        })
                        .collect::<CompileResult<_>>()?;

                    // Cache hit: return immediately without executing the body.
                    if let Some((value, diff)) = state
                        .pure_caches
                        .get(&(fn_name.as_str(), cache_key.clone()))
                    {
                        meta.add_assign(diff)?;
                        return Ok(value.clone());
                    }

                    Some(cache_key)
                } else {
                    None
                };

                // Cache miss / unpure context: execute with a pure frame.
                let mut frame = {
                    let mut frame = ProgramExecutionCallState::new(
                        state.call_stack.last().unwrap().variables.len(),
                    );
                    frame.is_pure = *is_pure;
                    frame
                };
                for (param, val) in params.iter().zip(arg_values) {
                    frame.variables[param.index] = Some(val);
                }
                let meta_clone = meta.clone();
                state.call_stack.push(frame);
                let body_result =
                    stacker::maybe_grow(32 * 1024, 1024 * 1024, || compile_stmt(body, state, meta));
                state.call_stack.pop();
                body_result?;
                let return_value = match std::mem::replace(
                    &mut state.control_flow,
                    ProgramExecutionControlFlow::Normal,
                ) {
                    ProgramExecutionControlFlow::Return(v) => v,
                    _ => ProgramValue::None,
                };

                if *is_pure {
                    // Store in cache.
                    state.pure_caches.insert(
                        (fn_name.as_str(), cache_key.unwrap()),
                        (return_value.clone(), meta.diff(&meta_clone)?),
                    );
                }

                Ok(return_value)
            }
            _ => new_compile_err(format!("'{}' is not a function", name)),
        }
    }
}

struct ProgramExecutionState<'a, Meta: CompilingMetadata> {
    control_flow: ProgramExecutionControlFlow,
    call_stack: Vec<ProgramExecutionCallState<'a>>,
    predefined_functions: LinearMap<&'a str, PredefinedFunction<Meta>>,
    pure_caches: HashMap<(&'a str, Vec<HashableProgramValue>), (ProgramValue, Meta::Diff)>,
}

impl<'a, Meta: CompilingMetadata> ProgramExecutionState<'a, Meta> {
    fn new(
        variable_len: usize,
        predefined_functions: HashMap<&'a str, PredefinedFunction<Meta>>,
    ) -> Self {
        Self {
            control_flow: ProgramExecutionControlFlow::Normal,
            call_stack: vec![ProgramExecutionCallState::new(variable_len)],
            predefined_functions: LinearMap::from_iter(predefined_functions.into_iter()),
            pure_caches: HashMap::new(),
        }
    }
}

impl<'a, Meta: CompilingMetadata> ProgramExecutionState<'a, Meta> {
    fn in_pure_context(&self) -> bool {
        self.call_stack.last().map(|f| f.is_pure).unwrap_or(false)
    }

    fn get_variable(&self, variable: &NotPythonExprVariable) -> CompileResult<&ProgramValue> {
        if self.in_pure_context() {
            // Only look in the top (pure) frame.
            let variable_value = self
                .call_stack
                .last()
                .and_then(|f| f.variables.get(variable.index))
                .ok_or_else(|| {
                    CompileError::new(format!(
                        "Pure function accesses non-local variable '{}'",
                        variable.name
                    ))
                })?;
            return variable_value.as_ref().ok_or(CompileError::new(format!(
                "Variable '{}' has no assigned value",
                variable.name
            )));
        }
        for call_state in self.call_stack.iter().rev() {
            if let Some(Some(variable_value)) = call_state.variables.get(variable.index) {
                return Ok(variable_value);
            }
        }
        new_compile_err(format!("Variable {} not found", variable.name))
    }

    fn decl_variable(&mut self, variable: &NotPythonExprVariable, value: ProgramValue) {
        let frame = self.call_stack.last_mut().unwrap();
        frame.variables[variable.index] = Some(value);
    }

    fn assign_variable(
        &mut self,
        variable: &NotPythonExprVariable,
        value: ProgramValue,
    ) -> CompileResult<()> {
        if self.in_pure_context() {
            // Only allow assigning to locals in the top (pure) frame.
            let frame = self.call_stack.last_mut().unwrap();
            if let Some(Some(variable_value)) = frame.variables.get_mut(variable.index) {
                *variable_value = value;
                return Ok(());
            }
            return new_compile_err(format!(
                "Pure function accesses non-local variable '{}'",
                variable.name
            ));
        }
        for call_state in self.call_stack.iter_mut().rev() {
            if let Some(Some(variable_value)) = call_state.variables.get_mut(variable.index) {
                *variable_value = value;
                return Ok(());
            }
        }
        new_compile_err(format!("Variable {} not found", variable.name))
    }

    fn decl_function(&mut self, name: &'a str, stmt: &'a NotPythonStmt) {
        self.call_stack
            .last_mut()
            .unwrap()
            .functions
            .insert(name, stmt);
    }

    fn get_function(&self, name: &str) -> CompileResult<Callable<'a, Meta>> {
        let in_pure = self.in_pure_context();
        for call_state in self.call_stack.iter().rev() {
            if let Some(&stmt) = call_state.functions.get(name) {
                if in_pure {
                    // Inside a pure function, only pure user functions may be called.
                    if let NotPythonStmt::Function { is_pure, .. } = stmt {
                        if !is_pure {
                            return new_compile_err(format!(
                                "Pure function calls non-pure function '{name}'"
                            ));
                        }
                    }
                }
                return Ok(Callable::UserFunction(stmt));
            }
        }
        if let Some(&f) = self.predefined_functions.get(name) {
            // Predefined functions are always allowed, even from pure context.
            return Ok(Callable::PredefinedFunction(f));
        }
        new_compile_err(format!("Function {} not found", name))
    }
}

fn compile_stmt<'a, Meta: CompilingMetadata>(
    stmt: &'a NotPythonStmt,
    state: &mut ProgramExecutionState<'a, Meta>,
    meta: &mut Meta,
) -> CompileResult<()> {
    if matches!(state.control_flow, ProgramExecutionControlFlow::Return(_)) {
        meta.log_zero_instruction()?;
        return Ok(());
    }
    match stmt {
        NotPythonStmt::Block(stmts) => {
            for stmt in stmts {
                compile_stmt(stmt, state, meta)?;
                if !matches!(state.control_flow, ProgramExecutionControlFlow::Normal) {
                    break;
                }
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
        NotPythonStmt::Decl(var, expr) => {
            let expr = eval_expr(expr, state, meta)?;
            state.decl_variable(var, expr);
            meta.log_atomic_instruction()?;
        }
        NotPythonStmt::Assign(var, expr) => {
            state.get_variable(var)?;
            let expr = eval_expr(expr, state, meta)?;
            state.assign_variable(var, expr)?;
            meta.log_atomic_instruction()?;
        }
        NotPythonStmt::If {
            condition,
            then,
            else_,
        } => match eval_expr(condition, state, meta)? {
            ProgramValue::Bool(b) => {
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
            _ => return new_compile_err("Condition expression is not a boolean".to_string()),
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
            is_pure: _,
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

fn eval_unary_op(val: ProgramValue, op: UnaryOp) -> CompileResult<ProgramValue> {
    match op {
        UnaryOp::Neg => match val {
            ProgramValue::Int(i) => Ok(ProgramValue::Int(-i)),
            ProgramValue::Float(f) => Ok(ProgramValue::Float(-f)),
            _ => new_compile_err("Cannot negate non-numeric value".to_string()),
        },
        UnaryOp::Not => match val {
            ProgramValue::Bool(b) => Ok(ProgramValue::Bool(!b)),
            _ => new_compile_err("Cannot apply 'not' to non-boolean value".to_string()),
        },
    }
}

fn eval_binary_op(
    lhs: ProgramValue,
    rhs: ProgramValue,
    op: BinaryOp,
) -> CompileResult<ProgramValue> {
    use ProgramValue::*;

    match op {
        BinaryOp::Add => match (lhs, rhs) {
            (Int(a), Int(b)) => Ok(Int(a + b)),
            (Int(a), Float(b)) => Ok(Float(a as f64 + b)),
            (Float(a), Int(b)) => Ok(Float(a + b as f64)),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            (String(a), String(b)) => Ok(String(a + b)),
            _ => new_compile_err("'+' operands must be numeric or both strings".to_string()),
        },
        BinaryOp::Sub => match (lhs, rhs) {
            (Int(a), Int(b)) => Ok(Int(a - b)),
            (Int(a), Float(b)) => Ok(Float(a as f64 - b)),
            (Float(a), Int(b)) => Ok(Float(a - b as f64)),
            (Float(a), Float(b)) => Ok(Float(a - b)),
            _ => new_compile_err("'-' operands must be numeric".to_string()),
        },
        BinaryOp::Mul => match (lhs, rhs) {
            (Int(a), Int(b)) => Ok(Int(a * b)),
            (Int(a), Float(b)) => Ok(Float(a as f64 * b)),
            (Float(a), Int(b)) => Ok(Float(a * b as f64)),
            (Float(a), Float(b)) => Ok(Float(a * b)),
            _ => new_compile_err("'*' operands must be numeric".to_string()),
        },
        BinaryOp::Div => match (lhs, rhs) {
            (Int(a), Int(b)) => {
                if b == 0 {
                    return new_compile_err("Division by zero".to_string());
                }
                Ok(Int(a / b))
            }
            (Int(a), Float(b)) => Ok(Float(a as f64 / b)),
            (Float(a), Int(b)) => Ok(Float(a / b as f64)),
            (Float(a), Float(b)) => Ok(Float(a / b)),
            _ => new_compile_err("'/' operands must be numeric".to_string()),
        },
        BinaryOp::Mod => match (lhs, rhs) {
            (Int(a), Int(b)) => {
                if b == 0 {
                    return new_compile_err("Modulo by zero".to_string());
                }
                Ok(Int(a % b))
            }
            _ => new_compile_err("'%' operands must be integers".to_string()),
        },
        BinaryOp::And => match (lhs, rhs) {
            (Bool(a), Bool(b)) => Ok(Bool(a && b)),
            _ => new_compile_err("'and' operands must be booleans".to_string()),
        },
        BinaryOp::Or => match (lhs, rhs) {
            (Bool(a), Bool(b)) => Ok(Bool(a || b)),
            _ => new_compile_err("'or' operands must be booleans".to_string()),
        },
        BinaryOp::Equal => match (lhs, rhs) {
            (Int(a), Int(b)) => Ok(Bool(a == b)),
            (Float(a), Float(b)) => Ok(Bool(a == b)),
            (String(a), String(b)) => Ok(Bool(a == b)),
            (Bool(a), Bool(b)) => Ok(Bool(a == b)),
            (None, None) => Ok(Bool(true)),
            _ => Ok(Bool(false)),
        },
        BinaryOp::NotEqual => match (lhs, rhs) {
            (Int(a), Int(b)) => Ok(Bool(a != b)),
            (Float(a), Float(b)) => Ok(Bool(a != b)),
            (String(a), String(b)) => Ok(Bool(a != b)),
            (Bool(a), Bool(b)) => Ok(Bool(a != b)),
            (None, None) => Ok(Bool(false)),
            _ => Ok(Bool(true)),
        },
        BinaryOp::Greater => match (lhs, rhs) {
            (Int(a), Int(b)) => Ok(Bool(a > b)),
            (Int(a), Float(b)) => Ok(Bool((a as f64) > b)),
            (Float(a), Int(b)) => Ok(Bool(a > b as f64)),
            (Float(a), Float(b)) => Ok(Bool(a > b)),
            _ => new_compile_err("'>' operands must be numeric".to_string()),
        },
        BinaryOp::Less => match (lhs, rhs) {
            (Int(a), Int(b)) => Ok(Bool(a < b)),
            (Int(a), Float(b)) => Ok(Bool((a as f64) < b)),
            (Float(a), Int(b)) => Ok(Bool(a < b as f64)),
            (Float(a), Float(b)) => Ok(Bool(a < b)),
            _ => new_compile_err("'<' operands must be numeric".to_string()),
        },
        BinaryOp::GreaterEqual => match (lhs, rhs) {
            (Int(a), Int(b)) => Ok(Bool(a >= b)),
            (Int(a), Float(b)) => Ok(Bool((a as f64) >= b)),
            (Float(a), Int(b)) => Ok(Bool(a >= b as f64)),
            (Float(a), Float(b)) => Ok(Bool(a >= b)),
            _ => new_compile_err("'>=' operands must be numeric".to_string()),
        },
        BinaryOp::LessEqual => match (lhs, rhs) {
            (Int(a), Int(b)) => Ok(Bool(a <= b)),
            (Int(a), Float(b)) => Ok(Bool((a as f64) <= b)),
            (Float(a), Int(b)) => Ok(Bool(a <= b as f64)),
            (Float(a), Float(b)) => Ok(Bool(a <= b)),
            _ => new_compile_err("'<=' operands must be numeric".to_string()),
        },
        BinaryOp::In => match rhs {
            List(l) => Ok(Bool(l.contains(&lhs))),
            Dict(d) => {
                if let Some(k) = to_hashable(&lhs) {
                    Ok(Bool(d.contains_key(&k)))
                } else {
                    new_compile_err("'in' for dicts requires a hashable key".to_string())
                }
            }
            _ => new_compile_err("'in' requires a list or dict on the right-hand side".to_string()),
        },
    }
}

fn eval_expr<'a, Meta: CompilingMetadata>(
    expr: &NotPythonExpr,
    state: &mut ProgramExecutionState<'a, Meta>,
    meta: &mut Meta,
) -> CompileResult<ProgramValue> {
    match expr {
        NotPythonExpr::Int(i) => Ok(ProgramValue::Int(*i)),
        NotPythonExpr::Float(f) => Ok(ProgramValue::Float(*f)),
        NotPythonExpr::String((_, hs)) => Ok(ProgramValue::String(hs.clone())),
        NotPythonExpr::Boolean(b) => Ok(ProgramValue::Bool(*b)),
        NotPythonExpr::None => Ok(ProgramValue::None),
        NotPythonExpr::Variable(var) => state.get_variable(var).cloned(),
        NotPythonExpr::List(l) => Ok(ProgramValue::List(Box::new(
            l.iter()
                .map(|ex| eval_expr(ex, state, meta))
                .collect::<CompileResult<_>>()?,
        ))),
        NotPythonExpr::Dict(d) => {
            let mut map = HashMap::new();
            for (k, v) in d {
                let key = eval_expr(k, state, meta)?;
                let val = eval_expr(v, state, meta)?;
                match to_hashable(&key) {
                    Some(h) => {
                        map.insert(h, val);
                    }
                    None => {
                        return new_compile_err(
                            "Dict keys must be hashable (int, string, or bool)".to_string(),
                        );
                    }
                }
            }
            Ok(ProgramValue::Dict(Box::new(map)))
        }
        NotPythonExpr::BinaryOp(op, lhs, rhs) => eval_binary_op(
            eval_expr(lhs, state, meta)?,
            eval_expr(rhs, state, meta)?,
            *op,
        ),
        NotPythonExpr::UnaryOp(op, val) => eval_unary_op(eval_expr(val, state, meta)?, *op),
        NotPythonExpr::Call(name, args) => {
            let func = state.get_function(name)?;
            func.call(name, args, state, meta)
        }
        NotPythonExpr::Index(lhs, rhs) => {
            let rhs = eval_expr(rhs, state, meta)?;
            let lhs = state.get_variable(lhs)?;
            match lhs {
                ProgramValue::List(l) => match rhs {
                    ProgramValue::Int(i) => l
                        .get(i as usize)
                        .ok_or_else(|| CompileError::new( format!("Index {i} out of range") ))
                        .cloned(),
                    _ => new_compile_err("Index operator on lists can only be used with integers.".to_string()),
                },
                ProgramValue::Dict(d) => match to_hashable(&rhs) {
                    Some(k) => d.get(&k).ok_or_else(|| CompileError::new( "Key not found in dict".to_string() )).cloned(),
                    None => new_compile_err("Index operator on dicts can only be used with integers, bools, or strings.".to_string()),
                },
                _ => new_compile_err("Index operator can only be used on lists or dicts.".to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_program;
    use assertables::{assert_err, assert_ge};

    impl CompilingMetadata for u32 {
        type Diff = i32;

        fn log_zero_instruction(&mut self) -> CompileResult<()> {
            Ok(())
        }
        fn log_atomic_instruction(&mut self) -> CompileResult<()> {
            *self += 1;
            Ok(())
        }

        fn diff(&self, other: &Self) -> CompileResult<Self::Diff> {
            Ok((*self as i32) - (*other as i32))
        }

        fn add_assign(&mut self, diff: &Self::Diff) -> CompileResult<()> {
            *self = self.checked_add_signed(*diff).unwrap();
            Ok(())
        }
    }

    fn compiled(src: &str) -> u32 {
        let mut count: u32 = 0;
        compile_with_meta(&parse_program(src).unwrap(), HashMap::new(), &mut count).unwrap();
        count
    }

    fn compiled_err(src: &str) -> CompileError {
        let program = parse_program(src).unwrap();
        let mut state = ProgramExecutionState {
            control_flow: ProgramExecutionControlFlow::Normal,
            call_stack: vec![ProgramExecutionCallState::new(10)],
            predefined_functions: LinearMap::new(),
            pure_caches: HashMap::new(),
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

    // -------------------------------------------------------------------------
    // Pure functions
    // -------------------------------------------------------------------------

    #[test]
    fn pure_function_returns_value() {
        let src = "def pure double(x):\nreturn x + x;\nend\ny := double(5);\n";
        // def (1) + return in body (1) + decl y (1)
        assert_eq!(compiled(src), 3);
    }

    #[test]
    fn pure_function_caches_result() {
        // Call the same pure function twice with the same args.
        // Second call should be a cache hit, so the body (which has a pass) doesn't count.
        let src = "def pure f(x):\npass;\nreturn x;\nend\na := f(1);\nb := f(1);\n";
        // First call: def(1) + body pass(1) + decl a(1) = 3
        // Second call (cache hit): no body pass, just decl b(1) = 1 extra
        let first_only = compiled("def pure f(x):\npass;\nreturn x;\nend\na := f(1);\n");
        let both = compiled(src);
        // The second call adds fewer instructions than the first (no body re-execution).
        assert!(both - first_only < first_only);
    }

    #[test]
    fn pure_function_different_args_not_cached() {
        // Calls with different args both execute the body.
        let src = "def pure f(x):\npass;\nreturn x;\nend\na := f(1);\nb := f(2);\n";
        let first_only = compiled("def pure f(x):\npass;\nreturn x;\nend\na := f(1);\n");
        let both = compiled(src);
        // Second call adds at least as many instructions as the first (body runs again).
        assert_ge!(both, first_only * 2 - 1);
    }

    #[test]
    fn pure_accessing_non_local_var_is_error() {
        compiled_err("x := 10;\ndef pure f():\ny := x;\nend\nf();\n");
    }

    #[test]
    fn pure_calls_pure_ok() {
        let src = "def pure double(x):\nreturn x + x;\nend\ndef pure quad(x):\nreturn double(x) + double(x);\nend\ny := quad(3);\n";
        compiled(src);
    }

    #[test]
    fn pure_calls_non_pure_is_error() {
        let err = compiled_err("def impure():\npass;\nend\ndef pure f():\nimpure();\nend\nf();\n");
        assert!(err.to_string().contains("non-pure"));
    }

    #[test]
    fn pure_calls_predefined_ok() {
        fn noop(_meta: &mut (), args: &[ProgramValue]) -> CompileResult<ProgramValue> {
            Ok(args
                .into_iter()
                .next()
                .cloned()
                .unwrap_or(ProgramValue::None))
        }
        let program =
            parse_program("def pure f(x):\nreturn identity(x);\nend\ny := f(1);\n").unwrap();
        let mut predefined = HashMap::new();
        predefined.insert("identity", noop as PredefinedFunction<()>);
        compile_with_meta(&program, predefined, &mut ()).unwrap();
    }

    #[test]
    fn pure_with_non_hashable_arg_is_error() {
        let err = compiled_err("def pure f(x):\nreturn x;\nend\nf([1, 2]);\n");
        assert!(err.to_string().contains("hashable"));
    }

    #[test]
    fn pure_local_decl_and_assign_ok() {
        let src = "def pure f(x):\ny := x + 1;\ny = y + 1;\nreturn y;\nend\nz := f(5);\n";
        compiled(src);
    }

    #[test]
    fn deep_recursion_does_not_stack_overflow() {
        let src =
            "def f(n):\nif n <= 0:\nreturn 0;\nelse:\nreturn f(n - 1);\nend\nend\nx := f(30000);\n";
        compiled(src);
    }
}
