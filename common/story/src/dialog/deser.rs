//! We store dialogs in TOML files.
//! Here we parse that into Rust representation.

use std::{borrow::Cow, iter, str::FromStr};

use bevy::utils::{default, hashbrown::HashMap};
use serde::Deserialize;
use serde_with::{formats::PreferOne, serde_as, OneOrMany};

use crate::{
    dialog::{DialogGraph, GuardKind, LocalNodeName, Node, NodeKind},
    Character,
};

const ROOT_NODE_NAME: &str = "_root";

#[derive(Debug, Deserialize)]
pub(super) struct ParsedToml {
    #[serde(default)]
    dialog: ParsedDialog,

    /// Must always be present.
    root: ParsedNode,

    #[serde(rename = "node")]
    nodes: Vec<ParsedNode>,
}

#[derive(Debug, Deserialize, Default)]
struct ParsedDialog {
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

/// 1. Create a map of nodes, where the key is the node name.
/// 2. Add edges to the nodes.
/// 3. Find the root node.
pub(super) fn from_toml(
    ParsedToml {
        dialog,
        mut root,
        nodes,
    }: ParsedToml,
) -> DialogGraph {
    let mut node_map = HashMap::with_capacity(nodes.len() + 3);
    node_map.insert(
        LocalNodeName::EndDialog,
        Node {
            who: Character::Winnie,
            name: LocalNodeName::EndDialog,
            kind: NodeKind::Guard {
                kind: GuardKind::EndDialog,
                params: default(),
            },
            next: Vec::new(),
        },
    );
    node_map.insert(
        LocalNodeName::Emerge,
        Node {
            who: Character::Winnie,
            name: LocalNodeName::Emerge,
            kind: NodeKind::Guard {
                kind: GuardKind::Emerge,
                params: default(),
            },
            next: Vec::new(),
        },
    );

    //
    // 1.
    //
    if let Some(name) = &root.name {
        assert_eq!(ROOT_NODE_NAME, name, "Root node must be called _root");
    } else {
        root.name = Some(ROOT_NODE_NAME.to_owned());
    }

    let mut prev_who = Character::Winnie;
    for (index, node) in nodes.iter().chain(iter::once(&root)).enumerate() {
        let who = who_from_vars(&dialog.vars, &node).unwrap_or(prev_who);
        let name = node
            .name
            .clone()
            .map(From::from)
            .unwrap_or(LocalNodeName::Auto(index));
        let kind = node
            .guard
            .as_ref()
            .map(|name| NodeKind::Guard {
                kind: guard_name_to_lazy_state(name.as_str()),
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
    for (index, node) in nodes.iter().chain(iter::once(&root)).enumerate() {
        let name = node
            .name
            .clone()
            .map(From::from)
            .unwrap_or(LocalNodeName::Auto(index));

        if node.next.is_empty() {
            // if no next node is specified, that implies that the next node
            // is the one next in the node list

            let next_name = nodes
                .get(index + 1)
                .map(|next| {
                    next.name
                        .clone()
                        .map(From::from)
                        .unwrap_or(LocalNodeName::Auto(index + 1))
                })
                .unwrap_or_else(|| panic!("Node '{name:?}' has no next node"));

            node_map.get_mut(&name).unwrap().next.push(next_name);

            continue;
        }

        for next in &node.next {
            let next_name = LocalNodeName::from(next.clone());
            // asserts node exists
            node_map
                .get(&next_name)
                .unwrap_or_else(|| panic!("Node '{next}' not found"));

            node_map.get_mut(&name).unwrap().next.push(next_name);
        }
    }

    DialogGraph { nodes: node_map }
}

// TODO: this can be deleted and done with strum's `EnumString`
fn guard_name_to_lazy_state(name: &str) -> GuardKind {
    match name {
        "exhaustive_alternatives" => GuardKind::ExhaustiveAlternatives,
        "reach_last_alternative" => GuardKind::ReachLastAlternative,
        "end_dialog" => GuardKind::EndDialog,
        "emerge" => GuardKind::Emerge,
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

impl From<String> for LocalNodeName {
    fn from(s: String) -> Self {
        match s.as_str() {
            "_end_dialog" => LocalNodeName::EndDialog,
            "_emerge" => LocalNodeName::Emerge,
            ROOT_NODE_NAME => LocalNodeName::Root,
            _ => LocalNodeName::Explicit(s),
        }
    }
}
