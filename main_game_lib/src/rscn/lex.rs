//! Lexes .tscn files.
//!
//! We don't support the full format, only the parts that are relevant to our
//! game.

use std::ops::Range;

use logos::Logos;
use miette::LabeledSpan;

#[derive(Debug, Clone)]
pub(crate) struct TscnToken {
    pub(crate) kind: TscnTokenKind,
    pub(crate) span: Range<usize>,
}

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(skip r"[\r\t\f]+")]
pub(crate) enum TscnTokenKind {
    #[token("[")]
    SquareBracketOpen,
    #[token("]")]
    SquareBracketClose,
    #[token("(")]
    ParenOpen,
    #[token(")")]
    ParenClose,
    #[token("{")]
    CurlyBracketOpen,
    #[token("}")]
    CurlyBracketClose,
    #[token(":")]
    Colon,
    #[token("&")]
    Ampersand,
    #[token("=")]
    Equal,
    #[token(",")]
    Comma,
    #[token("/")]
    ForwardSlash,
    #[token(" ")]
    Space,
    #[token("\n")]
    NewLine,

    #[regex(r#"[A-Za-z0-9_/]+"#, priority = 3)]
    Identifier,

    #[token("true")]
    True,
    #[token("false")]
    False,
    #[regex(r#"-?\d+(\.\d+)?"#, priority = 4)]
    Number,
    #[regex(r#""[A-Za-z0-9_/?:\. ]+""#, priority = 2)]
    String,
}

pub(crate) fn lex(tscn: &str) -> miette::Result<Vec<TscnToken>> {
    TscnTokenKind::lexer(tscn)
        .spanned()
        .map(|(try_token, span)| {
            try_token
                .map(|kind| TscnToken {
                    kind,
                    span: span.clone(),
                })
                .map_err(|_| {
                    miette::miette! {
                        labels = vec![
                            LabeledSpan::at(span.clone(), "this input"),
                        ],
                        "Unexpected input {}", &tscn[span.clone()],
                    }
                    .with_source_code(tscn.to_string())
                })
        })
        .collect()
}

impl std::fmt::Display for TscnTokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TscnTokenKind::SquareBracketOpen => write!(f, "["),
            TscnTokenKind::SquareBracketClose => write!(f, "]"),
            TscnTokenKind::ParenOpen => write!(f, "("),
            TscnTokenKind::ParenClose => write!(f, ")"),
            TscnTokenKind::CurlyBracketOpen => write!(f, "{{"),
            TscnTokenKind::CurlyBracketClose => write!(f, "}}"),
            TscnTokenKind::Colon => write!(f, ":"),
            TscnTokenKind::Ampersand => write!(f, "&"),
            TscnTokenKind::Equal => write!(f, "="),
            TscnTokenKind::Space => write!(f, "space"),
            TscnTokenKind::NewLine => write!(f, "new line"),
            TscnTokenKind::Identifier => write!(f, "identifier"),
            TscnTokenKind::True => write!(f, "true"),
            TscnTokenKind::False => write!(f, "false"),
            TscnTokenKind::Number => write!(f, "number"),
            TscnTokenKind::String => write!(f, "string"),
            TscnTokenKind::ForwardSlash => write!(f, "/"),
            TscnTokenKind::Comma => write!(f, ","),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::lex;

    #[test]
    fn it_lexes_tscn() -> miette::Result<()> {
        let workspace_root =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect(
                "Failed to get CARGO_MANIFEST_DIR environment variable",
            ));
        let scenes_dir = if workspace_root.ends_with("main_game_lib") {
            // we are running this test from the main_game_lib directory
            format!("{}/../main_game/assets/scenes", workspace_root.display())
        } else {
            // we are running this test from the workspace root
            format!("{}/main_game/assets/scenes", workspace_root.display())
        };
        let dir_iter = std::fs::read_dir(&scenes_dir).unwrap_or_else(|err| {
            panic!("Failed to read directory '{scenes_dir}' with .tscn files: {err}");
        });

        for entry in dir_iter {
            let entry = entry
                .expect("Failed to read entry in directory with .tscn files");
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "tscn") {
                continue;
            }

            let tscn = std::fs::read_to_string(&path).unwrap_or_else(|err| {
                panic!("Failed to read .tscn file at {path:?}: {err}");
            });

            let tokens = lex(&tscn)?;
            assert!(!tokens.is_empty(), "Empty .tscn file at {path:?}");
        }

        Ok(())
    }
}
