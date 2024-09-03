use bevy::{
    color::Color,
    math::{Rect, Vec2},
    utils::{default, HashMap},
};

use crate::rscn::{
    intermediate_repr::{self, ParsedNodeKind, Y},
    Config, In2D, RscnNode, NodeName, SpriteFrames, SpriteTexture, TscnTree,
};

struct Properties {
    z_index: Option<f32>,
    position: Vec2,
    metadata: HashMap<String, String>,
    path: Option<String>,
    animation: Option<SpriteFrames>,
    visible: bool,
    color: Option<Color>,
    flip_horizontally: bool,
    flip_vertically: bool,
}

pub(crate) fn from_state(
    mut state: intermediate_repr::State,
    conf: &Config,
) -> TscnTree {
    let root_node_index = state
        .nodes
        .iter()
        .position(|node| node.parent.is_none())
        .expect("there should be a node with no parent");
    let parsed_root_node = state.nodes.remove(root_node_index);
    assert!(
        parsed_root_node.section_keys.is_empty(),
        "Root node must have no extra data"
    );
    assert_eq!(
        ParsedNodeKind::Node2D,
        parsed_root_node.kind,
        "Root node must be of type Node2D"
    );

    let root_node_name = parsed_root_node.name;
    let mut root = RscnNode {
        in_2d: Some(In2D {
            position: Vec2::ZERO,
            z_index: None,
            texture: None,
        }),
        metadata: default(),
        children: default(),
    };

    // sort the nodes by their parent path:
    // "." is 1, "JustAName" is 2, and each "/" in the string adds 1 to the
    // so that e.g. "JustAName/Child" is 3
    state.nodes.sort_by_key(|node| {
        let p = node
            .parent
            .as_ref()
            .expect("each node except for root should have a parent");

        if p == "." {
            1
        } else {
            2 + p.chars().filter(|c| *c == '/').count()
        }
    });

    // now that the nodes are sorted we can iterate over them and we will be
    // guaranteed that a parent is always added before its children

    let mut nodes = vec![];
    std::mem::swap(&mut nodes, &mut state.nodes); // to avoid borrow checker
    for parsed_node in nodes {
        let mut properties = default();

        for section_key in parsed_node.section_keys {
            apply_section_key(conf, &state, section_key, &mut properties);
        }

        let Properties {
            z_index,
            position,
            metadata,
            path,
            animation,
            visible,
            color,
            flip_horizontally,
            flip_vertically,
        } = properties;

        let in_2d = match parsed_node.kind {
            ParsedNodeKind::AnimatedSprite2D => Some(In2D {
                position,
                z_index,
                texture: Some(SpriteTexture {
                    path: path.unwrap_or_else(|| {
                        panic!(
                            "Node '{}': AnimatedSprite2D should have a texture",
                            parsed_node.name
                        )
                    }),
                    visible,
                    color,
                    animation: {
                        assert!(animation.is_some());
                        animation
                    },
                    flip_horizontally,
                    flip_vertically,
                }),
            }),
            ParsedNodeKind::Sprite2D => Some(In2D {
                position,
                z_index,
                texture: Some(SpriteTexture {
                    path: path.unwrap_or_else(|| {
                        panic!(
                            "Node '{}': Sprite2D should have a texture",
                            parsed_node.name
                        )
                    }),
                    visible,
                    color,
                    animation: {
                        assert!(animation.is_none());
                        None
                    },
                    flip_horizontally,
                    flip_vertically,
                }),
            }),
            ParsedNodeKind::Node2D => Some(In2D {
                position,
                z_index,
                texture: {
                    assert!(path.is_none());
                    assert!(animation.is_none());
                    None
                },
            }),
            ParsedNodeKind::Node => {
                assert_eq!(Vec2::ZERO, position);
                assert!(z_index.is_none());
                assert!(path.is_none());
                assert!(animation.is_none());
                None
            }
        };

        let node = RscnNode {
            metadata,
            in_2d,
            children: default(),
        };

        let parent = parsed_node
            .parent
            .expect("each node except for root should have a parent");

        if parent == "." {
            root.children.insert(NodeName(parsed_node.name), node);
        } else {
            let mut current_parent = &mut root;

            for fragment in parent.split('/') {
                current_parent = current_parent
                    .children
                    .get_mut(&NodeName(fragment.to_string()))
                    .expect("node path should point to a valid parent node");
            }

            current_parent
                .children
                .insert(NodeName(parsed_node.name), node);
        }
    }

    TscnTree {
        root,
        root_node_name: NodeName(root_node_name),
    }
}

fn apply_section_key(
    conf: &Config,
    state: &intermediate_repr::State,
    section_key: intermediate_repr::SectionKey,
    Properties {
        z_index,
        position,
        metadata,
        path,
        animation,
        visible,
        color,
        flip_horizontally,
        flip_vertically,
    }: &mut Properties,
) {
    use intermediate_repr::{Number, SectionKey, X};

    match section_key {
        SectionKey::RegionRect2(..) => {
            panic!("Node should not have a region section key")
        }
        SectionKey::SingleAnim(..) => {
            panic!("Node should not have an animation section key")
        }
        SectionKey::AtlasExtResource(..) => {
            panic!("Node should not have an atlas section key")
        }

        SectionKey::SelfModulateColor(
            Number(r),
            Number(g),
            Number(b),
            Number(a),
        ) => {
            assert!(
                color.replace(Color::srgba(r, g, b, a)).is_none(),
                "Node should not have more than one color"
            );
        }
        SectionKey::FlipHorizontally(flip) => {
            *flip_horizontally = flip;
        }
        SectionKey::FlipVertically(flip) => {
            *flip_vertically = flip;
        }
        SectionKey::Visibility(visibility) => {
            *visible = visibility;
        }
        SectionKey::ZIndex(Number(z)) => {
            assert!(
                z_index.replace(z).is_none(),
                "Node should not have more than one z_index"
            );
        }
        SectionKey::Position(X(x), y) => {
            *position = Vec2::new(x, y.into_bevy_position_coords());
        }
        SectionKey::StringMetadata(key, value) => {
            assert!(
                metadata.insert(key, value).is_none(),
                "Node should not have more than \
                one metadata with the same key"
            );
        }

        SectionKey::TextureExtResource(id) => {
            let prefixless_path = state
                .ext_resources
                .iter()
                .find(|res| res.id == id)
                .map(|res| conf.to_prefixless_path(&res.path))
                .expect("ext resource should exist");
            assert!(
                path.replace(prefixless_path).is_none(),
                "Node should not have more than one texture"
            );
        }
        SectionKey::FrameIndex(index) => {
            animation
                .as_mut()
                .expect("Frame index always comes after sprite_frames")
                .first_index = index;
        }
        SectionKey::Autoplay => {
            animation
                .as_mut()
                .expect("Autoplay always comes after sprite_frames")
                .should_autoload = true;
        }
        SectionKey::SpriteFramesSubResource(id) => {
            let res = state
                .sub_resources
                .iter()
                .find(|res| res.id == id)
                .expect("sub resource should exist");
            assert_eq!(
                1,
                res.section_keys.len(),
                "SpriteFrames should have exactly one animation"
            );

            let SectionKey::SingleAnim(anim) = &res.section_keys[0] else {
                panic!(
                    "SpriteFrames should have \
                            exactly one SingleAnim section key"
                )
            };

            let mut max_y = 0.0f32;
            let mut max_x = 0.0f32;
            let frames: Vec<_> = anim
                .frames
                .iter()
                .map(|frame| {
                    let frame = state
                        .sub_resources
                        .iter()
                        .find(|res| res.id == frame.texture)
                        .expect("sub resource should exist");
                    assert_eq!(2, frame.section_keys.len());

                    let prefixless_path = frame
                        .section_keys
                        .iter()
                        .find_map(|section_key| {
                            let SectionKey::AtlasExtResource(id) = section_key
                            else {
                                return None;
                            };

                            let prefixless_path = state
                                .ext_resources
                                .iter()
                                .find(|res| res.id == *id)
                                .map(|res| conf.to_prefixless_path(&res.path))
                                .expect("ext resource should exist");

                            Some(prefixless_path)
                        })
                        .expect(
                            "sub resource should have an atlas section key",
                        );

                    if let Some(path) = path {
                        assert_eq!(path, &prefixless_path);
                    } else {
                        *path = Some(prefixless_path);
                    }

                    let rect = frame
                        .section_keys
                        .iter()
                        .find_map(|section_key| {
                            // we don't convert into bevy coords here because
                            // bevy uses the top-left corner as the origin
                            // for textures
                            let SectionKey::RegionRect2(
                                X(x1),
                                Y(y1),
                                X(w),
                                Y(h),
                            ) = section_key
                            else {
                                return None;
                            };

                            Some(Rect {
                                min: Vec2::new(*x1, *y1),
                                max: Vec2::new(*x1 + *w, *y1 + *h),
                            })
                        })
                        .expect(
                            "sub resource should have a region section key",
                        );

                    max_x = max_x.max(rect.max.x);
                    max_y = max_y.max(rect.max.y);

                    rect
                })
                .collect();
            assert!(frames.len() > anim.index as usize);

            assert!(animation
                .replace(SpriteFrames {
                    should_endless_loop: anim.loop_,
                    fps: anim.speed.into(),
                    should_autoload: anim.autoload,
                    first_index: anim.index as usize,
                    frames,
                    size: Vec2::new(max_x, max_y),
                })
                .is_none());
        }
    }
}

impl Config {
    fn to_prefixless_path(&self, godot_path: &str) -> String {
        assert!(godot_path.starts_with(&self.asset_path_prefix));
        godot_path[self.asset_path_prefix.len()..].to_owned()
    }
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            z_index: None,
            position: Vec2::ZERO,
            metadata: default(),
            path: None,
            animation: None,
            visible: true,
            color: None,
            flip_horizontally: false,
            flip_vertically: false,
        }
    }
}
