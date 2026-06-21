use crate::parser::{NotPythonExpr, NotPythonExprOp, NotPythonStmt};
use crate::visitor::Visitor;

enum Lit {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

fn expr_to_lit(expr: &NotPythonExpr) -> Option<Lit> {
    match expr {
        NotPythonExpr::Int(i) => Some(Lit::Int(*i)),
        NotPythonExpr::Float(f) => Some(Lit::Float(*f)),
        NotPythonExpr::Boolean(b) => Some(Lit::Bool(*b)),
        NotPythonExpr::String(s) => Some(Lit::Str(s.clone())),
        _ => None,
    }
}

fn lit_to_expr(lit: Lit) -> NotPythonExpr {
    match lit {
        Lit::Int(i) => NotPythonExpr::Int(i),
        Lit::Float(f) => NotPythonExpr::Float(f),
        Lit::Bool(b) => NotPythonExpr::Boolean(b),
        Lit::Str(s) => NotPythonExpr::String(s),
    }
}

fn fold_binary(lhs: &Lit, rhs: &Lit, op: &NotPythonExprOp) -> Option<Lit> {
    match op {
        NotPythonExprOp::Add(_, _) => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => a.checked_add(*b).map(Lit::Int),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Float(*a as f64 + b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Float(a + *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Float(a + b)),
            (Lit::Str(a), Lit::Str(b)) => Some(Lit::Str(a.clone() + b)),
            _ => None,
        },
        NotPythonExprOp::Sub(_, _) => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => a.checked_sub(*b).map(Lit::Int),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Float(*a as f64 - b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Float(a - *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Float(a - b)),
            _ => None,
        },
        NotPythonExprOp::Mul(_, _) => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => a.checked_mul(*b).map(Lit::Int),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Float(*a as f64 * b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Float(a * *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Float(a * b)),
            _ => None,
        },
        NotPythonExprOp::Div(_, _) => match (lhs, rhs) {
            (_, Lit::Int(0)) => None, // let runtime fire the division-by-zero error
            (Lit::Int(a), Lit::Int(b)) => a.checked_div(*b).map(Lit::Int),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Float(*a as f64 / b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Float(a / *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Float(a / b)),
            _ => None,
        },
        NotPythonExprOp::Mod(_, _) => match (lhs, rhs) {
            (_, Lit::Int(0)) => None, // let runtime fire the modulo-by-zero error
            (Lit::Int(a), Lit::Int(b)) => a.checked_rem(*b).map(Lit::Int),
            _ => None,
        },
        NotPythonExprOp::And(_, _) => match (lhs, rhs) {
            (Lit::Bool(a), Lit::Bool(b)) => Some(Lit::Bool(*a && *b)),
            _ => None,
        },
        NotPythonExprOp::Or(_, _) => match (lhs, rhs) {
            (Lit::Bool(a), Lit::Bool(b)) => Some(Lit::Bool(*a || *b)),
            _ => None,
        },
        NotPythonExprOp::Equal(_, _) => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a == b)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a == b)),
            (Lit::Bool(a), Lit::Bool(b)) => Some(Lit::Bool(a == b)),
            (Lit::Str(a), Lit::Str(b)) => Some(Lit::Bool(a == b)),
            _ => None,
        },
        NotPythonExprOp::NotEqual(_, _) => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a != b)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a != b)),
            (Lit::Bool(a), Lit::Bool(b)) => Some(Lit::Bool(a != b)),
            (Lit::Str(a), Lit::Str(b)) => Some(Lit::Bool(a != b)),
            _ => None,
        },
        NotPythonExprOp::Greater(_, _) => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a > b)),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Bool((*a as f64) > *b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Bool(*a > *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a > b)),
            _ => None,
        },
        NotPythonExprOp::Less(_, _) => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a < b)),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Bool((*a as f64) < *b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Bool(*a < *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a < b)),
            _ => None,
        },
        NotPythonExprOp::GreaterEqual(_, _) => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a >= b)),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Bool((*a as f64) >= *b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Bool(*a >= *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a >= b)),
            _ => None,
        },
        NotPythonExprOp::LessEqual(_, _) => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a <= b)),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Bool((*a as f64) <= *b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Bool(*a <= *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a <= b)),
            _ => None,
        },
        NotPythonExprOp::In(_, _) => None,
        _ => None,
    }
}

fn fold_unary(val: &Lit, op: &NotPythonExprOp) -> Option<Lit> {
    match op {
        NotPythonExprOp::Neg(_) => match val {
            Lit::Int(i) => i.checked_neg().map(Lit::Int),
            Lit::Float(f) => Some(Lit::Float(-f)),
            _ => None,
        },
        NotPythonExprOp::Not(_) => match val {
            Lit::Bool(b) => Some(Lit::Bool(!b)),
            _ => None,
        },
        _ => None,
    }
}

struct ConstantFolder;

impl Visitor for ConstantFolder {
    fn post_expr(&mut self, expr: NotPythonExpr) -> NotPythonExpr {
        let folded = if let NotPythonExpr::Op(ref op) = expr {
            match op {
                NotPythonExprOp::Neg(v) | NotPythonExprOp::Not(v) => {
                    expr_to_lit(v).and_then(|l| fold_unary(&l, op))
                }
                NotPythonExprOp::Add(l, r)
                | NotPythonExprOp::Sub(l, r)
                | NotPythonExprOp::Mul(l, r)
                | NotPythonExprOp::Div(l, r)
                | NotPythonExprOp::Mod(l, r)
                | NotPythonExprOp::And(l, r)
                | NotPythonExprOp::Or(l, r)
                | NotPythonExprOp::Equal(l, r)
                | NotPythonExprOp::NotEqual(l, r)
                | NotPythonExprOp::Greater(l, r)
                | NotPythonExprOp::Less(l, r)
                | NotPythonExprOp::GreaterEqual(l, r)
                | NotPythonExprOp::LessEqual(l, r)
                | NotPythonExprOp::In(l, r) => expr_to_lit(l)
                    .zip(expr_to_lit(r))
                    .and_then(|(ll, rl)| fold_binary(&ll, &rl, op)),
            }
        } else {
            None
        };
        folded.map(lit_to_expr).unwrap_or(expr)
    }
}

pub fn fold_expr(expr: &mut NotPythonExpr) {
    let owned = std::mem::replace(expr, NotPythonExpr::None);
    *expr = ConstantFolder.visit_expr(owned);
}

pub fn fold_stmt(stmt: &mut NotPythonStmt) {
    let owned = std::mem::replace(stmt, NotPythonStmt::Pass);
    *stmt = ConstantFolder.visit_stmt(owned);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{NotPythonProgram, NotPythonStmt, parse_program};

    fn parsed_decl_expr(src: &str) -> NotPythonExpr {
        let prog: NotPythonProgram = parse_program(src).unwrap();
        let NotPythonStmt::Block(stmts) = prog.statement else {
            panic!("expected top-level Block");
        };
        let NotPythonStmt::Decl(_, expr) = stmts.into_iter().next().unwrap() else {
            panic!("expected Decl");
        };
        expr
    }

    #[test]
    fn test_mul_chain_folds_to_int() {
        assert_eq!(
            parsed_decl_expr("x = 9*9*9*9*9*9"),
            NotPythonExpr::Int(531441)
        );
    }

    #[test]
    fn test_mixed_arithmetic_folds() {
        assert_eq!(parsed_decl_expr("x = 3 + 4 * 2"), NotPythonExpr::Int(11));
    }

    #[test]
    fn test_div_by_zero_not_folded() {
        // Should remain an Op node, not panic at parse time
        let expr = parsed_decl_expr("x = 1 / 0");
        assert!(matches!(expr, NotPythonExpr::Op(_)));
    }

    #[test]
    fn test_bool_fold() {
        assert_eq!(
            parsed_decl_expr("x = true and false"),
            NotPythonExpr::Boolean(false)
        );
        assert_eq!(
            parsed_decl_expr("x = not true"),
            NotPythonExpr::Boolean(false)
        );
    }

    #[test]
    fn test_comparison_fold() {
        assert_eq!(parsed_decl_expr("x = 3 < 5"), NotPythonExpr::Boolean(true));
        assert_eq!(
            parsed_decl_expr("x = 10 == 10"),
            NotPythonExpr::Boolean(true)
        );
    }

    #[test]
    fn test_identifier_blocks_fold() {
        // i - 1 cannot be folded because i is an identifier
        let expr = parsed_decl_expr("x = i - 1");
        assert!(matches!(expr, NotPythonExpr::Op(_)));
    }
}
