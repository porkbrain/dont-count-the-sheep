use std::ops::Range;

use miette::LabeledSpan;

use crate::parse::{
    tscn_identifiers::EXT_RESOURCE, ExtResource, Map, Scene, Value,
};

const EXT_RESOURCE_TYPE: &str = "type";
const EXT_RESOURCE_TYPE_TEXTURE_2D: &str = "Texture2D";
const EXT_RESOURCE_UID: &str = "uid";
const EXT_RESOURCE_TYPE_TEXTURE_2D_PATH: &str = "path";

/// Ext resources don't have any section keys, therefore we can insert the
/// section in the state directly.
pub(super) fn parse_attributes_into_state(
    state: &mut Scene,
    span: Range<usize>,
    mut attrs: Map<Value>,
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

    let uid = attrs.remove(EXT_RESOURCE_UID).ok_or_else(|| {
        miette::miette! {
            labels = vec![
                LabeledSpan::at(span.clone(), "this attribute"),
            ],
            "Missing '{attr}' attribute in '{section}' section",
            section = EXT_RESOURCE,
            attr = EXT_RESOURCE_UID,
        }
    }).and_then(|val| {
        val.into_string().ok_or_else(|| {
            miette::miette! {
                labels = vec![
                    LabeledSpan::at(span.clone(), "this attribute"),
                ],
                "Expected string value for '{attr}' attribute in '{section}' section",
                section = EXT_RESOURCE,
                attr = EXT_RESOURCE_UID,
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
                .push(ExtResource::Texture2D { uid, path });
        }
        _ => {
            state.ext_resources.push(ExtResource::Other {
                kind,
                uid,
                attributes: attrs,
            });
        }
    }

    Ok(())
}
