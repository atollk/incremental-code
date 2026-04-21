use crate::widgets::code_editor::code_logos::LogosCodeLanguage;
use logos::Logos;
use ratatui_core::style::Style;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

pub fn python_language<'a>(
    theme: HashMap<NotPythonLangToken, Style>,
) -> LogosCodeLanguage<'a, NotPythonLangToken> {
    LogosCodeLanguage::new("  ", "#", theme)
}

#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\f]+")] // skip horizontal whitespace only (newlines matter)
pub enum NotPythonLangToken {
    LexError,

    // -------------------------------------------------------------------------
    // Comments
    // -------------------------------------------------------------------------
    /// A `#`-comment running to end of line.
    #[regex(r"#[^\r\n]*", allow_greedy = true)]
    Comment,

    // -------------------------------------------------------------------------
    // Literals
    // -------------------------------------------------------------------------
    /// Integer literals (with underscores allowed)
    #[regex(r"0|[1-9][0-9_]*", |lex| lex.slice().to_string())]
    Int(String),

    /// Floating-point literals
    #[regex(r"([0-9][0-9_]*)?\.[0-9][0-9_]*", |lex| lex.slice().to_string())]
    Float(String),

    // -------------------------------------------------------------------------
    // String literals
    // -------------------------------------------------------------------------
    /// Double-quoted string
    #[regex(r#""([^"\\]|\\.)*""#, |lex| { let s = lex.slice(); s[1..s.len()-1].to_string() })]
    StringLiteral(String),

    // -------------------------------------------------------------------------
    // Keywords
    // `#[token]` has higher priority than `#[regex]`, so keywords always win
    // over the Identifier regex below — no need for boundary assertions.
    // -------------------------------------------------------------------------
    #[token("False")]
    KwFalse,
    #[token("None")]
    KwNone,
    #[token("True")]
    KwTrue,
    #[token("and")]
    KwAnd,
    #[token("break")]
    KwBreak,
    #[token("continue")]
    KwContinue,
    #[token("def")]
    KwDef,
    #[token("elif")]
    KwElif,
    #[token("else")]
    KwElse,
    #[token("if")]
    KwIf,
    #[token("in")]
    KwIn,
    #[token("not")]
    KwNot,
    #[token("or")]
    KwOr,
    #[token("pass")]
    KwPass,
    #[token("return")]
    KwReturn,
    #[token("loop")]
    KwLoop,
    #[token("end")]
    KwEnd,

    // -------------------------------------------------------------------------
    // Identifiers
    // Listed after keywords; `#[token]` priority ensures keywords win.
    // -------------------------------------------------------------------------
    /// ASCII-only fast path.
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // -------------------------------------------------------------------------
    // Operators
    // Three-character operators must be listed before their two-character
    // prefixes, which must be listed before single-character ones.
    // Logos resolves ties by longest match, but explicit ordering is clearer.
    // -------------------------------------------------------------------------
    #[token("==")]
    EqEqual,
    #[token("!=")]
    NotEqual,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,
    #[token(":=")]
    ColonEqual,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("=")]
    Equal,

    // -------------------------------------------------------------------------
    // Delimiters & punctuation
    // -------------------------------------------------------------------------
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token(";")]
    Semi,

    // -------------------------------------------------------------------------
    // Whitespace-sensitive tokens
    // -------------------------------------------------------------------------
    /// Physical newline (CR+LF, CR, or LF).
    /// Logical-newline / INDENT / DEDENT handling must be done in a
    /// post-processing pass that tracks this token plus bracket depth.
    #[regex(r"\r\n|\r|\n")]
    Newline,

    /// Explicit line continuation: `\` immediately followed by a newline.
    /// Emit so the indent pass knows to suppress the following `Newline`.
    #[regex(r"\\(\r\n|\r|\n)")]
    LineContinuation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    fn lex(src: &str) -> Vec<NotPythonLangToken> {
        NotPythonLangToken::lexer(src)
            .map(|r| r.unwrap_or(NotPythonLangToken::LexError))
            .collect()
    }

    #[test]
    fn integer_literals() {
        assert_eq!(lex("42"), vec![NotPythonLangToken::Int("42".into())]);
        assert_eq!(lex("1_000"), vec![NotPythonLangToken::Int("1_000".into())]);
    }

    #[test]
    fn float_literals() {
        assert_eq!(lex("3.14"), vec![NotPythonLangToken::Float("3.14".into())]);
        assert_eq!(lex(".5"), vec![NotPythonLangToken::Float(".5".into())]);
    }

    #[test]
    fn string_literals() {
        assert_eq!(
            lex(r#""hello""#),
            vec![NotPythonLangToken::StringLiteral("hello".into())]
        );
        // Escape sequences are preserved as-is (not interpreted by the lexer).
        assert_eq!(
            lex(r#""a\"b""#),
            vec![NotPythonLangToken::StringLiteral(r#"a\"b"#.into())]
        );
    }

    #[test]
    fn keywords_take_priority_over_identifiers() {
        assert_eq!(lex("if"), vec![NotPythonLangToken::KwIf]);
        assert_eq!(lex("else"), vec![NotPythonLangToken::KwElse]);
        assert_eq!(lex("elif"), vec![NotPythonLangToken::KwElif]);
        assert_eq!(lex("loop"), vec![NotPythonLangToken::KwLoop]);
        assert_eq!(lex("end"), vec![NotPythonLangToken::KwEnd]);
        assert_eq!(lex("def"), vec![NotPythonLangToken::KwDef]);
        assert_eq!(lex("True"), vec![NotPythonLangToken::KwTrue]);
        assert_eq!(lex("False"), vec![NotPythonLangToken::KwFalse]);
        assert_eq!(lex("None"), vec![NotPythonLangToken::KwNone]);
        // Identifiers that merely start with a keyword prefix are not keywords.
        assert_eq!(
            lex("iffy"),
            vec![NotPythonLangToken::Identifier("iffy".into())]
        );
        assert_eq!(
            lex("end_game"),
            vec![NotPythonLangToken::Identifier("end_game".into())]
        );
    }

    #[test]
    fn identifier() {
        assert_eq!(
            lex("foo_bar"),
            vec![NotPythonLangToken::Identifier("foo_bar".into())]
        );
    }

    #[test]
    fn operators() {
        assert_eq!(lex("=="), vec![NotPythonLangToken::EqEqual]);
        assert_eq!(lex("!="), vec![NotPythonLangToken::NotEqual]);
        assert_eq!(lex(":="), vec![NotPythonLangToken::ColonEqual]);
        assert_eq!(lex("<="), vec![NotPythonLangToken::LessEqual]);
        assert_eq!(lex(">="), vec![NotPythonLangToken::GreaterEqual]);
        assert_eq!(
            lex("+ - * / %"),
            vec![
                NotPythonLangToken::Plus,
                NotPythonLangToken::Minus,
                NotPythonLangToken::Star,
                NotPythonLangToken::Slash,
                NotPythonLangToken::Percent,
            ]
        );
    }

    #[test]
    fn comment_runs_to_end_of_line() {
        assert_eq!(
            lex("x # comment\n"),
            vec![
                NotPythonLangToken::Identifier("x".into()),
                NotPythonLangToken::Comment,
                NotPythonLangToken::Newline,
            ]
        );
        // Comment at EOF (no trailing newline) produces no Newline token.
        assert_eq!(lex("# eof"), vec![NotPythonLangToken::Comment]);
    }

    #[test]
    fn newlines_are_tokens() {
        assert_eq!(
            lex("a\nb"),
            vec![
                NotPythonLangToken::Identifier("a".into()),
                NotPythonLangToken::Newline,
                NotPythonLangToken::Identifier("b".into()),
            ]
        );
    }

    #[test]
    fn horizontal_whitespace_is_skipped() {
        assert_eq!(
            lex("a   b\t c"),
            vec![
                NotPythonLangToken::Identifier("a".into()),
                NotPythonLangToken::Identifier("b".into()),
                NotPythonLangToken::Identifier("c".into()),
            ]
        );
    }

    #[test]
    fn unknown_char_is_lex_error() {
        assert_eq!(lex("@"), vec![NotPythonLangToken::LexError]);
        assert_eq!(
            lex("x @ y"),
            vec![
                NotPythonLangToken::Identifier("x".into()),
                NotPythonLangToken::LexError,
                NotPythonLangToken::Identifier("y".into()),
            ]
        );
    }

    #[test]
    fn display() {
        assert_eq!(NotPythonLangToken::KwIf.to_string(), "if");
        assert_eq!(NotPythonLangToken::EqEqual.to_string(), "==");
        assert_eq!(NotPythonLangToken::ColonEqual.to_string(), ":=");
        assert_eq!(NotPythonLangToken::Int("99".into()).to_string(), "99");
        assert_eq!(NotPythonLangToken::Float("1.5".into()).to_string(), "1.5");
        assert_eq!(NotPythonLangToken::Identifier("x".into()).to_string(), "x");
        assert_eq!(
            NotPythonLangToken::StringLiteral("hi".into()).to_string(),
            "\"hi\""
        );
    }
}

impl fmt::Display for NotPythonLangToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let NotPythonLangToken::StringLiteral(s) = self {
            write!(f, "\"{s}\"")
        } else {
            let s = match self {
                NotPythonLangToken::Comment => "#",
                NotPythonLangToken::Int(s) => s,
                NotPythonLangToken::Float(s) => s,
                NotPythonLangToken::StringLiteral(_) => unreachable!(),
                NotPythonLangToken::KwFalse => "False",
                NotPythonLangToken::KwNone => "None",
                NotPythonLangToken::KwTrue => "True",
                NotPythonLangToken::KwAnd => "and",
                NotPythonLangToken::KwBreak => "break",
                NotPythonLangToken::KwContinue => "continue",
                NotPythonLangToken::KwDef => "def",
                NotPythonLangToken::KwElif => "elif",
                NotPythonLangToken::KwElse => "else",
                NotPythonLangToken::KwIf => "if",
                NotPythonLangToken::KwIn => "in",
                NotPythonLangToken::KwNot => "not",
                NotPythonLangToken::KwOr => "or",
                NotPythonLangToken::KwPass => "pass",
                NotPythonLangToken::KwReturn => "return",
                NotPythonLangToken::KwLoop => "loop",
                NotPythonLangToken::KwEnd => "end",
                NotPythonLangToken::Identifier(s) => s,
                NotPythonLangToken::EqEqual => "==",
                NotPythonLangToken::NotEqual => "!=",
                NotPythonLangToken::LessEqual => "<=",
                NotPythonLangToken::GreaterEqual => ">=",
                NotPythonLangToken::ColonEqual => ":=",
                NotPythonLangToken::Less => "<",
                NotPythonLangToken::Greater => ">",
                NotPythonLangToken::Plus => "+",
                NotPythonLangToken::Minus => "-",
                NotPythonLangToken::Star => "*",
                NotPythonLangToken::Slash => "/",
                NotPythonLangToken::Percent => "%",
                NotPythonLangToken::Equal => "=",
                NotPythonLangToken::LParen => "(",
                NotPythonLangToken::RParen => ")",
                NotPythonLangToken::LBracket => "[",
                NotPythonLangToken::RBracket => "]",
                NotPythonLangToken::LBrace => "{",
                NotPythonLangToken::RBrace => "}",
                NotPythonLangToken::Comma => ",",
                NotPythonLangToken::Colon => ":",
                NotPythonLangToken::Dot => ".",
                NotPythonLangToken::Semi => ";",
                NotPythonLangToken::Newline => "\n",
                NotPythonLangToken::LineContinuation => "\\\n",
                NotPythonLangToken::LexError => "<failure_to_lex>",
            };
            f.write_str(s)
        }
    }
}
