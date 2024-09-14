use std::collections::BTreeMap;

use miette::LabeledSpan;

use super::lex::{TscnToken, TscnTokenKind};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Class(String, Vec<Value>),
    Array(Vec<Value>),
    Object(Map<Value>),
}

type MapImpl<V> = BTreeMap<String, V>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Map<V> {
    map_impl: MapImpl<V>,
}

impl Value {
    pub(crate) fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}

impl Map<Value> {
    pub(crate) fn insert(
        &mut self,
        key: String,
        value: Value,
    ) -> Option<Value> {
        self.map_impl.insert(key, value)
    }

    pub(crate) fn remove(&mut self, key: &str) -> Option<Value> {
        self.map_impl.remove(key)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.map_impl.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.map_impl.len()
    }

    pub(crate) fn entry(
        &mut self,
        key: String,
    ) -> std::collections::btree_map::Entry<String, Value> {
        self.map_impl.entry(key)
    }
}

impl<V> Default for Map<V> {
    fn default() -> Self {
        Self {
            map_impl: MapImpl::default(),
        }
    }
}

impl IntoIterator for Map<Value> {
    type Item = (String, Value);
    type IntoIter = std::collections::btree_map::IntoIter<String, Value>;

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
