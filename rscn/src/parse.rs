//! Parses a list of [TscnToken]s.
//!
//! We are parsing under the optimistic assumption that the .tscn file comes
//! from Godot.

mod ext_resource;
mod node;
mod sub_resource;

use std::{iter::Peekable, ops::Range};

use miette::{Context, LabeledSpan};

use super::{
    lex::{TscnToken, TscnTokenKind},
    value::*,
};
use crate::godot::*;

struct Parser<'a, I> {
    source: &'a str,
    tokens: I,
    state: Scene,
    open_section: OpenSection,
    /// We assert that the tscn string is not empty.
    /// This value starts on 0 which is going to be a valid index.
    ///
    /// Every time call [Self::next_token_no_eof_ignore_spaces] we update this
    /// value to the end of the read token.
    ///
    /// If we reach the end of the all tokens (EOF) but we are in an open state
    /// that's not ready to be closed, we error.
    /// This index is used to give better error messages because it can print
    /// the text where we expected more tokens to be.
    last_token_end: usize,
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
) -> miette::Result<Scene> {
    if tscn.is_empty() {
        miette::bail!("Empty .tscn source");
    }

    let mut parser = Parser {
        source: tscn,
        tokens: tokens.into_iter().peekable(),
        state: Scene::default(),
        open_section: OpenSection::default(),
        last_token_end: 0,
    };

    error_with_source_code!(tscn, parser.parse_headers());

    while let IsParsingDone::No =
        error_with_source_code!(tscn, parser.parse_next_statement())
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
    I: PeekableExt + Iterator<Item = TscnToken>,
{
    /// Each tscn file must start with a `gd_scene` heading.
    /// There must be exactly one `gd_scene` heading per tscn.
    ///
    /// Parses the attributes into [State::headers].
    fn parse_headers(&mut self) -> miette::Result<()> {
        // [gd_scene opt_attr1=... opt_attr2=... ... ]

        self.expect_exact(TscnTokenKind::SquareBracketOpen)?;
        self.expect_exact_with_content(
            TscnTokenKind::Identifier,
            tscn_identifiers::GD_SCENE,
        )?;
        // this ignores spaces
        let (_, headers) = self.expect_attributes()?;
        // happens exactly once, overwrites defaults (that did not allocate)
        self.state.headers = headers;

        Ok(())
    }

    /// Parses next chunk of tokens that together form a statement.
    /// There are two kinds of statements in .tscn:
    /// - A section heading (e.g. `[node ...]`)
    /// - A key-value pair (e.g. `position = ...`)
    ///
    /// This is meant to be called in a loop until it returns
    /// `Ok(IsParsingDone::Yes)` or error.
    fn parse_next_statement(&mut self) -> miette::Result<IsParsingDone> {
        match self.tokens.next() {
            Some(TscnToken {
                kind: TscnTokenKind::SquareBracketOpen,
                ..
            }) => {
                // we are starting a new section!
                self.close_section();
                self.parse_section_heading()?;

                Ok(IsParsingDone::No)
            }
            Some(TscnToken {
                kind: TscnTokenKind::Identifier,
                span,
            }) => {
                self.parse_section_key(span)?;

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

    /// Just found a '[' token, so now we expect the rest of the section
    /// heading.
    fn parse_section_heading(&mut self) -> miette::Result<()> {
        let span = self.expect_exact(TscnTokenKind::Identifier)?;
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
                self.open_section =
                    OpenSection::SubResource(sub_resource::parse_attributes(
                        span.start..ends_at,
                        ext_attrs,
                    )?);
            }
            tscn_identifiers::NODE => {
                let (ends_at, ext_attrs) = self.expect_attributes()?;
                self.open_section = OpenSection::Node(node::parse_attributes(
                    span.start..ends_at,
                    ext_attrs,
                )?);
            }
            unknown_section => {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(span, "this section"),
                    ],
                    "Unknown section '{unknown_section}'",
                }
            }
        };

        Ok(())
    }

    /// Parses a section key at given range and its value that should follow
    /// in the token iterator.
    fn parse_section_key(
        &mut self,
        key_span: Range<usize>,
    ) -> miette::Result<()> {
        let key = self.source[key_span.clone()].to_owned();

        // tscn supports keys with forward slashes to create nested
        // dictionaries
        // e.g. `key/subkey = value`
        let nested_key = if let Some(TscnToken {
            kind: TscnTokenKind::ForwardSlash,
            ..
        }) = self.peek_next_token_swallow_spaces()
        {
            self.tokens.next(); // skip '/'
            let nested_key_span =
                self.expect_exact(TscnTokenKind::Identifier)?;
            let nested_key = self.source[nested_key_span.clone()].to_owned();
            Some((nested_key_span, nested_key))
        } else {
            None
        };

        self.expect_exact(TscnTokenKind::Equal).with_context(|| {
            format!("Section key must be in format '{key} = value'")
        })?;

        let value = self.expect_value()?;

        /// Inserts a section key into the current node or subresource,
        /// depending on the type of `K`.
        ///
        /// If `nested_key` is `Some`, it inserts the value into the nested
        /// dictionary.
        fn insert_section_key<K: Ord>(
            section_keys: &mut Map<K, Value>,
            (key_span, key): (Range<usize>, K),
            nested_key: Option<(Range<usize>, String)>,
            value: Value,
        ) -> miette::Result<()> {
            if let Some((nested_key_span, nested_key)) = nested_key {
                let nested_dict = section_keys
                    .entry(key)
                    .or_insert_with(|| Value::Object(Default::default()));

                if let Value::Object(nested_dict) = nested_dict {
                    if nested_dict.insert(nested_key, value).is_some() {
                        miette::bail! {
                            labels = vec![
                                LabeledSpan::at(nested_key_span, "this key"),
                            ],
                            "Duplicate nested key",
                        }
                    }

                    Ok(())
                } else {
                    miette::bail! {
                        labels = vec![
                            LabeledSpan::at(key_span.start..nested_key_span.end, "this key"),
                        ],
                        "Expected object value for nested key",
                    }
                }
            } else if section_keys.insert(key, value).is_some() {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(key_span, "this key"),
                    ],
                    "Duplicate key'",
                }
            } else {
                Ok(())
            }
        }

        match &mut self.open_section {
            OpenSection::None => {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(key_span, "this key"),
                    ],
                    "Unexpected section key '{key}'",
                }
            }
            OpenSection::Node(Node { section_keys, .. }) => insert_section_key(
                section_keys,
                (key_span, From::from(key)),
                nested_key,
                value,
            ),
            OpenSection::SubResource(SubResource { section_keys, .. }) => {
                insert_section_key(
                    section_keys,
                    (key_span, From::from(key)),
                    nested_key,
                    value,
                )
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
    ///
    /// Ignores spaces.
    fn expect_exact(
        &mut self,
        expected: TscnTokenKind,
    ) -> miette::Result<Range<usize>> {
        let TscnToken { kind: got, span } =
            self.next_token_no_eof_ignore_spaces()?;

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

    /// Returns the next token without consuming it, so that [Iterator::next]
    /// called on [Self::tokens] will return the same token.
    ///
    /// Exception for [TscnTokenKind::Space] which is consumed.
    fn peek_next_token_swallow_spaces(&mut self) -> Option<&TscnToken> {
        if let Some(TscnToken {
            kind: TscnTokenKind::Space,
            ..
        }) = self.tokens.peek()
        {
            self.tokens.next();
            self.peek_next_token_swallow_spaces()
        } else {
            self.tokens.peek()
        }
    }

    /// Looks at the next token and errors if there is none or if it is not the
    /// expected one.
    ///
    /// Then, it checks if the token's content is equal to the expected one.
    fn expect_exact_with_content(
        &mut self,
        expected_token: TscnTokenKind,
        expected_content: &str,
    ) -> miette::Result<()> {
        let range = self.expect_exact(expected_token)?;

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
        let token = self.next_token_no_eof_ignore_spaces()?;
        Value::try_from_token(self.source, token)
    }

    /// Expects a dictionary of attributes to follow in the token iterator.
    /// Returns the position of the closing square bracket and the attributes.
    ///
    /// Ignores spaces.
    fn expect_attributes(
        &mut self,
    ) -> miette::Result<(usize, Map<String, Value>)> {
        let mut map = Map::default();

        loop {
            match self.next_token_no_eof_ignore_spaces()? {
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
                    self.expect_exact(TscnTokenKind::Equal)?;
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

    /// A value is what follows after a section key assignment.
    ///
    /// Some examples of section keys and their values:
    ///
    /// ```tscn,no_run
    /// z_index = -3
    ///
    /// position = Vector2(-23, 14)
    ///
    /// animations = [{
    /// "frames": [{
    ///     "duration": 1.0,
    ///     "texture": SubResource("AtlasTexture_yvafp")
    ///     }, {
    ///     "duration": 1.0,
    ///     "texture": SubResource("AtlasTexture_l80js")
    ///     }],
    ///     "loop": true,
    ///     "name": &"default",
    ///     "speed": 5.0
    ///  }]
    /// ```
    fn expect_value(&mut self) -> miette::Result<Value> {
        let first_token = self.next_token_no_eof_ignore_spaces()?;

        match first_token {
            // we found a class!
            // e.g. Vector2(20, -56)
            TscnToken {
                kind: TscnTokenKind::Identifier,
                span,
            } => {
                let class_name = &self.source[span.clone()];
                self.expect_exact(TscnTokenKind::ParenOpen)?;

                let mut values = Vec::new();
                loop {
                    let value = self.expect_value()?;
                    values.push(value);

                    match self.next_token_no_eof_ignore_spaces()? {
                        TscnToken {
                            kind: TscnTokenKind::ParenClose,
                            ..
                        } => break,
                        TscnToken {
                            kind: TscnTokenKind::Comma,
                            ..
                        } => continue,
                        TscnToken { kind, span } => {
                            miette::bail! {
                                labels = vec![
                                    LabeledSpan::at(span, "this token"),
                                ],
                                "Expected ',' or ')', got {kind}",
                            }
                        }
                    }
                }

                Ok(Value::Class(class_name.to_owned(), values))
            }
            // we found an array!
            TscnToken {
                kind: TscnTokenKind::SquareBracketOpen,
                ..
            } => {
                let mut values = vec![];
                let mut is_first_el = true;
                loop {
                    match self.peek_next_token_swallow_spaces() {
                        None => {
                            // TODO
                            miette::bail! {
                                labels = vec![
                                    LabeledSpan::at(self.last_token_end..self.source.len() - 1, "this input"),
                                ],
                                "Unexpected end of file",
                            }
                        }
                        Some(TscnToken {
                            kind: TscnTokenKind::NewLine,
                            ..
                        }) => {
                            self.tokens.next(); // skip '\n'
                        }
                        Some(TscnToken {
                            kind: TscnTokenKind::SquareBracketClose,
                            ..
                        }) => {
                            self.tokens.next(); // skip ']'
                            break;
                        }
                        Some(TscnToken {
                            kind: TscnTokenKind::Comma,
                            span,
                        }) => {
                            if is_first_el {
                                miette::bail! {
                                    labels = vec![
                                        LabeledSpan::at(span.clone(), "this input"),
                                    ],
                                    "Unexpected ','",
                                }
                            }

                            self.tokens.next(); // skip ','
                            values.push(self.expect_value()?);
                        }
                        _ => {
                            is_first_el = false;
                            values.push(self.expect_value()?);
                        }
                    }
                }

                Ok(Value::Array(values))
            }
            // we found an object!
            TscnToken {
                kind: TscnTokenKind::CurlyBracketOpen,
                ..
            } => {
                // we look for a string (key) -> a colon -> a value ->
                // either a comma or a closing curly bracket

                let mut map = Map::default();
                let mut is_first_el = true;
                loop {
                    match self.peek_next_token_swallow_spaces() {
                        None => {
                            // TODO
                            miette::bail! {
                                labels = vec![
                                    LabeledSpan::at(self.last_token_end..self.source.len() - 1, "this input"),
                                ],
                                "Unexpected end of file",
                            }
                        }
                        Some(TscnToken {
                            kind: TscnTokenKind::CurlyBracketClose,
                            ..
                        }) => {
                            self.tokens.next(); // skip '}'
                            break;
                        }
                        Some(TscnToken {
                            kind: TscnTokenKind::NewLine,
                            ..
                        }) => {
                            self.tokens.next(); // skip '\n'
                        }
                        Some(TscnToken {
                            kind: TscnTokenKind::String,
                            span,
                        }) => {
                            let span = span.clone();
                            // TODO: strip quotes
                            let key = self.source[span.clone()].to_owned();
                            self.tokens.next(); // skip string key
                            self.expect_exact(TscnTokenKind::Colon)?;

                            let next_token = self
                                .peek_next_token_swallow_spaces()
                                .ok_or_else(|| {
                                    // TODO
                                    miette::miette! {
                                        labels = vec![
                                            LabeledSpan::at(span, "this input"),
                                        ],
                                        "Unexpected end of file",
                                    }
                                })?;

                            if let TscnToken {
                                kind: TscnTokenKind::Ampersand,
                                ..
                            } = next_token
                            {
                                // not sure why does tscn have string references
                                // but so be it
                                self.tokens.next(); // skip '&'

                                let string_val_span =
                                    self.expect_exact(TscnTokenKind::String)?;

                                map.insert(
                                    key,
                                    Value::String(
                                        self.source[string_val_span.clone()]
                                            .to_owned(),
                                    ),
                                );
                            } else {
                                // any value is possible

                                let value = self.expect_value()?;
                                map.insert(key, value);
                            }

                            is_first_el = false;
                        }
                        Some(TscnToken {
                            kind: TscnTokenKind::Comma,
                            span,
                        }) if is_first_el => {
                            miette::bail! {
                                labels = vec![
                                    LabeledSpan::at(span.clone(), "this input"),
                                ],
                                "Unexpected ','",
                            }
                        }
                        Some(TscnToken {
                            kind: TscnTokenKind::Comma,
                            ..
                        }) => {
                            self.tokens.next(); // skip ','
                        }
                        Some(TscnToken { kind, span }) => {
                            miette::bail! {
                                labels = vec![
                                    LabeledSpan::at(span.clone(), "this token"),
                                ],
                                "Expected string key or '}}', got {kind}",
                            }
                        }
                    }
                }

                Ok(Value::Object(map))
            }
            // we found a primitive!
            token @ TscnToken {
                kind:
                    TscnTokenKind::Number
                    | TscnTokenKind::String
                    | TscnTokenKind::True
                    | TscnTokenKind::False,
                ..
            } => Value::try_from_token(self.source, token),
            // we found something we shouldn't have
            TscnToken { kind: got, span } => {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(span, "this token"),
                    ],
                    "Expected value, got {got}",
                }
            }
        }
    }

    /// Returns the next token or an error if there is none.
    ///
    /// If the next token is a space, ignore it.
    fn next_token_no_eof_ignore_spaces(&mut self) -> miette::Result<TscnToken> {
        loop {
            let token = self.tokens.next().ok_or_else(|| {
                miette::miette! {
                    // SAFETY: we have checked that source is not empty
                    labels = vec![
                        LabeledSpan::at(self.last_token_end..self.source.len() - 1, "this input"),
                    ],
                    "Unexpected end of file",
                }
            })?;

            self.last_token_end = token.span.end;

            if token.kind != TscnTokenKind::Space {
                break Ok(token);
            }
        }
    }
}

#[derive(Default)]
enum OpenSection {
    #[default]
    None,
    SubResource(SubResource),
    Node(Node),
}

mod tscn_identifiers {
    pub(super) const GD_SCENE: &str = "gd_scene";
    pub(super) const EXT_RESOURCE: &str = "ext_resource";
    pub(super) const SUB_RESOURCE: &str = "sub_resource";
    pub(super) const NODE: &str = "node";
}

trait PeekableExt {
    fn peek(&mut self) -> Option<&TscnToken>;
}

impl<I> PeekableExt for Peekable<I>
where
    I: Iterator<Item = TscnToken>,
{
    fn peek(&mut self) -> Option<&TscnToken> {
        self.peek()
    }
}
