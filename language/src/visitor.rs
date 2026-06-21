use crate::parser::{NotPythonExpr, NotPythonExprVariable, NotPythonStmt};

pub fn walk_expr_variable<V: Visitor + ?Sized>(
    _v: &mut V,
    var: NotPythonExprVariable,
) -> NotPythonExprVariable {
    var
}

pub fn walk_expr<V: Visitor + ?Sized>(v: &mut V, expr: NotPythonExpr) -> NotPythonExpr {
    match expr {
        NotPythonExpr::BinaryOp(op, l, r) => {
            NotPythonExpr::BinaryOp(op, Box::new(v.visit_expr(*l)), Box::new(v.visit_expr(*r)))
        }
        NotPythonExpr::UnaryOp(op, e) => NotPythonExpr::UnaryOp(op, Box::new(v.visit_expr(*e))),
        NotPythonExpr::Variable(var) => NotPythonExpr::Variable(v.visit_expr_variable(var)),
        NotPythonExpr::Index(var, idx) => {
            NotPythonExpr::Index(v.visit_expr_variable(var), Box::new(v.visit_expr(*idx)))
        }
        NotPythonExpr::List(items) => {
            NotPythonExpr::List(items.into_iter().map(|e| v.visit_expr(e)).collect())
        }
        NotPythonExpr::Dict(pairs) => NotPythonExpr::Dict(
            pairs
                .into_iter()
                .map(|(k, val)| (v.visit_expr(k), v.visit_expr(val)))
                .collect(),
        ),
        NotPythonExpr::Call(name, args) => {
            NotPythonExpr::Call(name, args.into_iter().map(|a| v.visit_expr(a)).collect())
        }
        other => other,
    }
}

pub fn walk_stmt<V: Visitor + ?Sized>(v: &mut V, stmt: NotPythonStmt) -> NotPythonStmt {
    match stmt {
        NotPythonStmt::Block(stmts) => {
            NotPythonStmt::Block(stmts.into_iter().map(|s| v.visit_stmt(s)).collect())
        }
        NotPythonStmt::Call(name, args) => {
            NotPythonStmt::Call(name, args.into_iter().map(|a| v.visit_expr(a)).collect())
        }
        NotPythonStmt::Decl(var, expr) => {
            NotPythonStmt::Decl(v.visit_expr_variable(var), v.visit_expr(expr))
        }
        NotPythonStmt::Assign(var, expr) => {
            NotPythonStmt::Assign(v.visit_expr_variable(var), v.visit_expr(expr))
        }
        NotPythonStmt::Return(Some(expr)) => NotPythonStmt::Return(Some(v.visit_expr(expr))),
        NotPythonStmt::If {
            condition,
            then,
            else_,
        } => NotPythonStmt::If {
            condition: v.visit_expr(condition),
            then: Box::new(v.visit_stmt(*then)),
            else_: else_.map(|e| Box::new(v.visit_stmt(*e))),
        },
        NotPythonStmt::Loop(body) => NotPythonStmt::Loop(Box::new(v.visit_stmt(*body))),
        NotPythonStmt::Function {
            name,
            params,
            body,
            is_pure,
        } => NotPythonStmt::Function {
            name,
            params: params
                .into_iter()
                .map(|p| v.visit_expr_variable(p))
                .collect(),
            body: Box::new(v.visit_stmt(*body)),
            is_pure,
        },
        other => other,
    }
}

pub trait Visitor {
    fn pre_expr_variable(&mut self, var: NotPythonExprVariable) -> NotPythonExprVariable {
        var
    }
    fn visit_expr_variable_children(
        &mut self,
        var: NotPythonExprVariable,
    ) -> NotPythonExprVariable {
        walk_expr_variable(self, var)
    }
    fn post_expr_variable(&mut self, var: NotPythonExprVariable) -> NotPythonExprVariable {
        var
    }

    fn pre_expr(&mut self, expr: NotPythonExpr) -> NotPythonExpr {
        expr
    }
    fn visit_expr_children(&mut self, expr: NotPythonExpr) -> NotPythonExpr {
        walk_expr(self, expr)
    }
    fn post_expr(&mut self, expr: NotPythonExpr) -> NotPythonExpr {
        expr
    }

    fn pre_stmt(&mut self, stmt: NotPythonStmt) -> NotPythonStmt {
        stmt
    }
    fn visit_stmt_children(&mut self, stmt: NotPythonStmt) -> NotPythonStmt {
        walk_stmt(self, stmt)
    }
    fn post_stmt(&mut self, stmt: NotPythonStmt) -> NotPythonStmt {
        stmt
    }

    fn visit_expr_variable(&mut self, var: NotPythonExprVariable) -> NotPythonExprVariable {
        let var = self.pre_expr_variable(var);
        let var = self.visit_expr_variable_children(var);
        self.post_expr_variable(var)
    }

    fn visit_expr(&mut self, expr: NotPythonExpr) -> NotPythonExpr {
        let expr = self.pre_expr(expr);
        let expr = self.visit_expr_children(expr);
        self.post_expr(expr)
    }

    fn visit_stmt(&mut self, stmt: NotPythonStmt) -> NotPythonStmt {
        let stmt = self.pre_stmt(stmt);
        let stmt = self.visit_stmt_children(stmt);
        self.post_stmt(stmt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{NotPythonProgram, parse_program};

    struct IdentityVisitor;
    impl Visitor for IdentityVisitor {}

    fn parse(src: &str) -> NotPythonStmt {
        let prog: NotPythonProgram = parse_program(src).unwrap();
        prog.statement
    }

    #[test]
    fn identity_visitor_preserves_literals() {
        let stmt = parse("x = 1 + 2");
        let result = IdentityVisitor.visit_stmt(stmt.clone());
        assert_eq!(result, stmt);
    }

    #[test]
    fn identity_visitor_preserves_nested_stmts() {
        let stmt = parse("if true:\n    x = 1\nelse:\n    x = 2");
        let result = IdentityVisitor.visit_stmt(stmt.clone());
        assert_eq!(result, stmt);
    }

    #[test]
    fn post_expr_hook_fires_on_all_exprs() {
        struct Counter(usize);
        impl Visitor for Counter {
            fn post_expr(&mut self, expr: NotPythonExpr) -> NotPythonExpr {
                self.0 += 1;
                expr
            }
        }

        // "x = 1 + 2" has three expressions: Int(1), Int(2), BinaryOp(Add)
        let stmt = parse("x = 1 + 2");
        let mut c = Counter(0);
        c.visit_stmt(stmt);
        assert_eq!(c.0, 3);
    }

    #[test]
    fn post_expr_variable_hook_fires() {
        struct VarCounter(usize);
        impl Visitor for VarCounter {
            fn post_expr_variable(&mut self, var: NotPythonExprVariable) -> NotPythonExprVariable {
                self.0 += 1;
                var
            }
        }

        // "x = y + z" has two variable reads (y, z) plus one in the Decl lhs
        let stmt = parse("x = y + z");
        let mut c = VarCounter(0);
        c.visit_stmt(stmt);
        assert_eq!(c.0, 3);
    }

    #[test]
    fn post_expr_can_rewrite_literals() {
        struct DoubleInts;
        impl Visitor for DoubleInts {
            fn post_expr(&mut self, expr: NotPythonExpr) -> NotPythonExpr {
                if let NotPythonExpr::Int(n) = expr {
                    NotPythonExpr::Int(n * 2)
                } else {
                    expr
                }
            }
        }

        let stmt = parse("x = 3");
        let result = DoubleInts.visit_stmt(stmt);
        let NotPythonStmt::Decl(_, expr) = result else {
            panic!()
        };
        assert_eq!(expr, NotPythonExpr::Int(6));
    }

    #[test]
    fn visit_stmt_children_override_can_skip_subtree() {
        struct NoOpVisitor;
        impl Visitor for NoOpVisitor {
            fn visit_stmt_children(&mut self, stmt: NotPythonStmt) -> NotPythonStmt {
                // Skip all children — return stmt as-is
                stmt
            }
            fn post_expr(&mut self, _expr: NotPythonExpr) -> NotPythonExpr {
                panic!("should not be reached");
            }
        }

        let stmt = parse("x = 1 + 2");
        // Should not panic
        NoOpVisitor.visit_stmt(stmt);
    }
}
