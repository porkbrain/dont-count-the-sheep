mod colon;
mod curly_bracket;
mod float;
mod int;
mod paren;
mod square_bracket;
mod string;
mod string_attribute;

use logos::Logos;

use crate::{
    Animation, ExtResource, ExtResourceAttribute, Fps, NodeAttribute,
    ParseConf, SectionKey, State, SubResource, SubResourceAttribute, X, Y,
};

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
    #[token("{")]
    CurlyBracketOpen,
    #[token("}")]
    CurlyBracketClose,
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
    /// Zero or more node attributes.
    /// Ends with [`TscnToken::SquareBracketClose`].
    NodeAttributes(Vec<NodeAttribute>),
    /// Building a specific section key.
    SectionKey(SectionKeyBuilder),
}

fn parse_with_state(
    conf: &ParseConf,
    state: &mut State,
    expecting: Expecting,
    token: TscnToken,
    s: &str,
) -> Expecting {
    match token {
        ////
        // Headings
        ////
        TscnToken::GdSceneHeading => expecting, // after "[" comes "gd_scene"
        TscnToken::ExtResourceHeading => {
            Expecting::ExtResourceAttributes(Vec::new())
        }
        TscnToken::SubResourceHeading => {
            Expecting::SubResourceAttributes(Vec::new())
        }
        TscnToken::NodeHeading => Expecting::NodeAttributes(Vec::new()),

        ////
        // Structs
        ////
        TscnToken::ExtResourceStruct => {
            // after `ExtResource` comes `(`
            match expecting {
                Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceExpecting::ExtResource,
                )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
                    ExtResourceExpecting::ParenOpen,
                )),
                Expecting::SectionKey(SectionKeyBuilder::Texture(
                    ExtResourceExpecting::ExtResource,
                )) => Expecting::SectionKey(SectionKeyBuilder::Texture(
                    ExtResourceExpecting::ParenOpen,
                )),
                _ => {
                    panic!("Unexpected ExtResourceStruct for {expecting:?}")
                }
            }
        }
        TscnToken::SubResourceStruct => {
            match expecting {
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
                Expecting::SectionKey(SectionKeyBuilder::SpriteFrames(
                    SubResourceExpecting::SubResource,
                )) => Expecting::SectionKey(SectionKeyBuilder::SpriteFrames(
                    SubResourceExpecting::ParenOpen,
                )),
                _ => {
                    panic!("Unexpected SubResourceStruct for {expecting:?}")
                }
            }
        }
        TscnToken::Rect2Struct => {
            match expecting {
                // after `Rect2` comes `(`
                Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2Expecting::Rect2,
                )) => Expecting::SectionKey(SectionKeyBuilder::Region(
                    Rect2Expecting::ParenOpen,
                )),
                _ => {
                    panic!("Unexpected Rect2Struct for {expecting:?}")
                }
            }
        }
        TscnToken::Vector2Struct => {
            match expecting {
                // after `Vector2` comes `(`
                Expecting::SectionKey(SectionKeyBuilder::Position(
                    Vector2Expecting::Vector2,
                )) => Expecting::SectionKey(SectionKeyBuilder::Position(
                    Vector2Expecting::ParenOpen,
                )),
                _ => {
                    panic!("Unexpected Vector2Struct for {expecting:?}")
                }
            }
        }

        ////
        // No-ops
        ////
        TscnToken::SectionKeyAssignment => {
            assert!(matches!(expecting, Expecting::SectionKey(_)));
            expecting
        }
        TscnToken::IntAttribute => {
            // there seem to only be int attributes in the gd_scene heading
            // and we don't really care about them
            assert_eq!(Expecting::GdSceneHeading, expecting);
            expecting
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
            expecting
        }

        ////
        // Attributes
        ////
        TscnToken::StringAttribute
            if Expecting::GdSceneHeading == expecting =>
        {
            // we don't care about any gd_scene attributes
            expecting
        }
        TscnToken::StringAttribute => string_attribute::parse(expecting, s),

        ////
        // Basic types
        ////
        TscnToken::String => string::parse(state, expecting, s),
        TscnToken::Int => int::parse(state, expecting, s),
        TscnToken::Float => float::parse(expecting, s),
        TscnToken::True | TscnToken::False => match expecting {
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
        },

        ////
        // Brackets, quotes, parens, ...
        ////
        TscnToken::SquareBracketOpen => square_bracket::parse_open(expecting),
        TscnToken::SquareBracketClose => {
            square_bracket::parse_close(state, expecting)
        }
        TscnToken::ParenOpen => paren::parse_open(expecting),
        TscnToken::ParenClose => paren::parse_close(state, expecting),
        TscnToken::Colon => colon::parse(expecting),
        TscnToken::CurlyBracketOpen => curly_bracket::parse_open(expecting),
        TscnToken::CurlyBracketClose => curly_bracket::parse_close(expecting),
    }
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
    /// e.g. `z_index = -2`
    ZIndex,
    /// e.g. `texture = ExtResource("3_j8n3v")`
    Texture(ExtResourceExpecting),
    /// e.g. `position = Vector2(-201.5, 49.5)`
    Position(Vector2Expecting),
    /// e.g. `sprite_frames = SubResource("SpriteFrames_33ymd")`
    SpriteFrames(SubResourceExpecting),
    /// e.g. `metadata/zone = "ElevatorZone"`
    /// or   `metadata/label = "Elevator"`
    ///
    /// The string is the key "zone" or "label" etc.
    Metadata(String),
}

/// e.g. `ExtResource("4_oy5kx")`
#[derive(Default, Debug, PartialEq, Eq)]
enum ExtResourceExpecting {
    #[default]
    ExtResource,
    ParenOpen,
    String,
    ParenClose(String),
}

/// e.g. `SubResource("4_oy5kx")`
#[derive(Default, Debug, PartialEq, Eq)]
enum SubResourceExpecting {
    #[default]
    SubResource,
    ParenOpen,
    String,
    ParenClose(String),
}

/// e.g. `Rect2(385, 0, -51, 57)`
#[derive(Default, Debug, PartialEq, Eq)]
enum Rect2Expecting {
    #[default]
    Rect2,
    ParenOpen,
    Int1,
    Int2(i64),
    Int3(i64, i64),
    Int4(i64, i64, i64),
    ParenClose(i64, i64, i64, i64),
}

/// e.g. `Vector2(-201.5, 49.5)`
#[derive(Default, Debug, PartialEq, Eq)]
enum Vector2Expecting {
    #[default]
    Vector2,
    ParenOpen,
    Float1,
    Float2(X),
    ParenClose(X, Y),
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
