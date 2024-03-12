use logos::Logos;

#[derive(Default)]
pub struct ParseConf {}

pub fn parse(tscn: &str) {
    parse_with_conf(tscn, Default::default())
}

pub fn parse_with_conf(tscn: &str, conf: ParseConf) {
    let mut lex = TscnToken::lexer(tscn);
    let mut expecting = Expecting::default();
    let mut state = State::default();

    while let Some(token) = lex.next() {
        let Ok(token) = token else {
            panic!("No token for {}", lex.slice());
        };

        expecting = parse_token_with_state(
            &conf,
            &mut state,
            expecting,
            token,
            lex.slice(),
        );
    }
}

fn parse_token_with_state(
    conf: &ParseConf,
    state: &mut State,
    mut expecting: Expecting,
    token: TscnToken,
    s: &str,
) -> Expecting {
    match token {
        TscnToken::GdSceneHeading => {} // after "[" comes "gd_scene"
        TscnToken::ExtResourceHeading => {
            expecting = Expecting::ExtResourceAttributes(Vec::new());
        }
        TscnToken::SubResourceHeading => {
            expecting = Expecting::SubResourceAttributes(Vec::new());
        }
        TscnToken::ExtResourceStruct => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceBuilderExpecting::ExtResource,
                )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceBuilderExpecting::ParenOpen,
                )),
                _ => {
                    panic!("Unexpected ExtResourceStruct for {expecting:?}")
                }
            };
        }
        TscnToken::Rect2Struct => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::Rect2,
                )) => Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::ParenOpen,
                )),
                _ => {
                    panic!("Unexpected Rect2Struct for {expecting:?}")
                }
            };
        }
        TscnToken::IntAttribute => {
            // there seem to only be int attributes in the gd_scene heading
            // and we don't really care about them
            assert_eq!(Expecting::GdSceneHeading, expecting);
        }
        TscnToken::StringAttribute
            if Expecting::GdSceneHeading == expecting =>
        {
            // we don't care about any gd_scene attributes
        }
        TscnToken::StringAttribute => {
            let mut split = s.split('=');
            let key = split.next().expect("non empty key");
            let value = split.next().expect("non empty value");
            let value = &value[1..value.len() - 1]; // remove quotes

            match expecting {
                Expecting::ExtResourceAttributes(ref mut attrs) => {
                    let attr = match (key, value) {
                        ("type", "Texture2D") => {
                            ExtResourceAttribute::TypeTexture2D
                        }
                        ("uid", _) => {
                            ExtResourceAttribute::Uid(value.to_string())
                        }
                        ("path", _) => {
                            ExtResourceAttribute::Path(value.to_string())
                        }
                        ("id", _) => {
                            ExtResourceAttribute::Id(value.to_string())
                        }
                        _ => {
                            panic!("Unknown ExtResourceAttribute {key}={value}")
                        }
                    };
                    attrs.push(attr);
                }
                Expecting::SubResourceAttributes(ref mut attrs) => {
                    let attr = match (key, value) {
                        ("type", "AtlasTexture") => {
                            SubResourceAttribute::TypeAtlasTexture
                        }
                        ("type", "SpriteFrames") => {
                            SubResourceAttribute::TypeSpriteFrames
                        }
                        ("id", _) => {
                            SubResourceAttribute::Id(value.to_string())
                        }
                        _ => {
                            panic!("Unknown SubResourceAttribute {key}={value}")
                        }
                    };
                    attrs.push(attr);
                }
                _ => {
                    panic!("Unexpected string attribute for {expecting:?}")
                }
            }
        }
        TscnToken::String => {
            expecting = match expecting {
                Expecting::HeadingOrSectionKey => match s {
                    "atlas" => Expecting::SectionKey(SectionKeyBuilder::Atlas(
                        ExtResourceBuilderExpecting::ExtResource,
                    )),
                    "region" => Expecting::SectionKey(
                        SectionKeyBuilder::Region(Rect2BuilderExpecting::Rect2),
                    ),
                    _ => {
                        panic!("Unknown section key: {s}")
                    }
                },
                Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceBuilderExpecting::String,
                )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceBuilderExpecting::EndQuote(s.to_string()),
                )),
                _ => {
                    panic!("Unexpected string {s} for {expecting:?}")
                }
            }
        }
        TscnToken::Int => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::Int1,
                )) => Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::Int2(s.parse().unwrap()),
                )),
                Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::Int2(int1),
                )) => Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::Int3(int1, s.parse().unwrap()),
                )),
                Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::Int3(int1, int2),
                )) => Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::Int4(int1, int2, s.parse().unwrap()),
                )),
                Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::Int4(int1, int2, int3),
                )) => Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::ParenClose(
                        int1,
                        int2,
                        int3,
                        s.parse().unwrap(),
                    ),
                )),
                _ => {
                    panic!("Unexpected int {s} for {expecting:?}")
                }
            }
        }
        TscnToken::SectionKeyAssignment => {
            assert!(matches!(expecting, Expecting::SectionKey(_)));
        }
        TscnToken::SquareBracketOpen => match expecting {
            Expecting::Heading | Expecting::GdSceneHeading => {} /* starts with "[" */
            Expecting::HeadingOrSectionKey => {
                expecting = Expecting::Heading;
            }
            _ => panic!("Unexpected square bracket open for {expecting:?}"),
        },
        TscnToken::SquareBracketClose => match expecting {
            Expecting::GdSceneHeading => {
                expecting = Expecting::Heading;
            }
            Expecting::ExtResourceAttributes(attrs) => {
                state.ext_resources.push(ExtResource { attrs });
                // no section keys for ext resources
                expecting = Expecting::Heading;
            }
            Expecting::SubResourceAttributes(attrs) => {
                state.sub_resources.push(SubResource {
                    attrs,
                    section_keys: Vec::new(),
                });
                // supports section keys such as atlas, region or animations
                expecting = Expecting::HeadingOrSectionKey;
            }
            _ => {
                panic!("Unexpected square bracket close for {expecting:?}")
            }
        },
        TscnToken::ParenOpen => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceBuilderExpecting::ParenOpen,
                )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceBuilderExpecting::StartQuote,
                )),
                Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::ParenOpen,
                )) => Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::Int1,
                )),
                _ => {
                    panic!("Unexpected paren open for {expecting:?}")
                }
            }
        }
        TscnToken::ParenClose => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceBuilderExpecting::ParenClose(with_str),
                )) => {
                    state
                        .sub_resources
                        .last_mut()
                        .expect("sub resource to come before section key")
                        .section_keys
                        .push(SectionKey::AtlasExtResource(with_str));
                    Expecting::HeadingOrSectionKey
                }
                Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2BuilderExpecting::ParenClose(int1, int2, int3, int4),
                )) => {
                    state
                        .sub_resources
                        .last_mut()
                        .expect("sub resource to come before section key")
                        .section_keys
                        .push(SectionKey::RegionRect2(int1, int2, int3, int4));
                    Expecting::HeadingOrSectionKey
                }
                _ => {
                    panic!("Unexpected paren close for {expecting:?}")
                }
            }
        }
        TscnToken::Quote => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceBuilderExpecting::StartQuote,
                )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceBuilderExpecting::String,
                )),
                Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceBuilderExpecting::EndQuote(with_str),
                )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceBuilderExpecting::ParenClose(with_str),
                )),
                _ => {
                    panic!("Unexpected quote for {expecting:?}")
                }
            }
        }
        token => {
            panic!("{token:?} => {s} ({}), expecting {expecting:?}", s.len());
        }
    }

    expecting
}

#[derive(Default)]
struct State {
    ext_resources: Vec<ExtResource>,
    sub_resources: Vec<SubResource>,
    nodes: Vec<()>,
}

#[derive(Debug, PartialEq, Eq)]
struct ExtResource {
    attrs: Vec<ExtResourceAttribute>,
}

#[derive(Debug, PartialEq, Eq)]
struct SubResource {
    attrs: Vec<SubResourceAttribute>,
    section_keys: Vec<SectionKey>,
}

#[derive(Logos, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t\n\f,]+")]
enum TscnToken {
    #[token("[")]
    SquareBracketOpen,
    #[token("]")]
    SquareBracketClose,
    #[token("(")]
    ParenOpen,
    #[token(")")]
    ParenClose,
    #[token("\"")]
    Quote,

    #[regex(r#"-?\d+"#, priority = 3)]
    Int,
    #[regex(r#"-?\d+\.\d+"#)]
    Float,
    #[regex(r#"[A-Za-z0-9_/]+"#, priority = 2)]
    String,

    #[token("gd_scene", priority = 999999)]
    GdSceneHeading,
    #[token("ext_resource")]
    ExtResourceHeading,
    #[token("sub_resource")]
    SubResourceHeading,
    #[token("node")]
    NodeHeading,

    #[token("ExtResource")]
    ExtResourceStruct,
    #[token("Rect2")]
    Rect2Struct,
    #[token("Vector2")]
    Vector2Struct,
    #[token("SubResource")]
    SubResourceStruct,

    /// e.g. `uid="uid://dyrtqlwb1xtvf"`
    #[regex(r#"[a-z_]+="[^"]*""#)]
    StringAttribute,
    /// e.g. `load_steps=14`
    #[regex("[a-z_]+=-?[0-9]+")]
    IntAttribute,
    #[token(" = ")]
    SectionKeyAssignment,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum Expecting {
    /// The file must start with a `gd_scene` heading and it must be present
    /// only once.
    #[default]
    GdSceneHeading,
    /// The next thing must be a heading (except for `gd_scene`)
    Heading,
    /// The next thing is either a new heading (except for `gd_scene`) or a
    /// section key.
    HeadingOrSectionKey,
    /// Zero or more ext resource attributes.
    /// Ends with [`TscnToken::SquareBracketClose`].
    ExtResourceAttributes(Vec<ExtResourceAttribute>),
    /// Zero or more sub resource attributes.
    /// Ends with [`TscnToken::SquareBracketClose`].
    SubResourceAttributes(Vec<SubResourceAttribute>),
    /// Building a specific section key.
    SectionKey(SectionKeyBuilder),
}

#[derive(Debug, PartialEq, Eq)]
enum ExtResourceAttribute {
    TypeTexture2D,
    Uid(String),
    Path(String),
    Id(String),
}

#[derive(Debug, PartialEq, Eq)]
enum SubResourceAttribute {
    TypeAtlasTexture,
    TypeSpriteFrames,
    Id(String),
}

#[derive(Debug, PartialEq, Eq)]
enum SectionKey {
    AtlasExtResource(String),
    RegionRect2(i64, i64, i64, i64),
}

#[derive(Debug, PartialEq, Eq)]
enum SectionKeyBuilder {
    /// e.g. `atlas = ExtResource("4_oy5kx")`
    Atlas(ExtResourceBuilderExpecting),
    /// e.g. `region = Rect2(385, 0, -51, 57)`
    Region(Rect2BuilderExpecting),
}

/// e.g. `ExtResource("4_oy5kx")`
#[derive(Debug, PartialEq, Eq)]
enum ExtResourceBuilderExpecting {
    ExtResource,
    ParenOpen,
    StartQuote,
    String,
    EndQuote(String),
    ParenClose(String),
}

/// e.g. `Rect2(385, 0, -51, 57)`
#[derive(Debug, PartialEq, Eq)]
enum Rect2BuilderExpecting {
    Rect2,
    ParenOpen,
    Int1,
    Int2(i64),
    Int3(i64, i64),
    Int4(i64, i64, i64),
    ParenClose(i64, i64, i64, i64),
}
