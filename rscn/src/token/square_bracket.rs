use super::*;

pub(super) fn parse_open(expecting: Expecting) -> Expecting {
    match expecting {
        // starts with "[", we're already fine
        Expecting::Heading | Expecting::GdSceneHeading => expecting,
        Expecting::HeadingOrSectionKey => Expecting::Heading,
        _ => panic!("Unexpected square bracket open for {expecting:?}"),
    }
}

pub(super) fn parse_close(
    state: &mut State,
    expecting: Expecting,
) -> Expecting {
    match expecting {
        Expecting::GdSceneHeading => Expecting::Heading,
        Expecting::ExtResourceAttributes(attrs) => {
            state.ext_resources.push(ExtResource { attrs });
            // no section keys for ext resources
            Expecting::Heading
        }
        Expecting::SubResourceAttributes(attrs) => {
            state.sub_resources.push(SubResource {
                attrs,
                section_keys: Vec::new(),
            });
            // supports section keys such as atlas, region or animations
            Expecting::HeadingOrSectionKey
        }
        _ => {
            panic!("Unexpected square bracket close for {expecting:?}")
        }
    }
}
