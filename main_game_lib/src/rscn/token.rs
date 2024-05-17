mod colon;
mod curly_bracket;
mod number;
mod paren;
mod square_bracket;
mod string;
mod string_attribute;

use logos::Logos;

use crate::rscn::intermediate_repr::*;

pub(crate) fn parse(tscn: &str) -> State {
    let mut lex = TscnToken::lexer(tscn);
    let mut expecting = Expecting::default();
    let mut state = State::default();

    while let Some(token) = lex.next() {
        let Ok(token) = token else {
            panic!("No token for {}", lex.slice());
        };

        expecting = parse_with_state(&mut state, expecting, token, lex.slice());
    }

    state
}

#[derive(Logos, Debug, PartialEq, Eq)]
#[logos(skip r"[\r\t\n\f,]+")]
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
    #[token(" ", priority = 1)]
    Space,

    #[regex(r#"-?\d+(\.\d+)?"#, priority = 3)]
    Number,
    #[regex(r#"[A-Za-z0-9_/]+|"[A-Za-z0-9_/ ]+""#, priority = 2)]
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
    ExtResourceAttributes {
        id: Option<ExtResourceId>,
        kind: Option<ExtResourceKind>,
        path: Option<String>,
    },
    /// Zero or more sub resource attributes.
    /// Ends with [`TscnToken::SquareBracketClose`].
    SubResourceAttributes {
        id: Option<SubResourceId>,
        kind: Option<SubResourceKind>,
    },
    /// Zero or more node attributes.
    /// Ends with [`TscnToken::SquareBracketClose`].
    NodeAttributes {
        name: Option<String>,
        parent: Option<String>,
        kind: Option<ParsedNodeKind>,
    },
    /// Building a specific section key.
    SectionKey(SectionKeyBuilder),
}

fn parse_with_state(
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
        TscnToken::ExtResourceHeading => Expecting::ExtResourceAttributes {
            id: None,   // mandatory
            kind: None, // mandatory
            path: None, // mandatory
        },
        TscnToken::SubResourceHeading => Expecting::SubResourceAttributes {
            id: None,   // mandatory
            kind: None, // mandatory
        },
        TscnToken::NodeHeading => Expecting::NodeAttributes {
            name: None, // mandatory
            kind: None, // mandatory
            // parent can be empty only for the root node
            parent: None,
        },

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
        TscnToken::Space => {
            // since we need to support spaces in strings but can ignore them
            // otherwise, we match on this token and don't do anything
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
        TscnToken::Number => number::parse(state, expecting, s),
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
            Expecting::SectionKey(SectionKeyBuilder::Visibility) => {
                let visible = matches!(token, TscnToken::True);
                state
                    .nodes
                    .last_mut()
                    .unwrap()
                    .section_keys
                    .push(SectionKey::Visibility(visible));

                Expecting::HeadingOrSectionKey
            }
            Expecting::SectionKey(
                dir @ (SectionKeyBuilder::FlipHorizontally
                | SectionKeyBuilder::FlipVertically),
            ) => {
                let visible = matches!(token, TscnToken::True);
                state.nodes.last_mut().unwrap().section_keys.push(
                    if matches!(dir, SectionKeyBuilder::FlipHorizontally) {
                        SectionKey::FlipHorizontally(visible)
                    } else {
                        SectionKey::FlipVertically(visible)
                    },
                );

                Expecting::HeadingOrSectionKey
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
    /// Can be deduped with [`Self::Texture`]
    Atlas(ExtResourceExpecting),
    /// e.g. `texture = ExtResource("3_j8n3v")`
    /// Can be deduped with [`Self::Atlas`]
    Texture(ExtResourceExpecting),

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
    /// e.g. `frame = 2`
    FrameIndex,
    /// e.g. `frame_progress = 0.847`
    FrameProgress,
    /// e.g. `autoplay = "default"` and must always be "default"
    Autoplay,
    /// e.g. `position = Vector2(-201.5, 49.5)`
    Position(Vector2Expecting),
    /// e.g. `sprite_frames = SubResource("SpriteFrames_33ymd")`
    SpriteFrames(SubResourceExpecting),
    /// e.g. `metadata/zone = "ElevatorZone"`
    /// or   `metadata/label = "Elevator"`
    ///
    /// The string is the key "zone" or "label" etc.
    StringMetadata(String),
    /// true or false
    Visibility,
    /// true or false
    FlipHorizontally,
    /// true or false
    FlipVertically,
    /// e.g. `self_modulate = Color(1, 1, 1, 0.823529)`
    SelfModulate(ColorExpecting),
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
    X1,
    Y1(X),
    X2(X, Y),
    Y2(X, Y, X),
    ParenClose(X, Y, X, Y),
}

/// e.g. `Vector2(-201.5, 49.5)`
#[derive(Default, Debug, PartialEq, Eq)]
enum Vector2Expecting {
    #[default]
    Vector2,
    ParenOpen,
    X,
    Y(X),
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

#[derive(Default, Debug, PartialEq, Eq)]
enum ColorExpecting {
    #[default]
    Color,
    ParenOpen,
    R,
    G(Number),
    B(Number, Number),
    A(Number, Number, Number),
    ParenClose(Number, Number, Number, Number),
}
