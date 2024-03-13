use super::*;

pub(super) fn parse_open(expecting: Expecting) -> Expecting {
    match expecting {
        // starts with "[", we're already fine
        Expecting::Heading | Expecting::GdSceneHeading => expecting,
        Expecting::HeadingOrSectionKey => Expecting::Heading,

        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::StartSquareBracket,
        }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::StartCurlyBracket,
        }),
        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FramesStartSquareBracket,
        }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameStartCurlyBracketOrDone,
        }),

        _ => panic!("Unexpected square bracket open for {expecting:?}"),
    }
}

pub(super) fn parse_close(
    state: &mut State,
    expecting: Expecting,
) -> Expecting {
    match expecting {
        Expecting::GdSceneHeading => Expecting::Heading,
        Expecting::ExtResourceAttributes { kind, id, path } => {
            state.ext_resources.push(ParsedExtResource {
                kind: kind
                    .expect("ExtResource 'type' attribute should be present"),
                id: id.expect("ExtResource 'id' attribute should be present"),
                path: path
                    .expect("ExtResource 'path' attribute should be present"),
            });
            // no section keys for ext resources
            Expecting::Heading
        }
        Expecting::SubResourceAttributes { id, kind } => {
            state.sub_resources.push(ParsedSubResource {
                id: id.expect("SubResource 'id' attribute should be present"),
                kind: kind
                    .expect("SubResource 'type' attribute should be present"),
                section_keys: Vec::new(),
            });
            // supports section keys such as atlas, region or animations
            Expecting::HeadingOrSectionKey
        }
        Expecting::NodeAttributes { name, kind, parent } => {
            state.nodes.push(ParsedNode {
                name: name.expect("Node 'name' attribute should be present"),
                kind: kind.expect("Node 'type' attribute should be present"),
                parent,
                section_keys: Vec::new(),
            });
            // supports section keys such as z_index, texture, position,
            // sprite_frames or metadata/WHATEVER
            Expecting::HeadingOrSectionKey
        }

        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameStartCurlyBracketOrDone,
        }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::ReadNextParamOrDone,
        }),
        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state: animation,
            expecting: SingleAnimExpecting::EndSquareBracket,
        }) => {
            state
                .sub_resources
                .last_mut()
                .expect("sub resource to come before section key")
                .section_keys
                .push(SectionKey::SingleAnim(animation));
            Expecting::HeadingOrSectionKey
        }
        _ => {
            panic!("Unexpected square bracket close for {expecting:?}")
        }
    }
}
