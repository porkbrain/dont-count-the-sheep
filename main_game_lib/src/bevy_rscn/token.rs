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
    /// e.g. `frame_progress = 0.847`
    FrameProgress,
    /// e.g. `autoplay = "default"` and must always be "default"
    Autoplay,
    /// e.g. `sprite_frames = SubResource("SpriteFrames_33ymd")`
    SpriteFrames(SubResourceExpecting),
}
