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
    /// e.g. `metadata/zone = "Elevator"`
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
