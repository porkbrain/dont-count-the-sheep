use std::collections::BTreeMap;

use miette::LabeledSpan;

use super::{parse_string, Parser, PeekableExt, SpannedValue};
use crate::lex::{TscnToken, TscnTokenKind};

impl<'a, I> Parser<'a, I>
where
    I: PeekableExt + Iterator<Item = TscnToken>,
{
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
    pub(super) fn expect_value(&mut self) -> miette::Result<SpannedValue> {
        let first_token = self.next_token_no_eof_ignore_spaces()?;

        match first_token {
            // we found a class!
            // e.g. Vector2(20, -56)
            TscnToken {
                kind: TscnTokenKind::Identifier,
                span,
            } => {
                let class_starts_at = span.start;
                let class_name = &self.source[span.clone()];
                self.expect_exact(TscnTokenKind::ParenOpen)?;

                let mut values = Vec::new();
                let class_ends_at = loop {
                    let value = self.expect_value()?;
                    values.push(value);

                    match self.next_token_no_eof_ignore_spaces()? {
                        TscnToken {
                            kind: TscnTokenKind::ParenClose,
                            span,
                        } => break span.end,
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
                };

                Ok(SpannedValue::Class(
                    class_starts_at..class_ends_at,
                    class_name.to_owned(),
                    values,
                ))
            }
            // we found an array!
            TscnToken {
                kind: TscnTokenKind::SquareBracketOpen,
                span,
            } => {
                let arr_starts_at = span.start;
                let mut values = vec![];
                let mut is_first_el = true;
                let arr_ends_at = loop {
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
                            span,
                        }) => {
                            let span_end = span.end;
                            self.tokens.next(); // skip ']'
                            break span_end;
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
                };

                Ok(SpannedValue::Array(arr_starts_at..arr_ends_at, values))
            }
            // we found an object!
            TscnToken {
                kind: TscnTokenKind::CurlyBracketOpen,
                span,
            } => {
                // we look for a string (key) -> a colon -> a value ->
                // either a comma or a closing curly bracket

                let object_starts_at = span.start;
                let mut map = BTreeMap::default();
                let mut is_first_el = true;
                let object_ends_at = loop {
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
                            span,
                        }) => {
                            let span_end = span.end;
                            self.tokens.next(); // skip '}'
                            break span_end;
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
                            let key = parse_string(self.source, span)?;

                            self.tokens.next(); // skip string key
                            self.expect_exact(TscnTokenKind::Colon)?;

                            let value = self.expect_value()?;
                            map.insert(key, value);

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
                };

                Ok(SpannedValue::Object(object_starts_at..object_ends_at, map))
            }
            // we found a primitive!
            token @ TscnToken {
                kind:
                    TscnTokenKind::Number
                    | TscnTokenKind::String
                    | TscnTokenKind::True
                    | TscnTokenKind::False,
                ..
            } => SpannedValue::try_from_token(self.source, token),
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
}
