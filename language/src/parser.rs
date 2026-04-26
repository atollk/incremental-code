use crate::lexer::NotPythonLangToken;
use chumsky::{
    input::{Stream, ValueInput},
    prelude::*,
};
use log::debug;
use logos::Logos;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum NotPythonExprOp {
    // Arithmetic
    Add(Box<NotPythonExpr>, Box<NotPythonExpr>),
    Sub(Box<NotPythonExpr>, Box<NotPythonExpr>),
    Mul(Box<NotPythonExpr>, Box<NotPythonExpr>),
    Div(Box<NotPythonExpr>, Box<NotPythonExpr>),
    Mod(Box<NotPythonExpr>, Box<NotPythonExpr>),
    Neg(Box<NotPythonExpr>),
    // Boolean
    And(Box<NotPythonExpr>, Box<NotPythonExpr>),
    Or(Box<NotPythonExpr>, Box<NotPythonExpr>),
    Not(Box<NotPythonExpr>),
    // Comparison
    Equal(Box<NotPythonExpr>, Box<NotPythonExpr>),
    NotEqual(Box<NotPythonExpr>, Box<NotPythonExpr>),
    Greater(Box<NotPythonExpr>, Box<NotPythonExpr>),
    Less(Box<NotPythonExpr>, Box<NotPythonExpr>),
    GreaterEqual(Box<NotPythonExpr>, Box<NotPythonExpr>),
    LessEqual(Box<NotPythonExpr>, Box<NotPythonExpr>),
    // Other
    In(Box<NotPythonExpr>, Box<NotPythonExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum NotPythonExpr {
    // Atoms
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    None,
    Identifier(String),

    // Non-atoms
    List(Vec<NotPythonExpr>),
    Dict(Vec<(NotPythonExpr, NotPythonExpr)>),
    Op(NotPythonExprOp),
    Call(Box<NotPythonExpr>, Vec<NotPythonExpr>),
    Index(Box<NotPythonExpr>, Box<NotPythonExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum NotPythonStmt {
    // Basic
    Call(String, Vec<NotPythonExpr>),
    Pass,
    Block(Vec<NotPythonStmt>),

    // Variables
    Decl(String, NotPythonExpr),
    Assign(String, NotPythonExpr),

    // Control
    If {
        condition: NotPythonExpr,
        then: Box<NotPythonStmt>,
        else_: Option<Box<NotPythonStmt>>,
    },
    Loop(Box<NotPythonStmt>),
    Return(Option<NotPythonExpr>),
    Break,
    Continue,

    // Function
    Function {
        name: String,
        params: Vec<String>,
        body: Box<NotPythonStmt>,
    },
}

type Extra<'tok> = extra::Err<Rich<'tok, NotPythonLangToken>>;

fn wrap_unary_op<'tok, I>(
    wrapped_parser: impl Parser<'tok, I, NotPythonExpr, Extra<'tok>> + Clone,
    op_parser: impl Parser<'tok, I, fn(Box<NotPythonExpr>) -> NotPythonExprOp, Extra<'tok>> + Clone,
) -> impl Parser<'tok, I, NotPythonExpr, Extra<'tok>> + Clone
where
    I: ValueInput<'tok, Token = NotPythonLangToken, Span = SimpleSpan>,
{
    op_parser.repeated().foldr(wrapped_parser, |op, rhs| {
        NotPythonExpr::Op(op(Box::new(rhs)))
    })
}

fn wrap_binary_op<'tok, I>(
    wrapped_parser: impl Parser<'tok, I, NotPythonExpr, Extra<'tok>> + Clone,
    op_parser: impl Parser<
        'tok,
        I,
        fn(Box<NotPythonExpr>, Box<NotPythonExpr>) -> NotPythonExprOp,
        Extra<'tok>,
    > + Clone,
) -> impl Parser<'tok, I, NotPythonExpr, Extra<'tok>> + Clone
where
    I: ValueInput<'tok, Token = NotPythonLangToken, Span = SimpleSpan>,
{
    wrapped_parser.clone().foldl(
        op_parser.then(wrapped_parser).repeated(),
        |lhs, (op, rhs)| NotPythonExpr::Op(op(Box::new(lhs), Box::new(rhs))),
    )
}

fn parser<'tok, 'src: 'tok, I>()
-> impl Parser<'tok, I, NotPythonStmt, extra::Err<Rich<'tok, NotPythonLangToken>>>
where
    I: ValueInput<'tok, Token = NotPythonLangToken, Span = SimpleSpan>,
{
    let expression = recursive(|expr| {
        let atom = select! {
            NotPythonLangToken::Int(s) => NotPythonExpr::Int(s.parse::<i64>().unwrap()),
            NotPythonLangToken::Float(s) => NotPythonExpr::Float(s.parse::<f64>().unwrap()),
            NotPythonLangToken::StringLiteral(s) => NotPythonExpr::String(s),
            NotPythonLangToken::KwTrue => NotPythonExpr::Boolean(true),
            NotPythonLangToken::KwFalse => NotPythonExpr::Boolean(false),
            NotPythonLangToken::KwNone => NotPythonExpr::None,
            NotPythonLangToken::Identifier(s) => NotPythonExpr::Identifier(s),
        }
        .boxed();
        let list = expr
            .clone()
            .separated_by(just(NotPythonLangToken::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(
                just(NotPythonLangToken::LBracket),
                just(NotPythonLangToken::RBracket),
            )
            .map(NotPythonExpr::List)
            .boxed();
        let dict = expr
            .clone()
            .then_ignore(just(NotPythonLangToken::Colon))
            .then(expr.clone())
            .separated_by(just(NotPythonLangToken::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(
                just(NotPythonLangToken::LBrace),
                just(NotPythonLangToken::RBrace),
            )
            .map(NotPythonExpr::Dict)
            .boxed();
        let call = select! { NotPythonLangToken::Identifier(s) => s }
            .then(
                expr.clone()
                    .separated_by(just(NotPythonLangToken::Comma))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(
                        just(NotPythonLangToken::LParen),
                        just(NotPythonLangToken::RParen),
                    ),
            )
            .map(|(name, args)| {
                NotPythonExpr::Call(Box::new(NotPythonExpr::Identifier(name)), args)
            })
            .boxed();
        let index = select! { NotPythonLangToken::Identifier(s) => s }
            .then(expr.clone().delimited_by(
                just(NotPythonLangToken::LBracket),
                just(NotPythonLangToken::RBracket),
            ))
            .map(|(name, idx)| {
                NotPythonExpr::Index(Box::new(NotPythonExpr::Identifier(name)), Box::new(idx))
            })
            .boxed();
        let base = choice((call, index, list, dict, atom));

        {
            let arith_op = {
                let neg = wrap_unary_op(
                    base,
                    just(NotPythonLangToken::Minus).to(NotPythonExprOp::Neg as fn(_) -> _),
                );
                let mul_div = wrap_binary_op(
                    neg,
                    choice((
                        just(NotPythonLangToken::Star).to(NotPythonExprOp::Mul as fn(_, _) -> _),
                        just(NotPythonLangToken::Slash).to(NotPythonExprOp::Div as fn(_, _) -> _),
                    )),
                );
                let add_sub = wrap_binary_op(
                    mul_div,
                    choice((
                        just(NotPythonLangToken::Plus).to(NotPythonExprOp::Add as fn(_, _) -> _),
                        just(NotPythonLangToken::Minus).to(NotPythonExprOp::Sub as fn(_, _) -> _),
                    )),
                );

                wrap_binary_op(
                    add_sub,
                    just(NotPythonLangToken::Percent).to(NotPythonExprOp::Mod as fn(_, _) -> _),
                )
            };
            let bool_op = wrap_binary_op(
                arith_op,
                choice((
                    just(NotPythonLangToken::EqEqual).to(NotPythonExprOp::Equal as fn(_, _) -> _),
                    just(NotPythonLangToken::NotEqual)
                        .to(NotPythonExprOp::NotEqual as fn(_, _) -> _),
                    just(NotPythonLangToken::Less).to(NotPythonExprOp::Less as fn(_, _) -> _),
                    just(NotPythonLangToken::Greater).to(NotPythonExprOp::Greater as fn(_, _) -> _),
                    just(NotPythonLangToken::LessEqual)
                        .to(NotPythonExprOp::LessEqual as fn(_, _) -> _),
                    just(NotPythonLangToken::GreaterEqual)
                        .to(NotPythonExprOp::GreaterEqual as fn(_, _) -> _),
                )),
            );
            let in_op = wrap_binary_op(
                bool_op,
                just(NotPythonLangToken::KwIn).to(NotPythonExprOp::In as fn(_, _) -> _),
            );
            let not_op = wrap_unary_op(
                in_op,
                just(NotPythonLangToken::KwNot).to(NotPythonExprOp::Not as fn(_) -> _),
            );
            let and_op = wrap_binary_op(
                not_op,
                just(NotPythonLangToken::KwAnd).to(NotPythonExprOp::And as fn(_, _) -> _),
            );
            wrap_binary_op(
                and_op,
                just(NotPythonLangToken::KwOr).to(NotPythonExprOp::Or as fn(_, _) -> _),
            )
            .boxed()
        }
    })
    .boxed();
    let statement = recursive(|stmt| {
        // Each simple statement ends with `;\n` (or `;<EOI>`).
        // Each block statement ends with `end\n` (or `end<EOI>`).
        // Block headers (if/elif/else/loop/def) end with `:\n`.
        let line_end = just(NotPythonLangToken::Newline)
            .ignored()
            .or(end())
            .boxed();

        let semi_line_end = just(NotPythonLangToken::Semi)
            .then(line_end.clone())
            .ignored()
            .boxed();

        // pass;
        let pass = just(NotPythonLangToken::KwPass)
            .then_ignore(semi_line_end.clone())
            .to(NotPythonStmt::Pass)
            .boxed();

        // break;
        let break_ = just(NotPythonLangToken::KwBreak)
            .then_ignore(semi_line_end.clone())
            .to(NotPythonStmt::Break)
            .boxed();

        // continue;
        let continue_ = just(NotPythonLangToken::KwContinue)
            .then_ignore(semi_line_end.clone())
            .to(NotPythonStmt::Continue)
            .boxed();

        // return <expr>;
        let return_ = just(NotPythonLangToken::KwReturn)
            .ignore_then(expression.clone().or_not())
            .then_ignore(semi_line_end.clone())
            .map(NotPythonStmt::Return)
            .boxed();

        // name := expr;
        let decl = select! { NotPythonLangToken::Identifier(s) => s }
            .then_ignore(just(NotPythonLangToken::ColonEqual))
            .then(expression.clone())
            .then_ignore(semi_line_end.clone())
            .map(|(name, expr)| NotPythonStmt::Decl(name, expr))
            .boxed();

        // name = expr;
        let assign = select! { NotPythonLangToken::Identifier(s) => s }
            .then_ignore(just(NotPythonLangToken::Equal))
            .then(expression.clone())
            .then_ignore(semi_line_end.clone())
            .map(|(name, expr)| NotPythonStmt::Assign(name, expr))
            .boxed();

        // name(args...);
        let call_stmt = select! { NotPythonLangToken::Identifier(s) => s }
            .then(
                expression
                    .clone()
                    .separated_by(just(NotPythonLangToken::Comma))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(
                        just(NotPythonLangToken::LParen),
                        just(NotPythonLangToken::RParen),
                    ),
            )
            .then_ignore(semi_line_end)
            .map(|(name, args)| NotPythonStmt::Call(name, args))
            .boxed();

        // A block is a bare sequence of statements. Each statement carries its own
        // terminator, so no extra separator is needed. The block ends when the next
        // token can't start a statement (e.g. `end`, `elif`, `else`).
        let block = stmt
            .clone()
            .repeated()
            .collect::<Vec<_>>()
            .map(NotPythonStmt::Block)
            .boxed();

        // `:\n` header terminator shared by if/elif/else/loop/def.
        let colon_newline = just(NotPythonLangToken::Colon)
            .then(just(NotPythonLangToken::Newline))
            .ignored()
            .boxed();

        // if expr:\n block [elif expr:\n block]* [else:\n block] end\n
        let elif_branch = just(NotPythonLangToken::KwElif)
            .ignore_then(expression.clone())
            .then_ignore(colon_newline.clone())
            .then(block.clone())
            .boxed();

        let else_branch = just(NotPythonLangToken::KwElse)
            .ignore_then(colon_newline.clone())
            .ignore_then(block.clone())
            .boxed();

        let if_stmt = just(NotPythonLangToken::KwIf)
            .ignore_then(expression.clone())
            .then_ignore(colon_newline.clone())
            .then(block.clone())
            .then(elif_branch.repeated().collect::<Vec<_>>())
            .then(else_branch.or_not())
            .then_ignore(just(NotPythonLangToken::KwEnd))
            .then_ignore(line_end.clone())
            .map(|(((cond, then_block), elifs), else_block)| {
                // Build nested Ifs from the elif chain, right to left.
                let mut else_ = else_block.map(Box::new);
                for (elif_cond, elif_block) in elifs.into_iter().rev() {
                    else_ = Some(Box::new(NotPythonStmt::If {
                        condition: elif_cond,
                        then: Box::new(elif_block),
                        else_,
                    }));
                }
                NotPythonStmt::If {
                    condition: cond,
                    then: Box::new(then_block),
                    else_,
                }
            })
            .boxed();

        // loop:\n block end\n
        let loop_stmt = just(NotPythonLangToken::KwLoop)
            .ignore_then(colon_newline.clone())
            .ignore_then(block.clone())
            .then_ignore(just(NotPythonLangToken::KwEnd))
            .then_ignore(line_end.clone())
            .map(|body| NotPythonStmt::Loop(Box::new(body)))
            .boxed();

        // def name(params):\n block end\n
        let func_params = select! { NotPythonLangToken::Identifier(s) => s }
            .separated_by(just(NotPythonLangToken::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(
                just(NotPythonLangToken::LParen),
                just(NotPythonLangToken::RParen),
            )
            .boxed();

        let func_stmt = just(NotPythonLangToken::KwDef)
            .ignore_then(select! { NotPythonLangToken::Identifier(s) => s })
            .then(func_params)
            .then_ignore(colon_newline)
            .then(block)
            .then_ignore(just(NotPythonLangToken::KwEnd))
            .then_ignore(line_end)
            .map(|((name, params), body)| NotPythonStmt::Function {
                name,
                params,
                body: Box::new(body),
            })
            .boxed();

        choice((
            pass, break_, continue_, return_, if_stmt, loop_stmt, func_stmt, decl, assign,
            call_stmt,
        ))
        .boxed()
    });

    // Top-level: each statement carries its own terminator, so no padding needed.
    statement
        .repeated()
        .collect::<Vec<_>>()
        .map(NotPythonStmt::Block)
        .boxed()
}

fn parse<'a>(src: &'a str) -> Result<NotPythonStmt, Vec<Rich<'a, NotPythonLangToken>>> {
    let tokens: Vec<_> = NotPythonLangToken::lexer(src)
        .spanned()
        // Convert logos errors into tokens. We want parsing to be recoverable and not fail at the lexing stage, so
        // we have a dedicated `Token::Error` variant that represents a token error that was previously encountered
        .map(|(tok, span)| match tok {
            // Turn the `Range<usize>` spans logos gives us into chumsky's `SimpleSpan` via `Into`, because it's easier
            // to work with
            Ok(tok) => (tok, span.into()),
            Err(()) => (NotPythonLangToken::LexError, span.into()),
        })
        .collect();
    // Turn the token iterator into a stream that chumsky can use for things like backtracking
    let token_stream = Stream::from_iter(tokens)
        // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
        // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
        .map((0..src.len()).into(), |(t, s): (_, _)| (t, s));
    parser().parse(token_stream).into_result()
}

pub struct NotPythonProgram {
    pub(super) statement: NotPythonStmt,
}

pub fn parse_program<'a>(
    src: &'a str,
) -> anyhow::Result<NotPythonProgram, Vec<Rich<'a, NotPythonLangToken>>> {
    parse(src)
        .map(|ast| NotPythonProgram { statement: ast })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unwrap the top-level Block and return its statements.
    fn stmts(src: &str) -> Vec<NotPythonStmt> {
        let result = parse(src).unwrap_or_else(|e| panic!("parse failed: {e:?}"));
        let NotPythonStmt::Block(stmts) = result else {
            panic!("expected top-level Block");
        };
        stmts
    }

    // -------------------------------------------------------------------------
    // Simple statements
    // -------------------------------------------------------------------------

    #[test]
    fn pass_break_continue() {
        assert_eq!(stmts("pass;\n"), vec![NotPythonStmt::Pass]);
        assert_eq!(stmts("break;\n"), vec![NotPythonStmt::Break]);
        assert_eq!(stmts("continue;\n"), vec![NotPythonStmt::Continue]);
    }

    #[test]
    fn eoi_accepted_as_line_terminator() {
        // No trailing newline — EOI satisfies the line_end rule.
        assert_eq!(stmts("pass;"), vec![NotPythonStmt::Pass]);
    }

    #[test]
    fn multiple_stmts() {
        assert_eq!(
            stmts("pass;\nbreak;\ncontinue;\n"),
            vec![
                NotPythonStmt::Pass,
                NotPythonStmt::Break,
                NotPythonStmt::Continue
            ]
        );
    }

    #[test]
    fn decl() {
        assert_eq!(
            stmts("x := 42;\n"),
            vec![NotPythonStmt::Decl("x".into(), NotPythonExpr::Int(42))]
        );
    }

    #[test]
    fn assign() {
        assert_eq!(
            stmts("x = True;\n"),
            vec![NotPythonStmt::Assign(
                "x".into(),
                NotPythonExpr::Boolean(true)
            )]
        );
    }

    #[test]
    fn call_no_args() {
        assert_eq!(
            stmts("foo();\n"),
            vec![NotPythonStmt::Call("foo".into(), vec![])]
        );
    }

    #[test]
    fn call_with_args() {
        assert_eq!(
            stmts("foo(1, 2);\n"),
            vec![NotPythonStmt::Call(
                "foo".into(),
                vec![NotPythonExpr::Int(1), NotPythonExpr::Int(2)]
            )]
        );
    }

    // -------------------------------------------------------------------------
    // Expressions
    // -------------------------------------------------------------------------

    #[test]
    fn arithmetic_precedence() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3) due to precedence
        assert_eq!(
            stmts("x := 1 + 2 * 3;\n"),
            vec![NotPythonStmt::Decl(
                "x".into(),
                NotPythonExpr::Op(NotPythonExprOp::Add(
                    Box::new(NotPythonExpr::Int(1)),
                    Box::new(NotPythonExpr::Op(NotPythonExprOp::Mul(
                        Box::new(NotPythonExpr::Int(2)),
                        Box::new(NotPythonExpr::Int(3)),
                    ))),
                ))
            )]
        );
    }

    #[test]
    fn negation() {
        assert_eq!(
            stmts("x := -1;\n"),
            vec![NotPythonStmt::Decl(
                "x".into(),
                NotPythonExpr::Op(NotPythonExprOp::Neg(Box::new(NotPythonExpr::Int(1))))
            )]
        );
    }

    #[test]
    fn comparison_expr() {
        assert_eq!(
            stmts("x := 1 == 2;\n"),
            vec![NotPythonStmt::Decl(
                "x".into(),
                NotPythonExpr::Op(NotPythonExprOp::Equal(
                    Box::new(NotPythonExpr::Int(1)),
                    Box::new(NotPythonExpr::Int(2)),
                ))
            )]
        );
    }

    // -------------------------------------------------------------------------
    // Block statements
    // -------------------------------------------------------------------------

    #[test]
    fn if_no_else() {
        assert_eq!(
            stmts("if True:\npass;\nend\n"),
            vec![NotPythonStmt::If {
                condition: NotPythonExpr::Boolean(true),
                then: Box::new(NotPythonStmt::Block(vec![NotPythonStmt::Pass])),
                else_: None,
            }]
        );
    }

    #[test]
    fn if_else() {
        assert_eq!(
            stmts("if True:\npass;\nelse:\nbreak;\nend\n"),
            vec![NotPythonStmt::If {
                condition: NotPythonExpr::Boolean(true),
                then: Box::new(NotPythonStmt::Block(vec![NotPythonStmt::Pass])),
                else_: Some(Box::new(NotPythonStmt::Block(vec![NotPythonStmt::Break]))),
            }]
        );
    }

    #[test]
    fn if_elif_else() {
        // elif desugars into a nested If in the else branch.
        assert_eq!(
            stmts("if True:\npass;\nelif False:\nbreak;\nelse:\ncontinue;\nend\n"),
            vec![NotPythonStmt::If {
                condition: NotPythonExpr::Boolean(true),
                then: Box::new(NotPythonStmt::Block(vec![NotPythonStmt::Pass])),
                else_: Some(Box::new(NotPythonStmt::If {
                    condition: NotPythonExpr::Boolean(false),
                    then: Box::new(NotPythonStmt::Block(vec![NotPythonStmt::Break])),
                    else_: Some(Box::new(NotPythonStmt::Block(vec![
                        NotPythonStmt::Continue
                    ]))),
                })),
            }]
        );
    }

    #[test]
    fn loop_stmt() {
        assert_eq!(
            stmts("loop:\nbreak;\nend\n"),
            vec![NotPythonStmt::Loop(Box::new(NotPythonStmt::Block(vec![
                NotPythonStmt::Break
            ])))]
        );
    }

    #[test]
    fn func_def_no_params() {
        assert_eq!(
            stmts("def foo():\npass;\nend\n"),
            vec![NotPythonStmt::Function {
                name: "foo".into(),
                params: vec![],
                body: Box::new(NotPythonStmt::Block(vec![NotPythonStmt::Pass])),
            }]
        );
    }

    #[test]
    fn func_def_with_params() {
        assert_eq!(
            stmts("def add(x, y):\npass;\nend\n"),
            vec![NotPythonStmt::Function {
                name: "add".into(),
                params: vec!["x".into(), "y".into()],
                body: Box::new(NotPythonStmt::Block(vec![NotPythonStmt::Pass])),
            }]
        );
    }

    #[test]
    fn nested_blocks() {
        assert_eq!(
            stmts("loop:\nif True:\nbreak;\nend\nend\n"),
            vec![NotPythonStmt::Loop(Box::new(NotPythonStmt::Block(vec![
                NotPythonStmt::If {
                    condition: NotPythonExpr::Boolean(true),
                    then: Box::new(NotPythonStmt::Block(vec![NotPythonStmt::Break])),
                    else_: None,
                }
            ])))]
        );
    }

    // -------------------------------------------------------------------------
    // Error cases
    // -------------------------------------------------------------------------

    #[test]
    fn missing_semicolon_is_error() {
        assert!(parse("pass\n").is_err());
    }

    #[test]
    fn missing_end_is_error() {
        assert!(parse("loop:\npass;\n").is_err());
    }

    #[test]
    fn non_newline_after_end_is_error() {
        assert!(parse("loop:\npass;\nend pass;\n").is_err());
    }

    #[test]
    fn non_newline_after_block_header_is_error() {
        assert!(parse("loop: pass;\nend\n").is_err());
    }
}
