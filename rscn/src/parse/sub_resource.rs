use std::ops::Range;

use miette::LabeledSpan;

use crate::parse::{tscn_identifiers::SUB_RESOURCE, Map, SubResource, Value};

const SUB_RESOURCE_TYPE: &str = "type";
const SUB_RESOURCE_ID: &str = "id";

/// Subresources can have section keys.
pub(super) fn parse_attributes(
    span: Range<usize>,
    mut attrs: Map<String, Value>,
) -> miette::Result<SubResource> {
    let kind = attrs.remove(SUB_RESOURCE_TYPE).ok_or_else(|| {
        miette::miette! {
            labels = vec![
                LabeledSpan::at(span.clone(), "this attribute"),
            ],
            "Missing '{attr}' attribute in '{section}' section",
            section = SUB_RESOURCE,
            attr = SUB_RESOURCE_TYPE,
        }
    }).and_then(|val| {
        val.into_string().ok_or_else(|| {
            miette::miette! {
                labels = vec![
                    LabeledSpan::at(span.clone(), "this attribute"),
                ],
                "Expected string value for '{attr}' attribute in '{section}' section",
                section = SUB_RESOURCE,
                attr = SUB_RESOURCE_TYPE,
            }
        })
    })?;

    let id = attrs.remove(SUB_RESOURCE_ID).ok_or_else(|| {
        miette::miette! {
            labels = vec![
                LabeledSpan::at(span.clone(), "this attribute"),
            ],
            "Missing '{attr}' attribute in '{section}' section",
            section = SUB_RESOURCE,
            attr = SUB_RESOURCE_ID,
        }
    }).and_then(|val| {
        val.into_string().ok_or_else(|| {
            miette::miette! {
                labels = vec![
                    LabeledSpan::at(span.clone(), "this attribute"),
                ],
                "Expected string value for '{attr}' attribute in '{section}' section",
                section = SUB_RESOURCE,
                attr = SUB_RESOURCE_ID,
            }
        })
    })?;

    Ok(SubResource {
        id: id.into(),
        kind: kind.into(),
        // these are yet to be populated by subsequent parsing
        section_keys: Default::default(),
    })
}
