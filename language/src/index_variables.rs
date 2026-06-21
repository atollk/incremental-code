// Optimization Post-Processing that replaces all Named access to variables by indexed access instead

use std::collections::HashMap;

use crate::parser::{NotPythonExprVariable, NotPythonStmt};
use crate::visitor::Visitor;

/** Builds an index-list of all variables in a program and replaces their `index` fields. */
pub fn index_named_variable_access(root_stmt: NotPythonStmt) -> NotPythonStmt {
    VariableIndexer::default().visit_stmt(root_stmt)
}

#[derive(Default)]
struct VariableIndexer {
    name_to_index: HashMap<String, usize>,
    next_index: usize,
}

impl Visitor for VariableIndexer {
    fn post_expr_variable(&mut self, var: NotPythonExprVariable) -> NotPythonExprVariable {
        let idx = *self
            .name_to_index
            .entry(var.name.clone())
            .or_insert_with(|| {
                let i = self.next_index;
                self.next_index += 1;
                i
            });
        NotPythonExprVariable {
            name: var.name,
            index: idx,
        }
    }
}
