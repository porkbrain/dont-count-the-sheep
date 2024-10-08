use std::{collections::BTreeMap, ops::Range};

use miette::LabeledSpan;

use crate::parse::{
    tscn_identifiers::EXT_RESOURCE, ExtResource, Scene, SpannedValue,
};

const EXT_RESOURCE_TYPE: &str = "type";
const EXT_RESOURCE_TYPE_TEXTURE_2D: &str = "Texture2D";
const EXT_RESOURCE_ID: &str = "id";
const EXT_RESOURCE_TYPE_TEXTURE_2D_PATH: &str = "path";

/// Ext resources don't have any section keys, therefore we can insert the
/// section in the state directly.
pub(super) fn parse_attributes_into_state(
    state: &mut Scene,
    span: Range<usize>,
    mut attrs: BTreeMap<String, SpannedValue>,
) -> miette::Result<()> {
    let kind = attrs.remove(EXT_RESOURCE_TYPE).ok_or_else(|| {
        miette::miette! {
            labels = vec![
                LabeledSpan::at(span.clone(), "this attribute"),
            ],
            "Missing '{attr}' attribute in '{section}' section",
            section = EXT_RESOURCE,
            attr = EXT_RESOURCE_TYPE,
        }
    }).and_then(|val| {
        val.into_string().ok_or_else(|| {
            miette::miette! {
                labels = vec![
                    LabeledSpan::at(span.clone(), "this attribute"),
                ],
                "Expected string value for '{attr}' attribute in '{section}' section",
                section = EXT_RESOURCE,
                attr = EXT_RESOURCE_TYPE,
            }
        })
    })?;

    let id = attrs.remove(EXT_RESOURCE_ID).ok_or_else(|| {
        miette::miette! {
            labels = vec![
                LabeledSpan::at(span.clone(), "this attribute"),
            ],
            "Missing '{attr}' attribute in '{section}' section",
            section = EXT_RESOURCE,
            attr = EXT_RESOURCE_ID,
        }
    }).and_then(|val| {
        val.into_string().ok_or_else(|| {
            miette::miette! {
                labels = vec![
                    LabeledSpan::at(span.clone(), "this attribute"),
                ],
                "Expected string value for '{attr}' attribute in '{section}' section",
                section = EXT_RESOURCE,
                attr = EXT_RESOURCE_ID,
            }
        })
    })?.into();

    match kind.as_str() {
        EXT_RESOURCE_TYPE_TEXTURE_2D => {
            let path = attrs.remove(EXT_RESOURCE_TYPE_TEXTURE_2D_PATH).ok_or_else(|| {
                miette::miette! {
                    labels = vec![
                        LabeledSpan::at(span.clone(), "this attribute"),
                    ],
                    "Missing '{attr}' attribute in '{section}' section",
                    section = EXT_RESOURCE,
                    attr = EXT_RESOURCE_TYPE_TEXTURE_2D_PATH,
                }
            }).and_then(|val| {
                val.into_string().ok_or_else(|| {
                    miette::miette! {
                        labels = vec![
                            LabeledSpan::at(span.clone(), "this attribute"),
                        ],
                        "Expected string value for '{attr}' attribute in '{section}' section",
                        section = EXT_RESOURCE,
                        attr = EXT_RESOURCE_TYPE_TEXTURE_2D_PATH,
                    }
                })
            })?;

            state
                .ext_resources
                .push(ExtResource::Texture2D { id, path });
        }
        _ => {
            state.ext_resources.push(ExtResource::Other {
                kind,
                id,
                attributes: attrs,
            });
        }
    }

    Ok(())
}
