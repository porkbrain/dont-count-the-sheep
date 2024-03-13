//! Spawns the scene into a bevy world.

use std::str::FromStr;

use bevy::{
    asset::Handle,
    core::Name,
    ecs::{entity::Entity, event::Event, system::Commands},
    hierarchy::BuildChildren,
    math::Vec2,
    prelude::SpatialBundle,
    render::texture::Image,
    sprite::{Sprite, TextureAtlas, TextureAtlasLayout},
    transform::components::Transform,
    utils::default,
};
use common_top_down::InspectLabelCategory;

use crate::{In2D, Node, NodeName, SpriteTexture, TscnTree};

/// Guides the spawning process of a scene.
/// Use the [`TscnTree::spawn_into`] method to spawn the scene into a world.
/// The implementation has some knowledge of bevy and top down scenes to provide
/// default implementations for things like [`common_top_down::InspectLabel`]
/// and Y sorting.
///
/// For scene dependent behavior, the implementation defer to the user by
/// providing hooks like [`TscnToBevy::handle_plain_node`].
///
/// The implementation aggressively panics on invalid `.tscn` tree.
/// We recommend to do the same in the hooks.
pub trait TscnToBevy {
    /// The kind of action that can be emitted by an `InspectLabel`.
    type LocalActionKind: FromStr + Event + Clone;

    /// The kind of zone that can be entered by the player.
    type LocalZoneKind: FromStr;

    /// Gives access to the commands to spawn entities.
    fn cmd(&mut self) -> &mut Commands;

    /// Given a position in 2D, add a z index to it.
    /// This function is used for those nodes that don't have a z index set.
    /// If a 2D node has a 2D node child called "YSort", then the position fed
    /// to this function is the global position of that "YSort" node.
    fn ysort(&mut self, position: Vec2) -> f32;

    /// Any plan node (no 2D info) that is not handled by the default
    /// implementation will be passed to this function.
    ///
    /// The default implementation handles:
    /// - `InspectLabel`
    /// - `YSort`
    fn handle_plain_node(
        &mut self,
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

    /// When a player enters a zone, the entity with can be interacted with.
    /// See also [`common_top_down::inspect_and_interact::ZoneToInspectLabelEntity`].
    fn map_zone_to_inspect_label_entity(
        &mut self,
        _zone: Self::LocalZoneKind,
        _entity: Entity,
    ) {
        unimplemented!("Scene does not support mapping zones to entities")
    }
}

impl TscnTree {
    /// Spawns the tree of nodes into the world guided by the scene
    /// implementation.
    pub fn spawn_into(self, scene: &mut impl TscnToBevy) {
        let root = scene.cmd().spawn(Name::new("root")).id();
        node_to_entity(scene, root, self.root);
    }
}

fn node_to_entity<S: TscnToBevy>(scene: &mut S, entity: Entity, node: Node) {
    let In2D {
        position,
        z_index,
        texture,
    } = node.in_2d.expect("only 2D nodes represent entities");
    // might get populated by a YSort node, or if still None is calculated from
    // the position based on scene ysort impl
    let mut virtual_z_index = z_index;

    for (NodeName(child_name), mut child_node) in node.children {
        match (child_name.as_str(), child_node.in_2d.as_ref()) {
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
                    "YSort must child of a node with no zindex"
                );
                virtual_z_index = Some(scene.ysort(position + *child_position));
            }
            ("YSort", None) => panic!("YSort must be a Node2D with no zindex"),

            (_, Some(_)) => {
                // recursively spawn children
                let child_id = scene.cmd().spawn(Name::new(child_name)).id();
                scene.cmd().entity(entity).add_child(child_id);
                node_to_entity(scene, child_id, child_node);
            }

            ("InspectLabel", None) => {
                let with_label = child_node.metadata.remove("label").expect(
                    "Label metadata must be present on InspectLabelCategory",
                );

                let mut label = child_node
                    .metadata
                    .remove("category")
                    .map(|cat| {
                        InspectLabelCategory::from_str(&cat).expect(
                            "category must be a valid InspectLabelCategory",
                        )
                    })
                    .unwrap_or_default()
                    .into_label(with_label);

                if let Some(action) = child_node.metadata.remove("action") {
                    label.set_emit_event_on_interacted(
                        S::LocalActionKind::from_str(&action).unwrap_or_else(
                            |_| panic!("InspectLabel action not valid"),
                        ),
                    );
                }

                scene.cmd().entity(entity).insert(label);

                if let Some(zone) = child_node.metadata.remove("zone") {
                    scene.map_zone_to_inspect_label_entity(
                        S::LocalZoneKind::from_str(&zone)
                            .unwrap_or_else(|_| panic!("zone not valid")),
                        entity,
                    );
                }

                assert!(
                    child_node.metadata.is_empty(),
                    "InspectLabel node can only have \
                    label, category, action and zone metadata"
                );
            }
            (_, None) => {
                scene.handle_plain_node(entity, child_name, child_node)
            }
        }
    }

    let transform = Transform::from_translation(
        position
            .extend(virtual_z_index.unwrap_or_else(|| scene.ysort(position))),
    );
    scene.cmd().entity(entity).insert(SpatialBundle {
        transform,
        ..default()
    });

    if let Some(SpriteTexture { path, animation }) = texture {
        let texture = scene.load_texture(&path);
        scene
            .cmd()
            .entity(entity)
            .insert(texture)
            .insert(Sprite::default());

        if let Some(animation) = animation {
            let mut layout = TextureAtlasLayout::new_empty(animation.size);
            for frame in animation.frames {
                layout.add_texture(frame);
            }

            let layout = scene.add_texture_atlas(layout);
            scene.cmd().entity(entity).insert(TextureAtlas {
                index: animation.first_index,
                layout,
            });
        }
    }
}
