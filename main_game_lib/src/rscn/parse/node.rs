use std::ops::Range;

use miette::LabeledSpan;

use super::{tscn_identifiers::NODE, Map, ParsedNode, State, Value};

const NODE_NAME: &str = "name";
/// Optional attribute, will be none for the root node.
const NODE_PARENT: &str = "parent";
const NODE_KIND: &str = "type";

/// Subresources can have section keys.
pub(super) fn parse_attributes(
    state: &mut State,
    span: Range<usize>,
    mut attrs: Map<Value>,
) -> miette::Result<ParsedNode> {
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

    Ok(ParsedNode {
        name,
        parent,
        kind: kind.into(),
        // these are yet to be populated by subsequent parsing
        section_keys: Default::default(),
    })
}
