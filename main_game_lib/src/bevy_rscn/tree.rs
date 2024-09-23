use bevy::{
    color::Color,
    log::warn,
    math::{Rect, Vec2},
    utils::{default, HashMap},
};
use rscn::{
    self,
    godot::{self, ExtResource, SubResourceId, SubResourceSectionKey},
};

use crate::bevy_rscn::{
    Config, In2D, NodeName, RscnNode, SpriteFrames, SpriteTexture, TscnTree,
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

pub(crate) fn from_scene(
    mut scene: godot::Scene,
    conf: &Config,
) -> miette::Result<TscnTree> {
    let root_node_index = scene
        .nodes
        .iter()
        .position(|node| node.parent.is_none())
        .expect("there should be a node with no parent");
    let parsed_root_node = scene.nodes.remove(root_node_index);
    assert!(
        parsed_root_node.section.is_empty(),
        "Root node must have no extra data"
    );
    assert_eq!(
        godot::NodeKind::Node2D,
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
    scene.nodes.sort_by_key(|node| {
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
    std::mem::swap(&mut nodes, &mut scene.nodes); // to avoid borrow checker
    for parsed_node in nodes {
        let mut properties = default();

        for (section_key, section_value) in parsed_node.section {
            apply_section(
                conf,
                &scene,
                &mut properties,
                section_key,
                section_value,
            )?;
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
            godot::NodeKind::AnimatedSprite2D => Some(In2D {
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
            godot::NodeKind::Sprite2D => Some(In2D {
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
            godot::NodeKind::Node2D => Some(In2D {
                position,
                z_index,
                texture: {
                    assert!(path.is_none());
                    assert!(animation.is_none());
                    None
                },
            }),
            godot::NodeKind::Node => {
                assert_eq!(Vec2::ZERO, position);
                assert!(z_index.is_none());
                assert!(path.is_none());
                assert!(animation.is_none());
                None
            }
            godot::NodeKind::Other(kind) => {
                panic!("Node kind '{kind}' is not supported")
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

    Ok(TscnTree {
        root,
        root_node_name: NodeName(root_node_name),
    })
}

fn apply_section(
    conf: &Config,
    scene: &godot::Scene,
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
    section_key: rscn::godot::NodeSectionKey,
    section_value: rscn::value::SpannedValue,
) -> miette::Result<()> {
    use rscn::godot::NodeSectionKey;

    match section_key {
        NodeSectionKey::SelfModulate => {
            let (r, g, b, a) = section_value.into_self_modulate_color_rgba()?;
            assert!(
                color
                    .replace(Color::srgba(r as _, g as _, b as _, a as _))
                    .is_none(),
                "Node should not have more than one color"
            );
        }
        NodeSectionKey::FlipHorizontally => {
            *flip_horizontally =
                section_value.try_into_bool().map(|(_, b)| b)?;
        }
        NodeSectionKey::FlipVertically => {
            *flip_vertically = section_value.try_into_bool().map(|(_, b)| b)?;
        }
        NodeSectionKey::Visible => {
            *visible = section_value.try_into_bool().map(|(_, b)| b)?;
        }
        NodeSectionKey::ZIndex => {
            let (_, z) = section_value.try_into_number()?;
            assert!(
                z_index.replace(z as _).is_none(),
                "Node should not have more than one z_index"
            );
        }
        NodeSectionKey::Position => {
            let (x, godot_y) = section_value.into_vector2()?;
            // This is the conversion from godot to bevy coordinates.
            // Note that not all Y coords should be converted.
            // For example sprite atlas positions into textures in bevy follow
            // the image processing convention where the origin is
            // at the top left.
            let bevy_y = -godot_y;
            *position = Vec2::new(x as _, bevy_y as _);
        }
        NodeSectionKey::StringMetadata => {
            let (_, map) = section_value.try_into_object()?;
            for (key, value) in map {
                let (_, value) = value.try_into_string()?;
                assert!(
                    metadata.insert(key, value).is_none(),
                    "Node should not have more than \
                    one metadata with the same key"
                );
            }
        }
        NodeSectionKey::TextureExtResource => {
            let id = section_value.try_into_ext_resource()?;
            let prefixless_path = scene
                .ext_resources
                .iter()
                .find(|res| res.id() == &id)
                .map(|res| {
                    if let ExtResource::Texture2D {
                        path: texture_path, ..
                    } = res
                    {
                        conf.to_prefixless_path(&texture_path)
                    } else {
                        panic!("ext resource should be a texture: {res:?}")
                    }
                })
                .unwrap_or_else(|| {
                    panic!(
                        "ext resource {id:?} should exist in {:?}",
                        scene.ext_resources
                    )
                });
            assert!(
                path.replace(prefixless_path).is_none(),
                "Node should not have more than one texture"
            );
        }
        NodeSectionKey::FrameIndex => {
            let (_, index) = section_value.try_into_number()?;
            animation
                .as_mut()
                .expect("Frame index always comes after sprite_frames")
                .first_index = index as _;
        }
        NodeSectionKey::Autoplay => {
            let (_, autoplay_anim_name) = section_value.try_into_string()?;
            assert!(
                autoplay_anim_name == "default",
                "For now we only support autoplaying the default animation"
            );
            animation
                .as_mut()
                .expect("Autoplay always comes after sprite_frames")
                .should_autoload = true;
        }
        NodeSectionKey::FrameProgress => {
            warn!("Godot's FrameProgress is not supported yet");
        }
        NodeSectionKey::SpriteFrames => {
            let id = section_value.try_into_sub_resource()?;

            let res = scene
                .sub_resources
                .iter()
                .find(|res| res.id == id)
                .expect("sub resource should exist");
            assert_eq!(
                1,
                res.section.len(),
                "SpriteFrames should have exactly one animation"
            );

            let Some(section_value) =
                &res.section.get(&SubResourceSectionKey::Animations)
            else {
                panic!(
                    "SpriteFrames should have exactly one SingleAnim section key"
                )
            };

            let anim = (*section_value)
                .clone()
                .try_into_sprite_frames_animations()?;

            assert!(
                anim.len() == 1,
                "We currently support only a single animation"
            );
            let anim = anim.into_iter().next().unwrap();

            let mut max_y = 0.0f32;
            let mut max_x = 0.0f32;

            let frames = anim
                .frames
                .into_iter()
                .map(|(texture_id, duration)| {
                    assert!(
                        duration == 1.0,
                        "We don't support frame durations"
                    );

                    map_texture_to_atlas_rect(
                        conf, scene, path, &mut max_x, &mut max_y, texture_id,
                    )
                })
                .collect::<miette::Result<Vec<_>>>()?;

            assert!(animation
                .replace(SpriteFrames {
                    should_endless_loop: anim.loop_,
                    fps: anim.speed.into(),
                    frames,
                    size: Vec2::new(max_x, max_y),
                    // Can be set by [NodeSectionKey::Autoplay]
                    should_autoload: false,
                    // Can be set by [NodeSectionKey::FrameIndex]
                    first_index: 0,
                })
                .is_none());
        }
        NodeSectionKey::Other(key) => {
            warn!("Node section key '{key}' is not supported");
        }
    };

    Ok(())
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

/// Also updates the texture path, max_x, and max_y.
fn map_texture_to_atlas_rect(
    conf: &Config,
    scene: &godot::Scene,
    path: &mut Option<String>,
    max_x: &mut f32,
    max_y: &mut f32,
    texture_id: SubResourceId,
) -> miette::Result<Rect> {
    let frame = scene
        .sub_resources
        .iter()
        .find(|res| res.id == texture_id)
        .expect("sub resource should exist");
    assert_eq!(2, frame.section.len());

    let prefixless_path = frame
        .section
        .iter()
        .find_map(|(section_key, section_value)| {
            let SubResourceSectionKey::AtlasExtResource = section_key else {
                return None;
            };

            let id = match section_value.clone().try_into_ext_resource() {
                Ok(id) => id,
                Err(err) => {
                    panic!("ext resource should be a texture: {err}")
                }
            };

            let path = scene
                .ext_resources
                .iter()
                .find(|res| res.id() == &id)
                .map(|res| {
                    if let ExtResource::Texture2D {
                        path: texture_path, ..
                    } = res
                    {
                        conf.to_prefixless_path(&texture_path)
                    } else {
                        panic!("ext resource should be a texture")
                    }
                })
                .expect("ext resource should exist");

            Some(path)
        })
        .expect("sub resource should have an atlas section key");

    if let Some(path) = path {
        assert_eq!(path, &prefixless_path);
    } else {
        *path = Some(prefixless_path);
    }

    let rect = frame
        .section
        .iter()
        .find_map(|(section_key, section_value)| {
            let SubResourceSectionKey::Region = section_key else {
                return None;
            };

            let res = section_value.clone().try_into_rect2().and_then(
                |(x1, y1, w, h)| {
                    // we don't convert into bevy coords here
                    // because
                    // bevy uses the top-left corner as the
                    // origin
                    // for textures
                    let (x1, y1, w, h) = (x1 as _, y1 as _, w as f32, h as f32);
                    Ok(Rect {
                        min: Vec2::new(x1, y1),
                        max: Vec2::new(x1 + w, y1 + h),
                    })
                },
            );

            Some(res)
        })
        .expect("sub resource should have a region section key")?;

    *max_x = max_x.max(rect.max.x);
    *max_y = max_y.max(rect.max.y);

    Ok(rect)
}
