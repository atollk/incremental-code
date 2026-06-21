#![feature(iter_collect_into)]

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
pub use parser::{
    BinaryOp, NotPythonExpr, NotPythonProgram, NotPythonStmt, UnaryOp, parse_program,
};
