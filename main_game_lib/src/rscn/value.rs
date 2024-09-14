use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    Number(f64),
    String(String),
    Bool(bool),
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
    pub(crate) fn insert(&mut self, key: String, value: Value) {
        self.map_impl.insert(key, value);
    }

    pub(crate) fn remove(&mut self, key: &str) -> Option<Value> {
        self.map_impl.remove(key)
    }
}

impl<V> Default for Map<V> {
    fn default() -> Self {
        Self {
            map_impl: MapImpl::default(),
        }
    }
}
