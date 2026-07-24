use crate::parser::{BinaryOp, NotPythonExpr, NotPythonStmt, UnaryOp};
use crate::string_hasher::HashedString;
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
        NotPythonExpr::String((s, _)) => Some(Lit::Str(s.clone())),
        _ => None,
    }
}

fn lit_to_expr(lit: Lit) -> NotPythonExpr {
    match lit {
        Lit::Int(i) => NotPythonExpr::Int(i),
        Lit::Float(f) => NotPythonExpr::Float(f),
        Lit::Bool(b) => NotPythonExpr::Boolean(b),
        Lit::Str(s) => {
            let hs = HashedString::from(s.as_str());
            NotPythonExpr::String((s, hs))
        }
    }
}

fn fold_binary(lhs: &Lit, rhs: &Lit, op: BinaryOp) -> Option<Lit> {
    match op {
        BinaryOp::Add => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => a.checked_add(*b).map(Lit::Int),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Float(*a as f64 + b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Float(a + *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Float(a + b)),
            (Lit::Str(a), Lit::Str(b)) => Some(Lit::Str(a.clone() + b)),
            _ => None,
        },
        BinaryOp::Sub => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => a.checked_sub(*b).map(Lit::Int),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Float(*a as f64 - b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Float(a - *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Float(a - b)),
            _ => None,
        },
        BinaryOp::Mul => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => a.checked_mul(*b).map(Lit::Int),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Float(*a as f64 * b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Float(a * *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Float(a * b)),
            _ => None,
        },
        BinaryOp::Div => match (lhs, rhs) {
            (_, Lit::Int(0)) => None, // let runtime fire the division-by-zero error
            (Lit::Int(a), Lit::Int(b)) => a.checked_div(*b).map(Lit::Int),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Float(*a as f64 / b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Float(a / *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Float(a / b)),
            _ => None,
        },
        BinaryOp::Mod => match (lhs, rhs) {
            (_, Lit::Int(0)) => None, // let runtime fire the modulo-by-zero error
            (Lit::Int(a), Lit::Int(b)) => a.checked_rem(*b).map(Lit::Int),
            _ => None,
        },
        BinaryOp::And => match (lhs, rhs) {
            (Lit::Bool(a), Lit::Bool(b)) => Some(Lit::Bool(*a && *b)),
            _ => None,
        },
        BinaryOp::Or => match (lhs, rhs) {
            (Lit::Bool(a), Lit::Bool(b)) => Some(Lit::Bool(*a || *b)),
            _ => None,
        },
        BinaryOp::Equal => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a == b)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a == b)),
            (Lit::Bool(a), Lit::Bool(b)) => Some(Lit::Bool(a == b)),
            (Lit::Str(a), Lit::Str(b)) => Some(Lit::Bool(a == b)),
            _ => None,
        },
        BinaryOp::NotEqual => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a != b)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a != b)),
            (Lit::Bool(a), Lit::Bool(b)) => Some(Lit::Bool(a != b)),
            (Lit::Str(a), Lit::Str(b)) => Some(Lit::Bool(a != b)),
            _ => None,
        },
        BinaryOp::Greater => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a > b)),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Bool((*a as f64) > *b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Bool(*a > *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a > b)),
            _ => None,
        },
        BinaryOp::Less => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a < b)),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Bool((*a as f64) < *b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Bool(*a < *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a < b)),
            _ => None,
        },
        BinaryOp::GreaterEqual => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a >= b)),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Bool((*a as f64) >= *b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Bool(*a >= *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a >= b)),
            _ => None,
        },
        BinaryOp::LessEqual => match (lhs, rhs) {
            (Lit::Int(a), Lit::Int(b)) => Some(Lit::Bool(a <= b)),
            (Lit::Int(a), Lit::Float(b)) => Some(Lit::Bool((*a as f64) <= *b)),
            (Lit::Float(a), Lit::Int(b)) => Some(Lit::Bool(*a <= *b as f64)),
            (Lit::Float(a), Lit::Float(b)) => Some(Lit::Bool(a <= b)),
            _ => None,
        },
        BinaryOp::In => None,
    }
}

fn fold_unary(val: &Lit, op: UnaryOp) -> Option<Lit> {
    match op {
        UnaryOp::Neg => match val {
            Lit::Int(i) => i.checked_neg().map(Lit::Int),
            Lit::Float(f) => Some(Lit::Float(-f)),
            _ => None,
        },
        UnaryOp::Not => match val {
            Lit::Bool(b) => Some(Lit::Bool(!b)),
            _ => None,
        },
    }
}

struct ConstantFolder;

impl Visitor for ConstantFolder {
    fn post_expr(&mut self, expr: NotPythonExpr) -> NotPythonExpr {
        let folded = match &expr {
            NotPythonExpr::UnaryOp(op, v) => expr_to_lit(v).and_then(|l| fold_unary(&l, *op)),
            NotPythonExpr::BinaryOp(op, l, r) => expr_to_lit(l)
                .zip(expr_to_lit(r))
                .and_then(|(ll, rl)| fold_binary(&ll, &rl, *op)),
            _ => None,
        };
        folded.map(lit_to_expr).unwrap_or(expr)
    }
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
            parsed_decl_expr("x := 9*9*9*9*9*9;\n"),
            NotPythonExpr::Int(531441)
        );
    }

    #[test]
    fn test_mixed_arithmetic_folds() {
        assert_eq!(
            parsed_decl_expr("x := 3 + 4 * 2;\n"),
            NotPythonExpr::Int(11)
        );
    }

    #[test]
    fn test_div_by_zero_not_folded() {
        // Should remain a BinaryOp node, not panic at parse time
        let expr = parsed_decl_expr("x := 1 / 0;\n");
        assert!(matches!(expr, NotPythonExpr::BinaryOp(..)));
    }

    #[test]
    fn test_bool_fold() {
        assert_eq!(
            parsed_decl_expr("x := True and False;\n"),
            NotPythonExpr::Boolean(false)
        );
        assert_eq!(
            parsed_decl_expr("x := not True;\n"),
            NotPythonExpr::Boolean(false)
        );
    }

    #[test]
    fn test_comparison_fold() {
        assert_eq!(
            parsed_decl_expr("x := 3 < 5;\n"),
            NotPythonExpr::Boolean(true)
        );
        assert_eq!(
            parsed_decl_expr("x := 10 == 10;\n"),
            NotPythonExpr::Boolean(true)
        );
    }

    #[test]
    fn test_identifier_blocks_fold() {
        // i - 1 cannot be folded because i is an identifier
        let expr = parsed_decl_expr("x := i - 1;\n");
        assert!(matches!(expr, NotPythonExpr::BinaryOp(..)));
    }
}
