#![feature(iter_collect_into)]
#![feature(iterator_try_collect)]

mod compile;
mod fold_constants;
mod index_variables;
mod lexer;
mod parser;
mod visitor;

pub use compile::{
    CompilingMetadata, FnArgVec, HashableProgramValue, PredefinedFunction, ProgramValue,
    compile_with_meta,
};
pub use lexer::NotPythonLangToken;
pub use parser::{NotPythonExpr, NotPythonExprOp, NotPythonProgram, NotPythonStmt, parse_program};
