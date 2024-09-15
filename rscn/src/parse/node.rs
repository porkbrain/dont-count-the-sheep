use std::ops::Range;

use miette::LabeledSpan;

use crate::parse::{tscn_identifiers::NODE, Map, Node, Value};

const NODE_NAME: &str = "name";
/// Optional attribute, will be none for the root node.
const NODE_PARENT: &str = "parent";
const NODE_KIND: &str = "type";

/// Nodes can have section keys.
pub(super) fn parse_attributes(
    span: Range<usize>,
    mut attrs: Map<String, Value>,
) -> miette::Result<Node> {
    let name = attrs.remove(NODE_NAME).ok_or_else(|| {
        miette::miette! {
            labels = vec![
                LabeledSpan::at(span.clone(), "this attribute"),
            ],
            "Missing '{attr}' attribute in '{section}' section",
            section = NODE,
            attr = NODE_NAME,
        }
    }).and_then(|val| {
        val.into_string().ok_or_else(|| {
            miette::miette! {
                labels = vec![
                    LabeledSpan::at(span.clone(), "this attribute"),
                ],
                "Expected string value for '{attr}' attribute in '{section}' section",
                section = NODE,
                attr = NODE_NAME,
            }
        })
    })?;

    let kind = attrs.remove(NODE_KIND).ok_or_else(|| {
        miette::miette! {
            labels = vec![
                LabeledSpan::at(span.clone(), "this attribute"),
            ],
            "Missing '{attr}' attribute in '{section}' section",
            section = NODE,
            attr = NODE_KIND,
        }
    }).and_then(|val| {
        val.into_string().ok_or_else(|| {
            miette::miette! {
                labels = vec![
                    LabeledSpan::at(span.clone(), "this attribute"),
                ],
                "Expected string value for '{attr}' attribute in '{section}' section",
                section = NODE,
                attr = NODE_KIND,
            }
        })
    })?;

    let parent = attrs.remove(NODE_PARENT).map(|val| {
        val.into_string().ok_or_else(|| {
            miette::miette! {
                labels = vec![
                    LabeledSpan::at(span.clone(), "this attribute"),
                ],
                "Expected string value for '{attr}' attribute in '{section}' section",
                section = NODE,
                attr = NODE_PARENT,
            }
        })
    }).transpose()?;

    Ok(Node {
        name,
        parent,
        kind: kind.into(),
        // these are yet to be populated by subsequent parsing
        section_keys: Default::default(),
    })
}
