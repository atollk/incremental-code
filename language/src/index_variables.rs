// Optimization Post-Processing that replaces all Named access to variables by indexed access instead

use crate::NotPythonStmt;

/** Builds an index-list of all variables in a program and replaces their `index` fields. */
pub fn index_named_variable_access(root_stmt: &mut NotPythonStmt) {
    todo!()
}

fn index_named_variable_access_stmt(stmt: &mut NotPythonStmt) {}
