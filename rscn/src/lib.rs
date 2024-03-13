#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod intermediate_repr;
mod spawner;
mod token;

use std::borrow::Cow;

use bevy::{
    math::{Rect, Vec2},
    utils::{default, HashMap},
};
use serde::{Deserialize, Serialize};

use crate::intermediate_repr::ParsedNodeKind;
pub use crate::spawner::TscnToBevy;

/// Configure how the scene is converted from godot to bevy.
pub struct Config {
    /// We assert each asset path starts with this prefix.
    /// Then we strip it.
    pub asset_path_prefix: &'static str,
}

/// Parses Godot's .tscn file with very strict requirements on the content.
/// We only support nodes and parameters that are relevant to our game.
/// We panic on unsupported content aggressively.
///
/// See also [`TscnTree::spawn_into`].
pub fn parse(tscn: &str, config: Config) -> TscnTree {
    from_state_to_tscn(token::parse(tscn), config)
}

/// A godot scene is a tree of nodes.
/// This representation is used to populate bevy world.
/// We are very selective about what we support.
/// We panic on unsupported content aggressively.
///
/// See [`parse`] and [`TscnTree::spawn_into`].
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TscnTree {
    /// Root node name must always equal to the scene name.
    /// Other nodes refer to it as `"."`.
    pub root: Node,
}

/// Node's name is stored in the parent node's children map.
///
/// The convention is that a 2D node is an entity while a plain node is a
/// component.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Positional data is relevant for
    /// - `Node2D`
    /// - `Sprite2D`
    /// - `AnimatedSprite2D`
    ///
    /// and irrelevant for
    /// - `Node`
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub in_2d: Option<In2D>,
    /// Any node can have some metadata.
    /// These are relevant when spawning the node into bevy world.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// These nodes will be spawned as children if they have 2D positional
    /// data. Otherwise, they are treated as components and not entities.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub children: HashMap<NodeName, Node>,
}

/// The name of a node is unique within its parent.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct NodeName(pub String);

/// Either a `Node2D`, `Sprite2D`, or `AnimatedSprite2D` node.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct In2D {
    /// in 2D
    pub position: Vec2,
    /// Or calculated from position if missing.
    /// If a 2D node has a 2D node child called "YSort", then the position
    /// fed to the [`TscnToBevy::ysort`] function is the global position of
    /// that "YSort", i.e. the position of the 2D node plus the position of
    /// the "YSort".
    pub z_index: Option<f32>,
    /// for images and animations
    pub texture: Option<SpriteTexture>,
}

/// For images and animations.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteTexture {
    /// The path to the asset stripped of typically the `res://assets/` prefix.
    /// E.g. `apartment/cupboard.png`.
    /// The prefix is set in the [`Config`].
    pub path: String,
    /// We only support sprite frames that are part of an atlas (single file
    /// texture.)
    pub animation: Option<SpriteFrames>,
}

/// Atlas animation.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteFrames {
    /// If set to true, once the animation starts playing it will be repeated.
    pub should_endless_loop: bool,
    /// How many frames per second the animation should play at.
    pub fps: f32,
    /// If set to true, the animation starts playing as soon as it is loaded.
    pub should_autoload: bool,
    /// Set as the initial index to play the animation from.
    /// Must be index of a frame in the `frames` array.
    pub first_index: usize,
    /// Note that we use [`bevy::prelude::Rect`], hence the Y coordinate
    /// has been translated from godot to bevy coordinates.
    pub frames: Vec<Rect>,
    /// The min size of the texture that fits all the frames.
    pub size: Vec2,
}

impl<'a> From<NodeName> for Cow<'a, str> {
    fn from(NodeName(name): NodeName) -> Self {
        Cow::Owned(name)
    }
}

fn from_state_to_tscn(
    mut state: intermediate_repr::State,
    conf: Config,
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

    let mut root = Node {
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
        let mut metadata = HashMap::new();

        let mut z_index = None;
        let mut position = Vec2::ZERO;
        let mut path = None;
        let mut animation = None;

        for section_key in parsed_node.section_keys {
            apply_section_key(
                &conf,
                &state,
                section_key,
                &mut z_index,
                &mut position,
                &mut metadata,
                &mut path,
                &mut animation,
            );
        }

        let in_2d = match parsed_node.kind {
            ParsedNodeKind::AnimatedSprite2D => Some(In2D {
                position,
                z_index,
                texture: Some(SpriteTexture {
                    path: path.expect("AnimatedSprite2D should have a texture"),
                    animation: {
                        assert!(animation.is_some());
                        animation
                    },
                }),
            }),
            ParsedNodeKind::Sprite2D => Some(In2D {
                position,
                z_index,
                texture: Some(SpriteTexture {
                    path: path.expect("Sprite2D should have a texture"),
                    animation: {
                        assert!(animation.is_none());
                        None
                    },
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

        let node = Node {
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

    TscnTree { root }
}

#[allow(clippy::too_many_arguments)]
fn apply_section_key(
    conf: &Config,
    state: &intermediate_repr::State,
    section_key: intermediate_repr::SectionKey,
    z_index: &mut Option<f32>,
    position: &mut Vec2,
    metadata: &mut HashMap<String, String>,
    path: &mut Option<String>,
    animation: &mut Option<SpriteFrames>,
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

        SectionKey::ZIndex(Number(z)) => {
            assert!(
                z_index.replace(z).is_none(),
                "Node should not have more than one z_index"
            );
        }
        SectionKey::Position(X(x), y) => {
            *position = Vec2::new(x, y.into_bevy_coords());
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
        SectionKey::SpriteFramesSubResource(id) => {
            let res = state
                .sub_resources
                .iter()
                .find(|res| res.id == id)
                .expect("sub resource should exist");
            assert_eq!(1, res.section_keys.len());

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
                            let SectionKey::RegionRect2(X(x1), y1, X(x2), y2) =
                                section_key
                            else {
                                return None;
                            };

                            Some(Rect {
                                min: Vec2::new(*x1, y1.into_bevy_coords()),
                                max: Vec2::new(*x2, y2.into_bevy_coords()),
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
        assert!(godot_path.starts_with(self.asset_path_prefix));
        godot_path[self.asset_path_prefix.len()..].to_owned()
    }
}
