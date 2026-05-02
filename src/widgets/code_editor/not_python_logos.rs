use crate::widgets::code_editor::code_logos::LogosCodeLanguage;
use language::NotPythonLangToken;
use ratatui_core::style::Style;
use std::collections::HashMap;

/// Creates a [`LogosCodeLanguage`] for NotPython, using `theme` for syntax highlighting colours.
pub fn not_python_language<'a>(
    theme: HashMap<std::mem::Discriminant<NotPythonLangToken>, Style>,
) -> LogosCodeLanguage<'a, NotPythonLangToken> {
    LogosCodeLanguage::new("  ", "#", theme)
}

/// Returns the default colour theme for NotPython syntax highlighting.
pub fn not_python_default_theme() -> HashMap<std::mem::Discriminant<NotPythonLangToken>, Style> {
    use crate::widgets::code_editor::utils::rgb;
    use ratatui_core::style::Color;

    let colors: [(NotPythonLangToken, &str); _] = [
        // Comments
        (NotPythonLangToken::Comment, "#546e7a"),
        // Literals (note: String payload must match at lookup time)
        (NotPythonLangToken::Int(String::new()), "#f78c6c"),
        (NotPythonLangToken::Float(String::new()), "#f78c6c"),
        // Strings
        (NotPythonLangToken::StringLiteral(String::new()), "#c3e88d"),
        // Identifiers
        (NotPythonLangToken::Identifier(String::new()), "#82aaff"),
        // Constants
        (NotPythonLangToken::KwTrue, "#ff9cac"),
        (NotPythonLangToken::KwFalse, "#ff9cac"),
        (NotPythonLangToken::KwNone, "#ff9cac"),
        // Keywords
        (NotPythonLangToken::KwAnd, "#c792ea"),
        (NotPythonLangToken::KwBreak, "#c792ea"),
        (NotPythonLangToken::KwContinue, "#c792ea"),
        (NotPythonLangToken::KwDef, "#c792ea"),
        (NotPythonLangToken::KwElif, "#c792ea"),
        (NotPythonLangToken::KwElse, "#c792ea"),
        (NotPythonLangToken::KwIf, "#c792ea"),
        (NotPythonLangToken::KwIn, "#c792ea"),
        (NotPythonLangToken::KwNot, "#c792ea"),
        (NotPythonLangToken::KwOr, "#c792ea"),
        (NotPythonLangToken::KwPass, "#c792ea"),
        (NotPythonLangToken::KwReturn, "#c792ea"),
        (NotPythonLangToken::KwLoop, "#c792ea"),
        (NotPythonLangToken::KwEnd, "#c792ea"),
        // Operators
        (NotPythonLangToken::EqEqual, "#89ddff"),
        (NotPythonLangToken::NotEqual, "#89ddff"),
        (NotPythonLangToken::LessEqual, "#89ddff"),
        (NotPythonLangToken::GreaterEqual, "#89ddff"),
        (NotPythonLangToken::ColonEqual, "#89ddff"),
        (NotPythonLangToken::Less, "#89ddff"),
        (NotPythonLangToken::Greater, "#89ddff"),
        (NotPythonLangToken::Plus, "#89ddff"),
        (NotPythonLangToken::Minus, "#89ddff"),
        (NotPythonLangToken::Star, "#89ddff"),
        (NotPythonLangToken::Slash, "#89ddff"),
        (NotPythonLangToken::Percent, "#89ddff"),
        (NotPythonLangToken::Equal, "#89ddff"),
    ];

    colors
        .into_iter()
        .map(|(token, hex)| {
            let (r, g, b) = rgb(hex);
            (
                std::mem::discriminant(&token),
                Style::default().fg(Color::Rgb(r, g, b)),
            )
        })
        .collect()
}
