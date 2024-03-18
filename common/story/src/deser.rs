//! We store dialogs in TOML files.
//! Here we parse that into Rust representation.

use std::{borrow::Cow, str::FromStr};

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext},
    reflect::Reflect,
    utils::{default, hashbrown::HashMap},
};
use serde::{Deserialize, Serialize};
use serde_with::{formats::PreferOne, serde_as, OneOrMany};
use thiserror::Error;

use crate::Character;

#[derive(Asset, Deserialize, Serialize, Reflect)]
pub struct Dialog {
    pub nodes: HashMap<NodeName, Node>,
    pub root: NodeName,
}

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub struct Node {
    pub name: NodeName,
    pub who: Character,
    pub kind: NodeKind,
    pub next: Vec<NodeName>,
}

#[derive(
    Debug, Deserialize, Serialize, Reflect, Clone, Hash, PartialEq, Eq,
)]
pub enum NodeName {
    Explicit(String),
    Auto(usize),
    EndDialog,
    Emerge,
}

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub enum NodeKind {
    Guard {
        /// Guard states are persisted across dialog sessions if
        /// - the node has a [`NodeName::Explicit`]
        ///
        /// Otherwise the state is discarded after the dialog is over.
        state: Guard,
        #[reflect(ignore)]
        params: HashMap<String, toml::Value>,
    },
    Vocative {
        /// The dialog line to print.
        line: String,
    },
}

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub enum Guard {
    EndDialog,
    Emerge,
    ExhaustiveAlternatives(LazyGuardState<ExhaustiveAlternativesState>),
}

/// Loading state for each guard is unnecessary until we actually prompt the
/// guard.
/// This abstraction allows us to defer loading.
#[derive(Debug, Default, Deserialize, Serialize, Reflect)]
pub enum LazyGuardState<T> {
    Ready(T),
    #[default]
    Load,
}

#[derive(Debug, Serialize, Deserialize, Default, Reflect)]
pub struct ExhaustiveAlternativesState {
    pub next_to_show: usize,
}

/// Loads .toml files into [`Dialog`] representation.
#[derive(Default)]
pub struct DialogLoader;

/// Errors that can occur when loading assets from .toml files.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LoaderError {
    /// The file could not be loaded, most likely not found.
    #[error("Could load .toml file: {0}")]
    Io(#[from] std::io::Error),
    /// We convert the file bytes into a string, which can fail.
    #[error("Non-utf8 string in .toml file: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    /// The .toml file could not be parsed.
    #[error("Error parsing .toml file: {0}")]
    Toml(#[from] toml::de::Error),
}

impl AssetLoader for DialogLoader {
    type Asset = Dialog;
    type Settings = ();
    type Error = LoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let s = std::str::from_utf8(&bytes)?;
            let toml = toml::from_str(s)?;
            Ok(dialog_from_toml(toml))
        })
    }

    fn extensions(&self) -> &[&str] {
        &[]
    }
}

/// 1. Create a map of nodes, where the key is the node name.
/// 2. Add edges to the nodes.
/// 3. Find the root node.
fn dialog_from_toml(ParsedToml { dialog, nodes }: ParsedToml) -> Dialog {
    assert!(!nodes.is_empty(), "Dialog has no nodes");

    let mut node_map = HashMap::with_capacity(nodes.len() + 2);
    node_map.insert(
        NodeName::EndDialog,
        Node {
            who: Character::Winnie,
            name: NodeName::EndDialog,
            kind: NodeKind::Guard {
                state: Guard::EndDialog,
                params: default(),
            },
            next: Vec::new(),
        },
    );
    node_map.insert(
        NodeName::Emerge,
        Node {
            who: Character::Winnie,
            name: NodeName::Emerge,
            kind: NodeKind::Guard {
                state: Guard::Emerge,
                params: default(),
            },
            next: Vec::new(),
        },
    );

    //
    // 1.
    //
    let mut prev_who = Character::Winnie;
    for (index, node) in nodes.iter().enumerate() {
        let who = who_from_vars(&dialog.vars, &node).unwrap_or(prev_who);
        let name = node
            .name
            .clone()
            .map(From::from)
            .unwrap_or(NodeName::Auto(index));
        let kind = node
            .guard
            .as_ref()
            .map(|name| NodeKind::Guard {
                state: guard_name_to_lazy_state(name.as_str()),
                params: params_from_vars(&dialog.vars, &node),
            })
            .unwrap_or_else(|| NodeKind::Vocative {
                line: node
                    .en
                    .clone()
                    .map(|s| string_from_vars(&dialog.vars, s))
                    .unwrap_or_else(|| {
                        panic!("Node '{name:?}' has no dialog line")
                    }),
            });

        let prev_node = node_map.insert(
            name.clone(),
            Node {
                name,
                who,
                kind,
                // we will add edges later
                next: Vec::with_capacity(node.next.len()),
            },
        );
        assert!(prev_node.is_none(), "Duplicate node '{prev_node:?}'");

        prev_who = who;
    }

    //
    // 2.
    //
    for (index, node) in nodes.iter().enumerate() {
        let name = node
            .name
            .clone()
            .map(From::from)
            .unwrap_or(NodeName::Auto(index));

        if node.next.is_empty() {
            // if no next node is specified, that implies that the next node
            // is the one next in the node list

            let next_name = nodes
                .get(index + 1)
                .map(|next| {
                    next.name
                        .clone()
                        .map(From::from)
                        .unwrap_or(NodeName::Auto(index + 1))
                })
                .unwrap_or_else(|| panic!("Node '{name:?}' has no next node"));

            node_map.get_mut(&name).unwrap().next.push(next_name);

            continue;
        }

        for next in &node.next {
            let next_name = NodeName::from(next.clone());
            // asserts node exists
            node_map
                .get(&next_name)
                .unwrap_or_else(|| panic!("Node '{next}' not found"));

            node_map.get_mut(&name).unwrap().next.push(next_name);
        }
    }

    //
    // 3.
    //
    let root_name = if let Some(name) = dialog.first_node {
        NodeName::Explicit(name)
    } else {
        nodes
            .get(0)
            .unwrap()
            .name
            .clone()
            .map(From::from)
            .unwrap_or(NodeName::Auto(0))
    };
    // asserts root node exists
    node_map.get(&root_name).expect("Root node not found");

    Dialog {
        nodes: node_map,
        root: root_name,
    }
}

fn guard_name_to_lazy_state(name: &str) -> Guard {
    match name {
        "exhaustive_alternatives" => Guard::ExhaustiveAlternatives(default()),
        "end_dialog" => Guard::EndDialog,
        "emerge" => Guard::Emerge,
        _ => panic!("Unknown guard '{name}'"),
    }
}

fn params_from_vars(
    vars: &toml::Table,
    node: &ParsedNode,
) -> HashMap<String, toml::Value> {
    node.params
        .clone()
        .into_iter()
        .map(|(key, value)| match value.as_str() {
            Some(v) => {
                if v.starts_with("${")
                    && v.ends_with("}")
                    && let Some(new) = vars.get(&v[2..v.len() - 1])
                {
                    // preserves toml value type
                    (key, new.clone())
                } else {
                    (
                        key,
                        toml::Value::String(string_from_vars(
                            vars,
                            v.to_owned(),
                        )),
                    )
                }
            }
            _ => (key, value),
        })
        .collect()
}

/// There can be variables anywhere in the value, and multiple of
/// them as well.
/// Find all patterns ${[a-z0-9_]+} and replace them with the
/// appropriate var.
/// If the var is not string, convert it to string.
fn string_from_vars(vars: &toml::Table, mut v: String) -> String {
    while let Some(start) = v.rfind("${") {
        let end = v
            .rfind('}')
            .unwrap_or_else(|| panic!("Unmatched '${{' in '{v}'"));

        assert!(end > start, "Unmatched '${{' in '{v}'");
        let v_name = &v[(start + 2)..end];

        let replace_with = vars
            .get(v_name)
            .unwrap_or_else(|| panic!("Variable '{v_name}' not found ('{v}')"));
        let replace_with = match replace_with {
            toml::Value::Integer(i) => Cow::Owned(i.to_string()),
            toml::Value::Float(f) => Cow::Owned(f.to_string()),
            toml::Value::String(s) => Cow::Borrowed(s.as_str()),
            _ => panic!("Variable '{v_name}' must be a number or a string"),
        };

        v = v.replace(&v[start..=end], &replace_with);
    }

    v
}

fn who_from_vars(vars: &toml::Table, node: &ParsedNode) -> Option<Character> {
    node.who.as_ref().map(|who| {
        let s = if who.starts_with("${") && who.ends_with("}") {
            vars.get(&who[2..who.len() - 1])
                .unwrap_or_else(|| panic!("Variable '{who}' not found"))
                .as_str()
                .expect("Variable for 'who' must be a string")
        } else {
            who.as_str()
        };

        Character::from_str(s)
            .unwrap_or_else(|_| panic!("Unknown character '{s}'"))
    })
}

impl From<String> for NodeName {
    fn from(s: String) -> Self {
        match s.as_str() {
            "_end_dialog" => NodeName::EndDialog,
            "_emerge" => NodeName::Emerge,
            _ => NodeName::Explicit(s),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ParsedToml {
    dialog: ParsedDialog,

    #[serde(rename = "node")]
    nodes: Vec<ParsedNode>,
}

#[derive(Debug, Deserialize)]
struct ParsedDialog {
    first_node: Option<String>,
    #[serde(default)]
    vars: toml::Table,
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParsedNode {
    name: Option<String>,
    guard: Option<String>,
    #[serde(default)]
    params: toml::Table,
    who: Option<String>,
    en: Option<String>,
    #[serde(default)]
    #[serde_as(deserialize_as = "OneOrMany<_, PreferOne>")]
    next: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_basic_example() {
        dialog_from_toml(
            toml::from_str(include_str!("../tests/basic.toml")).unwrap(),
        );
    }
}
