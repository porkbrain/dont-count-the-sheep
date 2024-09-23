//! Generic kind of values that can be contained in a tscn file.
//!
//! See [crate::godot] module for Godot specific declarations.

use std::{collections::BTreeMap, ops::Range};

use miette::LabeledSpan;

use super::lex::{TscnToken, TscnTokenKind};

/// Analogical struct to serde crates' `Value`s, but also contains information
/// about the span of the value in the source.
#[derive(Debug, Clone, PartialEq)]
pub enum SpannedValue {
    /// We coerce all numbers to f64.
    Number(Range<usize>, f64),
    /// Good old string is captured with regex `"[A-Za-z0-9_/?:\. ]+"`.
    ///
    /// tscn values can have string references, which is a string with a `&`
    /// prefix. There's no distinction between string and string reference
    /// in this struct.
    String(Range<usize>, String),
    /// Boolean values are `true` or `false` in the tscn format.
    Bool(Range<usize>, bool),
    /// Class is a type of value that has a name and a list of values.
    ///
    /// Example:
    /// ```tscn
    /// ClassName(0, "or", "more", "values")
    /// ```
    ///
    /// Span contains length from the class name first char to the closing
    /// parenthesis.
    Class(Range<usize>, String, Vec<SpannedValue>),
    /// Array is a list of values.
    ///
    /// Span contains length from the opening bracket to the closing bracket.
    Array(Range<usize>, Vec<SpannedValue>),
    /// Dictionary is a map of string keys to values.
    ///
    /// Span contains length from the opening curly brace to the closing curly
    /// brace if dictionary, or from the first nested key char to the last
    /// nested key's value char if a nested key.
    Object(Range<usize>, BTreeMap<String, SpannedValue>),
}

impl SpannedValue {
    /// Only returns [Some] for [Value::String].
    pub fn into_string(self) -> Option<String> {
        match self {
            Self::String(_, s) => Some(s),
            _ => None,
        }
    }

    /// Tries to convert the value into a class.
    ///
    /// Returns an error with labelled span if the value is not a class.
    pub fn try_into_class(
        self,
    ) -> miette::Result<(Range<usize>, String, Vec<Self>)> {
        let got = self.variant_name();
        let expected = "class";
        match self {
            Self::Class(span, class, values) => Ok((span, class, values)),

            Self::String(span, _)
            | Self::Bool(span, _)
            | Self::Object(span, _)
            | Self::Number(span, _)
            | Self::Array(span, _) => {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(span, "this {got}"),
                    ],
                    "Expected {expected}, got {got}",
                }
            }
        }
    }

    /// Tries to convert the value into a string.
    ///
    /// Returns an error with labelled span if the value is not a string.
    pub fn try_into_string(self) -> miette::Result<(Range<usize>, String)> {
        let got = self.variant_name();
        let expected = "string";
        match self {
            Self::String(span, string) => Ok((span, string)),
            Self::Class(span, _, _)
            | Self::Bool(span, _)
            | Self::Object(span, _)
            | Self::Number(span, _)
            | Self::Array(span, _) => {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(span, "this {got}"),
                    ],
                    "Expected {expected}, got {got}",
                }
            }
        }
    }

    /// Tries to convert the value into a number.
    ///
    /// Returns an error with labelled span if the value is not a number.
    pub fn try_into_number(self) -> miette::Result<(Range<usize>, f64)> {
        let got = self.variant_name();
        let expected = "number";
        match self {
            Self::Number(span, number) => Ok((span, number)),
            Self::Class(span, _, _)
            | Self::Bool(span, _)
            | Self::Object(span, _)
            | Self::String(span, _)
            | Self::Array(span, _) => {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(span, "this {got}"),
                    ],
                    "Expected {expected}, got {got}",
                }
            }
        }
    }

    /// Tries to convert the value into a boolean.
    ///
    /// Returns an error with labelled span if the value is not a boolean.
    pub fn try_into_bool(self) -> miette::Result<(Range<usize>, bool)> {
        let got = self.variant_name();
        let expected = "boolean";
        match self {
            Self::Bool(span, b) => Ok((span, b)),
            Self::Class(span, _, _)
            | Self::Number(span, _)
            | Self::Object(span, _)
            | Self::String(span, _)
            | Self::Array(span, _) => {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(span, "this {got}"),
                    ],
                    "Expected {expected}, got {got}",
                }
            }
        }
    }

    /// Tries to convert the value into an object.
    ///
    /// Returns an error with labelled span if the value is not an object.
    pub fn try_into_object(
        self,
    ) -> miette::Result<(Range<usize>, BTreeMap<String, SpannedValue>)> {
        let got = self.variant_name();
        let expected = "object";
        match self {
            Self::Object(span, object) => Ok((span, object)),
            Self::Class(span, _, _)
            | Self::Number(span, _)
            | Self::Bool(span, _)
            | Self::String(span, _)
            | Self::Array(span, _) => {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(span, "this {got}"),
                    ],
                    "Expected {expected}, got {got}",
                }
            }
        }
    }

    /// Tries to convert the value into an array.
    ///
    /// Returns an error with labelled span if the value is not an array.
    pub fn try_into_array(
        self,
    ) -> miette::Result<(Range<usize>, Vec<SpannedValue>)> {
        let got = self.variant_name();
        let expected = "array";
        match self {
            Self::Array(span, array) => Ok((span, array)),
            Self::Class(span, _, _)
            | Self::Number(span, _)
            | Self::Bool(span, _)
            | Self::String(span, _)
            | Self::Object(span, _) => {
                miette::bail! {
                    labels = vec![
                        LabeledSpan::at(span, "this {got}"),
                    ],
                    "Expected {expected}, got {got}",
                }
            }
        }
    }

    /// Tries to convert the value into a class with specific name of a specific
    /// length.
    ///
    /// Returns an error with labelled span if
    /// - the value is not a class or
    /// - class name is not the expected one or
    /// - the class does not have the expected number of values.
    pub fn try_into_this_class_of_len<const N: usize>(
        self,
        class_name: &str,
    ) -> miette::Result<[Self; N]> {
        let (span, class, values) = self.try_into_class()?;

        if class != class_name {
            miette::bail! {
                labels = vec![
                    LabeledSpan::at(span.clone(), "this class"),
                ],
                "Expected class Vector2, got {class}",
            }
        }

        let len = values.len();
        if len != N {
            miette::bail! {
                labels = vec![
                    LabeledSpan::at(span, "this class"),
                ],
                "Expected {N} values, got {len}",
            }
        }

        // SAFETY: we just checked that the length is N.
        Ok(values.try_into().unwrap())
    }

    fn variant_name(&self) -> &'static str {
        match self {
            Self::Class(_, _, _) => "class",
            Self::Array(_, _) => "array",
            Self::Number(_, _) => "number",
            Self::String(_, _) => "string",
            Self::Bool(_, _) => "boolean",
            Self::Object(_, _) => "object",
        }
    }
}

impl SpannedValue {
    /// Tries to parse number, string, true, or false.
    pub(super) fn try_from_token(
        source: &str,
        token: TscnToken,
    ) -> Result<Self, miette::Error> {
        match token.kind {
            TscnTokenKind::Number => {
                let number = source[token.span.clone()]
                    .parse()
                    .map_err(|err| {
                        miette::miette! {
                            labels = vec![
                                LabeledSpan::at(token.span.clone(), "this number"),
                            ],
                            "Failed to parse number: {err}",
                        }
                    })?;
                Ok(SpannedValue::Number(token.span, number))
            }
            TscnTokenKind::String => {
                let string = source[token.span.clone()].to_owned();
                Ok(SpannedValue::String(token.span, string))
            }
            TscnTokenKind::True => Ok(SpannedValue::Bool(token.span, true)),
            TscnTokenKind::False => Ok(SpannedValue::Bool(token.span, false)),
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
}
