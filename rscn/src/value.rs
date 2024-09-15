//! Generic kind of values that can be contained in a tscn file.
//!
//! See [crate::godot] module for Godot specific declarations.

use std::{borrow::Borrow, collections::BTreeMap};

use miette::LabeledSpan;

use super::lex::{TscnToken, TscnTokenKind};

/// Analogical struct to serde crates' `Value`s.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// We coerce all numbers to f64.
    Number(f64),
    /// Good old string is captured with regex `"[A-Za-z0-9_/?:\. ]+"`.
    ///
    /// tscn values can have string references, which is a string with a `&`
    /// prefix. There's no distinction between string and string reference
    /// in this struct.
    String(String),
    /// Boolean values are `true` or `false` in the tscn format.
    Bool(bool),
    /// Class is a type of value that has a name and a list of values.
    ///
    /// Example:
    /// ```tscn
    /// ClassName(0, "or", "more", "values")
    /// ```
    Class(String, Vec<Value>),
    /// Array is a list of values.
    Array(Vec<Value>),
    /// Dictionary is a map of string keys to values.
    Object(Map<String, Value>),
}

type MapImpl<K, V> = BTreeMap<K, V>;

/// Similar to `serde_json::Map`.
#[derive(Debug, Clone, PartialEq)]
pub struct Map<K, V> {
    map_impl: MapImpl<K, V>,
}

impl Value {
    /// Only returns [Some] for [Value::String].
    pub fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}

impl<K: Ord> Map<K, Value> {
    /// See [BTreeMap::insert].
    pub fn insert(&mut self, key: K, value: Value) -> Option<Value> {
        self.map_impl.insert(key, value)
    }

    /// See [BTreeMap::remove].
    pub fn remove<Q>(&mut self, key: &Q) -> Option<Value>
    where
        Q: ?Sized,
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.map_impl.remove(key)
    }

    /// See [BTreeMap::is_empty].
    pub fn is_empty(&self) -> bool {
        self.map_impl.is_empty()
    }

    /// See [BTreeMap::len].
    pub fn len(&self) -> usize {
        self.map_impl.len()
    }

    /// See [BTreeMap::entry].
    pub fn entry(
        &mut self,
        key: K,
    ) -> std::collections::btree_map::Entry<K, Value> {
        self.map_impl.entry(key)
    }
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self {
            map_impl: MapImpl::default(),
        }
    }
}

impl<K> IntoIterator for Map<K, Value> {
    type Item = (K, Value);
    type IntoIter = std::collections::btree_map::IntoIter<K, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.map_impl.into_iter()
    }
}

impl Value {
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
                Ok(Value::Number(number))
            }
            TscnTokenKind::String => {
                let string = source[token.span.clone()].to_owned();
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
}
