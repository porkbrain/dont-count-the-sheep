//! Spawns the scene into a bevy world.

use std::time::Duration;

use bevy::{
    asset::Handle,
    color::Color,
    core::Name,
    ecs::{entity::Entity, system::Commands},
    hierarchy::BuildChildren,
    math::Vec3,
    prelude::SpatialBundle,
    render::{texture::Image, view::Visibility},
    sprite::{Sprite, TextureAtlas, TextureAtlasLayout},
    time::TimerMode,
    transform::components::Transform,
    utils::{default, HashMap},
};
use common_visuals::{AtlasAnimation, AtlasAnimationEnd, AtlasAnimationTimer};

use crate::{
    rscn::{In2D, Node, NodeName, Point, SpriteTexture, TscnTree},
    vec2_ext::Vec2Ext,
};

pub struct TscnSpawnerNew {
    handle_2d_nodes: HashMap<String, Box<dyn Handle2DNode>>,
    default_2d_node_handler: Option<Box<dyn Handle2DNode>>,
    handle_plain_nodes: HashMap<String, Box<dyn HandlePlainNode>>,
    default_plain_node_handler: Option<Box<dyn HandlePlainNode>>,
}

/// Hook for plain nodes (no 2D info).
pub trait HandlePlainNode {
    fn handle(
        &mut self,
        cmd: &mut Commands,
        parent: (Entity, &Node),
        name: &str,
        node: &Node,
    );
}

/// Hook for 2D nodes.
pub trait Handle2DNode {
    fn handle(
        &mut self,
        cmd: &mut Commands,
        parent: (Entity, &Node),
        name: &str,
        node: &Node,
    );
}

impl TscnSpawnerNew {
    pub fn new() -> Self {
        Self {
            handle_2d_nodes: HashMap::default(),
            default_2d_node_handler: None,
            handle_plain_nodes: HashMap::default(),
            default_plain_node_handler: None,
        }
    }

    /// Given 2D node name, tell us what should the spawner do with it.
    /// If the node is not recognized, the default handler will be called.
    pub fn add_2d_node_handler(
        &mut self,
        name: impl Into<String>,
        handler: impl Handle2DNode + 'static,
    ) {
        let name = name.into();
        let already_inserted = self
            .handle_2d_nodes
            .insert(name.clone(), Box::new(handler))
            .is_some();

        assert!(!already_inserted, "2D node {name} already has a handler");
    }

    /// If provided then will be called for any 2D node that doesn't have a
    /// handler yet.
    pub fn set_default_2d_node_handler(
        &mut self,
        handler: impl Handle2DNode + 'static,
    ) {
        self.default_2d_node_handler = Some(Box::new(handler));
    }

    /// Given plain node name (ie. not a 2D node), tell us what should the
    /// spawner do with it.
    pub fn add_plain_node_handler(
        &mut self,
        name: impl Into<String>,
        handler: impl HandlePlainNode + 'static,
    ) {
        let name = name.into();
        let already_inserted = self
            .handle_plain_nodes
            .insert(name.clone(), Box::new(handler))
            .is_some();

        assert!(!already_inserted, "Plain node {name} already has a handler");
    }

    /// If provided then will be called for any plain node that doesn't have a
    /// handler yet.
    pub fn set_default_plain_node_handler(
        &mut self,
        handler: impl HandlePlainNode + 'static,
    ) {
        self.default_plain_node_handler = Some(Box::new(handler));
    }
}

struct PointNodeHandler;
impl HandlePlainNode for PointNodeHandler {
    fn handle(
        &mut self,
        cmd: &mut Commands,
        (parent_entity, parent_node): (Entity, &Node),
        name: &str,
        node: &Node,
    ) {
        cmd.entity(parent_entity).insert(Point(
            parent_node
                .in_2d
                .as_ref()
                .expect("Point must have a 2D node parent")
                .position,
        ));
    }
}

/// Guides the spawning process of a scene.
///
/// Use the [`TscnTree::spawn_into`] method to spawn the scene into a world.
/// The implementation has some knowledge of bevy and top down scenes to provide
/// default implementations for things like [`crate::top_down::InspectLabel`]
/// and Y sorting.
///
/// For scene dependent behavior, the implementation defer to the user by
/// providing hooks like [`TscnSpawner::handle_plain_node`].
///
/// The implementation aggressively panics on invalid `.tscn` tree.
/// We recommend to do the same in the hooks.
pub trait TscnSpawner {
    /// Entity that has been spawned.
    /// Runs after all [`TscnSpawner::handle_plain_node`].
    /// The entity already has [`Name`], [`SpatialBundle`] and possibly
    /// [`Sprite`] and [`TextureAtlas`] components.
    fn on_spawned(
        &mut self,
        cmd: &mut Commands,
        who: Entity,
        name: NodeName,
        translation: Vec3,
    );

    /// Any plain node (no 2D info) that is not handled by the default
    /// implementation will be passed to this function.
    /// Runs before [`TscnSpawner::on_spawned`] of the parent.
    /// The parent is already scheduled to spawn and has some components
    /// like [`Name`], [`Sprite`] and [`Handle<Image>`] if applicable.
    /// It does not have a [`SpatialBundle`] yet.
    fn handle_plain_node(
        &mut self,
        _cmd: &mut Commands,
        _parent: Entity,
        _name: String,
        _node: Node,
    ) {
        unimplemented!("Scene does not support plain nodes")
    }

    /// Load a texture from a path.
    /// Pretty much an access to the asset server.
    fn load_texture(&mut self, _path: &str) -> Handle<Image> {
        unimplemented!("Scene does not support loading textures")
    }

    /// Add a texture atlas to the assets resource.
    fn add_texture_atlas(
        &mut self,
        _layout: TextureAtlasLayout,
    ) -> Handle<TextureAtlasLayout> {
        unimplemented!("Scene does not support texture atlases")
    }

    /// Some scenes that create many spawners might want to have their own
    /// spawner trait that extends this one.
    /// Those scenes can define some nodes that won't go into
    /// If a node name is recognized by this function, it will be passed to
    /// [Self::handle_extension_node] instead of [Self::handle_plain_node].
    fn is_extension_node(&self, _name: &str) -> bool {
        false
    }

    /// If [Self::is_extension_node] returns true for a node, this function
    /// will be called to handle it.
    fn handle_extension_node(
        &mut self,
        _cmd: &mut Commands,
        _parent: Entity,
        _name: String,
        _node: Node,
    ) {
        unimplemented!("Not a TscnSpawner extension")
    }
}

impl TscnTree {
    /// Spawns the tree of nodes into the world guided by the scene
    /// implementation.
    pub fn spawn_into_world(
        self,
        with_spawner: &mut impl TscnSpawner,
        cmd: &mut Commands,
    ) {
        let root = cmd.spawn(Name::new(self.root_node_name.clone())).id();
        node_to_entity(with_spawner, cmd, root, self.root_node_name, self.root);
    }
}

fn node_to_entity(
    spawner: &mut impl TscnSpawner,
    cmd: &mut Commands,
    entity: Entity,
    name: NodeName,
    node: Node,
) {
    let In2D {
        position,
        z_index,
        texture,
    } = node.in_2d.expect("only 2D nodes represent entities");

    let mut visibility = Visibility::default();
    if let Some(SpriteTexture {
        path,
        animation,
        visible,
        color,
        flip_horizontally,
        flip_vertically,
    }) = texture
    {
        let texture = spawner.load_texture(&path);
        cmd.entity(entity).insert(texture).insert(Sprite {
            color: color.unwrap_or(Color::WHITE),
            flip_x: flip_horizontally,
            flip_y: flip_vertically,
            ..default()
        });

        if !visible {
            visibility = Visibility::Hidden;
        }

        if let Some(animation) = animation {
            let mut layout =
                TextureAtlasLayout::new_empty(animation.size.as_uvec2());
            let frames_count = animation.frames.len();
            assert_ne!(0, frames_count);
            for frame in animation.frames {
                layout.add_texture(frame.as_urect());
            }

            let layout = spawner.add_texture_atlas(layout);
            cmd.entity(entity)
                .insert(TextureAtlas {
                    index: animation.first_index,
                    layout,
                })
                .insert(AtlasAnimation {
                    on_last_frame: if animation.should_endless_loop {
                        AtlasAnimationEnd::LoopIndefinitely
                    } else {
                        AtlasAnimationEnd::RemoveTimer
                    },
                    // This asks: "what's the first frame" when animation
                    // resets. Even though the first frame
                    // that's shown is the first_index.
                    first: 0,
                    last: frames_count - 1,
                    ..default()
                });

            if animation.should_autoload {
                cmd.entity(entity).insert(AtlasAnimationTimer::new(
                    Duration::from_secs_f32(1.0 / animation.fps),
                    TimerMode::Repeating,
                ));
            }
        }
    }

    // might get populated by a YSort node, or if still None is calculated from
    // the position based on scene ysort impl
    let mut virtual_z_index = z_index;

    for (NodeName(child_name), child_node) in node.children {
        match (child_name.as_str(), child_node.in_2d.as_ref()) {
            // Given a position in 2D, add a z index to it.
            // This function is used for those nodes that don't have a z index
            // set. If a 2D node has a 2D node child called "YSort",
            // then the position fed to this function is the global
            // position of that "YSort" node.
            (
                "YSort",
                Some(In2D {
                    position: child_position,
                    z_index: None,
                    texture: None,
                }),
            ) => {
                assert!(
                    virtual_z_index.is_none(),
                    "Node {name:?} has YSort child node and zindex at the same time"
                );
                virtual_z_index = Some((position + *child_position).ysort());
            }
            ("YSort", None) => panic!("YSort must be a Node2D with no zindex"),

            // ("Point", None) => {
            //     cmd.entity(entity).insert(Point(position));
            // }
            // ("Point", _) => panic!("Point must be a plain node"),
            (_, Some(_)) => {
                // recursively spawn children
                let child_id = cmd.spawn(Name::new(child_name.clone())).id();
                cmd.entity(entity).add_child(child_id);
                node_to_entity(
                    spawner,
                    cmd,
                    child_id,
                    NodeName(child_name),
                    child_node,
                );
            }

            (s, None) if spawner.is_extension_node(s) => spawner
                .handle_extension_node(cmd, entity, child_name, child_node),
            (_, None) => {
                spawner.handle_plain_node(cmd, entity, child_name, child_node)
            }
        }
    }

    // default zindex is 0 as per Godot, but we use f32::EPSILON to avoid z
    // fighting between nested nodes (parent vs child)
    let translation = position.extend(virtual_z_index.unwrap_or(f32::EPSILON));
    let transform = Transform::from_translation(translation);
    cmd.entity(entity).insert(SpatialBundle {
        transform,
        visibility,
        ..default()
    });

    bevy::log::trace!("Spawning {entity:?} {name:?} from scene file");
    // TODO: handle 2d node
    spawner.on_spawned(cmd, entity, name, translation);
}
