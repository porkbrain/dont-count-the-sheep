mod int;
mod paren;
mod square_bracket;
mod string;
mod string_attribute;

use logos::Logos;

use crate::{
    Animation, ExtResource, ExtResourceAttribute, Fps, ParseConf, SectionKey,
    State, SubResource, SubResourceAttribute,
};

#[derive(Logos, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t\n\f,]+")]
pub(crate) enum TscnToken {
    #[token("[")]
    SquareBracketOpen,
    #[token("]")]
    SquareBracketClose,
    #[token("(")]
    ParenOpen,
    #[token(")")]
    ParenClose,
    #[token("{")]
    CurlyBracketOpen,
    #[token("}")]
    CurlyBracketClose,
    // #[token("\"")]
    // Quote,
    #[token(":")]
    Colon,
    #[token("&")]
    Ampersand,

    #[regex(r#"-?\d+"#, priority = 3)]
    Int,
    #[regex(r#"-?\d+\.\d+"#)]
    Float,
    #[regex(r#"[A-Za-z0-9_/]+|"[A-Za-z0-9_/]+""#, priority = 2)]
    String,
    #[token("true")]
    True,
    #[token("false")]
    False,

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

        expecting =
            parse_with_state(&conf, &mut state, expecting, token, lex.slice());
    }
}

fn parse_with_state(
    conf: &ParseConf,
    state: &mut State,
    mut expecting: Expecting,
    token: TscnToken,
    s: &str,
) -> Expecting {
    match token {
        ////
        // Headings
        ////
        TscnToken::GdSceneHeading => {} // after "[" comes "gd_scene"
        TscnToken::ExtResourceHeading => {
            expecting = Expecting::ExtResourceAttributes(Vec::new());
        }
        TscnToken::SubResourceHeading => {
            expecting = Expecting::SubResourceAttributes(Vec::new());
        }

        ////
        // Structs
        ////
        TscnToken::ExtResourceStruct => {
            expecting = match expecting {
                // after `ExtResource` comes `(`
                Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceExpecting::ExtResource,
                )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceExpecting::ParenOpen,
                )),
                _ => {
                    panic!("Unexpected ExtResourceStruct for {expecting:?}")
                }
            };
        }
        TscnToken::SubResourceStruct => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting:
                        SingleAnimExpecting::FrameNextParamValue(with_param),
                }) if with_param == "texture" => {
                    // just forward to the next token
                    Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                        state,
                        expecting: SingleAnimExpecting::FrameNextParamValue(
                            with_param,
                        ),
                    })
                }
                _ => {
                    panic!("Unexpected SubResourceStruct for {expecting:?}")
                }
            };
        }
        TscnToken::Rect2Struct => {
            expecting = match expecting {
                // after `Rect2` comes `(`
                Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2Expecting::Rect2,
                )) => Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2Expecting::ParenOpen,
                )),
                _ => {
                    panic!("Unexpected Rect2Struct for {expecting:?}")
                }
            };
        }

        ////
        // No-ops
        ////
        TscnToken::SectionKeyAssignment => {
            assert!(matches!(expecting, Expecting::SectionKey(_)));
        }
        TscnToken::IntAttribute => {
            // there seem to only be int attributes in the gd_scene heading
            // and we don't really care about them
            assert_eq!(Expecting::GdSceneHeading, expecting);
        }
        TscnToken::Ampersand => {
            // godot's weird notation for strings
            assert!(matches!(
                &expecting,
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    expecting: SingleAnimExpecting::NextParamValue(with_param),
                    ..
                }) if with_param == "name"
            ));
        }

        ////
        // Attributes
        ////
        TscnToken::StringAttribute
            if Expecting::GdSceneHeading == expecting =>
        {
            // we don't care about any gd_scene attributes
        }
        TscnToken::StringAttribute => {
            expecting = string_attribute::parse(expecting, s)
        }

        ////
        // Basic types
        ////
        TscnToken::String => expecting = string::parse(expecting, s),
        TscnToken::Int => expecting = int::parse(expecting, s),
        TscnToken::Float => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting:
                        SingleAnimExpecting::FrameNextParamValue(with_param),
                }) if with_param == "duration" => {
                    assert_eq!(
                        "1.0", s,
                        "we only support evenly spaced frames"
                    );
                    Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                        state,
                        expecting: SingleAnimExpecting::FrameNextParamOrDone,
                    })
                }
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    mut state,
                    expecting: SingleAnimExpecting::NextParamValue(with_param),
                }) if with_param == "speed" => {
                    state.speed = Fps(s.parse().unwrap());
                    Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                        state,
                        expecting: SingleAnimExpecting::ReadNextParamOrDone,
                    })
                }
                _ => panic!("Unexpected float for {expecting:?}"),
            }
        }
        TscnToken::True | TscnToken::False => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    mut state,
                    expecting: SingleAnimExpecting::NextParamValue(with_param),
                }) if with_param == "loop" => {
                    state.loop_ = matches!(token, TscnToken::True);
                    Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                        state,
                        expecting: SingleAnimExpecting::ReadNextParamOrDone,
                    })
                }
                _ => panic!("Unexpected bool for {expecting:?}"),
            }
        }

        ////
        // Brackets, quotes, parens, ...
        ////
        TscnToken::SquareBracketOpen => {
            expecting = square_bracket::parse_open(expecting)
        }
        TscnToken::SquareBracketClose => {
            expecting = square_bracket::parse_close(state, expecting)
        }
        TscnToken::ParenOpen => expecting = paren::parse_open(expecting),
        TscnToken::ParenClose => {
            expecting = paren::parse_close(state, expecting)
        }
        // TscnToken::Quote => expecting = quote::parse(expecting),
        TscnToken::Colon => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting: SingleAnimExpecting::NextParamColon(with_param),
                }) if with_param == "frames" => {
                    Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                        state,
                        expecting:
                            SingleAnimExpecting::FramesStartSquareBracket,
                    })
                }
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting: SingleAnimExpecting::NextParamColon(with_param),
                }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting: SingleAnimExpecting::NextParamValue(with_param),
                }),
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting:
                        SingleAnimExpecting::FrameNextParamColon(with_param),
                }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting: SingleAnimExpecting::FrameNextParamValue(
                        with_param,
                    ),
                }),
                _ => panic!("Unexpected colon for {expecting:?}"),
            }
        }

        TscnToken::CurlyBracketOpen => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting: SingleAnimExpecting::StartCurlyBracket,
                }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting: SingleAnimExpecting::ReadNextParamOrDone,
                }),
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting: SingleAnimExpecting::FrameStartCurlyBracketOrDone,
                }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting: SingleAnimExpecting::FrameNextParamOrDone,
                }),
                _ => panic!("Unexpected curly bracket open for {expecting:?}"),
            }
        }
        TscnToken::CurlyBracketClose => {
            expecting = match expecting {
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting: SingleAnimExpecting::FrameNextParamOrDone,
                }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting:
                        SingleAnimExpecting::FrameStartCurlyBracketOrDone,
                }),
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting: SingleAnimExpecting::ReadNextParamOrDone,
                }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state,
                    expecting: SingleAnimExpecting::EndSquareBracket,
                }),
                _ => panic!("Unexpected curly bracket close for {expecting:?}"),
            }
        }

        ////
        // TODO: This should be unreachable
        ////
        token => {
            panic!("{token:?} => {s} ({}), expecting {expecting:?}", s.len());
        }
    }

    expecting
}

#[derive(Debug, PartialEq, Eq)]
enum SectionKeyBuilder {
    /// e.g. `atlas = ExtResource("4_oy5kx")`
    Atlas(ExtResourceExpecting),
    /// e.g. `region = Rect2(385, 0, -51, 57)`
    Region(Rect2Expecting),
    /// e.g.
    /// ```text
    /// animations = [{
    /// "frames": [{
    /// "duration": 1.0,
    /// "texture": SubResource("AtlasTexture_n0t2h")
    /// }, {
    /// "duration": 1.0,
    /// "texture": SubResource("AtlasTexture_s6ur5")
    /// }, {
    /// "duration": 1.0,
    /// "texture": SubResource("AtlasTexture_2slx6")
    /// }],
    /// "loop": true,
    /// "name": &"default",
    /// "speed": 5.0
    /// }]
    /// ```
    SingleAnim {
        state: Animation,
        expecting: SingleAnimExpecting,
    },
}

/// e.g. `ExtResource("4_oy5kx")`
#[derive(Debug, PartialEq, Eq)]
enum ExtResourceExpecting {
    ExtResource,
    ParenOpen,
    String,
    ParenClose(String),
}

/// e.g. `Rect2(385, 0, -51, 57)`
#[derive(Debug, PartialEq, Eq)]
enum Rect2Expecting {
    Rect2,
    ParenOpen,
    Int1,
    Int2(i64),
    Int3(i64, i64),
    Int4(i64, i64, i64),
    ParenClose(i64, i64, i64, i64),
}

/// This should be recursive if ever we need to refactor.
#[derive(Default, Debug, PartialEq, Eq)]
enum SingleAnimExpecting {
    #[default]
    StartSquareBracket,
    StartCurlyBracket,

    ReadNextParamOrDone,
    NextParamColon(String), // the param in question
    NextParamValue(String), // the param in question

    FramesStartSquareBracket,
    FrameStartCurlyBracketOrDone,
    FrameNextParamOrDone,
    FrameNextParamColon(String), // the param in question
    FrameNextParamValue(String), // the param in question

    EndSquareBracket,
}
