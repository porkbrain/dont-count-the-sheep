//! Parses a list of [TscnToken]s.

mod ext_resource;
mod node;
mod sub_resource;

use std::ops::Range;

use miette::LabeledSpan;

use super::{
    lex::{TscnToken, TscnTokenKind},
    value::*,
};
use crate::rscn::intermediate_repr::*;

struct Parser<'a, I> {
    source: &'a str,
    tokens: I,
    state: State,
    open_section: OpenSection,
    // TODO: remember position of last token to give better errors on
    // unexpected EOF
}

/// Wraps an expression that can error and adds source code to the miette error.
macro_rules! error_with_source_code {
    ($source:expr, $expr:expr) => {
        $expr.map_err(|err| err.with_source_code($source.to_owned()))?
    };
}

pub(crate) fn parse(
    tscn: &str,
    tokens: impl IntoIterator<Item = TscnToken>,
) -> miette::Result<State> {
    // TODO: forbid empty tscn

    let mut parser = Parser {
        source: tscn,
        tokens: tokens.into_iter(),
        state: State::default(),
        open_section: OpenSection::default(),
    };

    error_with_source_code!(tscn, parser.parse_headers());

    while let IsParsingDone::No =
        error_with_source_code!(tscn, parser.parse_next())
    {
        // keep it up chief
    }

    Ok(parser.state)
}

/// I love named booleans.
enum IsParsingDone {
    /// When parsing is done the parser can return its wrapped state.
    Yes,
    No,
}

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = TscnToken>,
{
    /// Each tscn file must start with a `gd_scene` heading.
    /// There must be exactly one `gd_scene` heading per tscn.
    ///
    /// Parses the attributes into [State::headers].
    fn parse_headers(&mut self) -> miette::Result<()> {
        // [gd_scene opt_attr1=... opt_attr2=... ... ]

        self.expect_exact_token(TscnTokenKind::SquareBracketOpen)?;
        self.expect_exact_token_with_content(
            TscnTokenKind::Identifier,
            tscn_identifiers::GD_SCENE,
        )?;
        // this ignores spaces
        let (_, headers) = self.expect_attributes()?;
        // happens exactly once, overwrites defaults (that did not allocate)
        self.state.headers = headers;

        Ok(())
    }

    /// Parses next chunk of tokens.
    ///
    /// This is meant to be called in a loop until it returns
    /// `Ok(IsParsingDone::Yes)` or error.
    fn parse_next(&mut self) -> miette::Result<IsParsingDone> {
        match self.tokens.next() {
            Some(TscnToken {
                kind: TscnTokenKind::SquareBracketOpen,
                span,
            }) => {
                // we are starting a new section!
                self.close_section();

                let new_section_kind = &self.source[span.clone()];
                match new_section_kind {
                    tscn_identifiers::EXT_RESOURCE => {
                        let (ends_at, ext_attrs) = self.expect_attributes()?;
                        ext_resource::parse_attributes_into_state(
                            &mut self.state,
                            span.start..ends_at,
                            ext_attrs,
                        )?;
                    }
                    tscn_identifiers::SUB_RESOURCE => {
                        let (ends_at, ext_attrs) = self.expect_attributes()?;
                        let sub_resource = sub_resource::parse_attributes(
                            &mut self.state,
                            span.start..ends_at,
                            ext_attrs,
                        )?;
                        self.open_section =
                            OpenSection::SubResource(sub_resource);
                    }
                    tscn_identifiers::NODE => {
                        let (ends_at, ext_attrs) = self.expect_attributes()?;
                        let node = node::parse_attributes(
                            &mut self.state,
                            span.start..ends_at,
                            ext_attrs,
                        )?;
                        self.open_section = OpenSection::Node(node);
                    }
                    _ => {
                        miette::bail! {
                            labels = vec![
                                LabeledSpan::at(span, "this section"),
                            ],
                            "Unknown section '{new_section_kind}'",
                        }
                    }
                }

                Ok(IsParsingDone::No)
            }
            Some(TscnToken {
                kind: TscnTokenKind::Identifier,
                span,
            }) => {
                // TODO
                Ok(IsParsingDone::No)
            }
            // ignore new lines and spaces
            Some(TscnToken {
                kind: TscnTokenKind::NewLine | TscnTokenKind::Space,
                ..
            }) => Ok(IsParsingDone::No),
            // either we have a new section starting with "[" or a key, but got
            // something else
            Some(TscnToken { kind, span }) => {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(span, "this token"),
                    ],
                    "Expected either new section or a key, got {kind}",
                }
            }
            // valid EOF as nothing is left open
            None => {
                self.close_section();
                Ok(IsParsingDone::Yes)
            }
        }
    }

    /// Finishes the current section and adds it to the state.
    fn close_section(&mut self) {
        match std::mem::replace(&mut self.open_section, OpenSection::None) {
            OpenSection::None => {}
            OpenSection::SubResource(sub_resource) => {
                self.state.sub_resources.push(sub_resource);
            }
            OpenSection::Node(node) => {
                self.state.nodes.push(node);
            }
        }
    }

    /// Looks at the next token and errors if there is none or if it is not the
    /// expected one.
    fn expect_exact_token(
        &mut self,
        expected: TscnTokenKind,
    ) -> miette::Result<Range<usize>> {
        let TscnToken { kind: got, span } = self.next_token_no_eof()?;

        if got != expected {
            miette::bail! {
                labels = vec![
                    LabeledSpan::at(span.clone(), "this token"),
                ],
                "Expected '{expected}' but got {got}",
            }
        }

        Ok(span)
    }

    /// Looks at the next token and errors if there is none or if it is not the
    /// expected one.
    ///
    /// Then, it checks if the token's content is equal to the expected one.
    fn expect_exact_token_with_content(
        &mut self,
        expected_token: TscnTokenKind,
        expected_content: &str,
    ) -> miette::Result<()> {
        let range = self.expect_exact_token(expected_token)?;

        let got_content = &self.source[range.clone()];

        if got_content != expected_content {
            miette::bail! {
                labels = vec![
                    LabeledSpan::at(range.clone(), "this token"),
                ],
                "Expected '{expected_content}' but got '{got_content}'",
            }
        }

        Ok(())
    }

    /// Expects a primitive value:
    /// - Number
    /// - String
    /// - True
    /// - False
    fn expect_primitive(&mut self) -> miette::Result<Value> {
        let token = self.next_token_no_eof()?;
        match token.kind {
            TscnTokenKind::Number => {
                let number = self.source[token.span.clone()]
                    .parse()
                    .map_err(|err| {
                        miette::miette! {
                            labels = vec![
                                LabeledSpan::at(token.span.clone(), "this number"),
                            ],
                            "Failed to parse number: {err}",
                        }
                    })?;
                Ok(Value::Number(number))
            }
            TscnTokenKind::String => {
                let string = self.source[token.span.clone()].to_owned();
                Ok(Value::String(string))
            }
            TscnTokenKind::True => Ok(Value::Bool(true)),
            TscnTokenKind::False => Ok(Value::Bool(false)),
            got => {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(token.span, "this token"),
                    ],
                    "Expected primitive value, got {got}",
                }
            }
        }
    }

    /// Expects a dictionary of attributes to follow in the token iterator.
    /// Returns the position of the closing square bracket and the attributes.
    ///
    /// Ignores spaces.
    fn expect_attributes(&mut self) -> miette::Result<(usize, Map<Value>)> {
        let mut map = Map::default();

        loop {
            match self.next_token_no_eof()? {
                TscnToken {
                    kind: TscnTokenKind::Space,
                    ..
                } => {
                    // ignore spaces
                }
                TscnToken {
                    kind: TscnTokenKind::SquareBracketClose,
                    span,
                } => {
                    // we are done with the attributes
                    break Ok((span.end, map));
                }
                TscnToken {
                    kind: TscnTokenKind::Identifier,
                    span,
                } => {
                    // we have an attribute
                    let attribute_name = &self.source[span.clone()];
                    self.expect_exact_token(TscnTokenKind::Equal)?;
                    let attribute_value = self.expect_primitive()?;
                    map.insert(attribute_name.to_owned(), attribute_value);
                }
                TscnToken { kind: got, span } => {
                    miette::bail! {
                        labels = vec![
                            LabeledSpan::at(span, "this token"),
                        ],
                        "Expected attribute identifier, got {got}",
                    }
                }
            }
        }
    }

    /// Returns the next token or an error if there is none.
    fn next_token_no_eof(&mut self) -> miette::Result<TscnToken> {
        self.tokens.next().ok_or_else(|| {
            // TODO
            miette::miette! {
                labels = vec![
                    LabeledSpan::at(0..self.source.len() - 1, "this input"),
                ],
                "Unexpected end of file",
            }
        })
    }
}

#[derive(Default)]
enum OpenSection {
    #[default]
    None,
    SubResource(ParsedSubResource),
    Node(ParsedNode),
}

mod tscn_identifiers {
    pub(super) const GD_SCENE: &str = "gd_scene";
    pub(super) const EXT_RESOURCE: &str = "ext_resource";
    pub(super) const SUB_RESOURCE: &str = "sub_resource";
    pub(super) const NODE: &str = "node";
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::rscn::lex::lex;

    #[test]
    fn it_lexes_and_parses_tscn() -> miette::Result<()> {
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

            let state = parse(&tscn, tokens)?;
            assert!(
                !state.ext_resources.is_empty(),
                "No external resources found"
            );
            assert!(!state.sub_resources.is_empty(), "No sub resources found");
            assert!(!state.nodes.is_empty(), "No nodes found");
        }

        Ok(())
    }
}
