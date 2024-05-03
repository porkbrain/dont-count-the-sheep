//! Spawns the scene into a bevy world.

use std::{str::FromStr, time::Duration};

use bevy::{
    asset::Handle,
    core::Name,
    ecs::{entity::Entity, event::Event, system::Commands},
    hierarchy::BuildChildren,
    math::Vec3,
    prelude::SpatialBundle,
    render::{color::Color, texture::Image, view::Visibility},
    sprite::{Sprite, TextureAtlas, TextureAtlasLayout},
    time::TimerMode,
    transform::components::Transform,
    utils::default,
};
use common_visuals::{AtlasAnimation, AtlasAnimationEnd, AtlasAnimationTimer};

use crate::{
    rscn::{In2D, Node, NodeName, Point, SpriteTexture, TscnTree},
    top_down::{layout::ysort, InspectLabelCategory},
};

/// Guides the spawning process of a scene.
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
    /// The kind of action that can be emitted by an `InspectLabel`.
    type LocalActionKind: FromStr + Event + Clone;

    /// The kind of zone that can be entered by the player.
    type LocalZoneKind: FromStr;

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

    /// Any plan node (no 2D info) that is not handled by the default
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

    /// When a player enters a zone, the entity with can be interacted with.
    /// See also [`crate::top_down::inspect_and_interact::ZoneToInspectLabelEntity`].
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
    pub fn spawn_into<T: TscnSpawner>(
        self,
        with_spawner: &mut T,
        cmd: &mut Commands,
    ) {
        let root = cmd.spawn(Name::new(self.root_node_name.clone())).id();
        node_to_entity(with_spawner, cmd, root, self.root_node_name, self.root);
    }
}

fn node_to_entity<T: TscnSpawner>(
    spawner: &mut T,
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
    }) = texture
    {
        let texture = spawner.load_texture(&path);
        cmd.entity(entity).insert(texture).insert(Sprite {
            color: color.unwrap_or(Color::WHITE),
            flip_x: flip_horizontally,
            ..default()
        });

        if !visible {
            visibility = Visibility::Hidden;
        }

        if let Some(animation) = animation {
            let mut layout = TextureAtlasLayout::new_empty(animation.size);
            let frames_count = animation.frames.len();
            assert_ne!(0, frames_count);
            for frame in animation.frames {
                layout.add_texture(frame);
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

    for (NodeName(child_name), mut child_node) in node.children {
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
                    "YSort must child of a node with no zindex"
                );
                virtual_z_index = Some(ysort(position + *child_position));
            }
            ("YSort", None) => panic!("YSort must be a Node2D with no zindex"),

            ("Point", None) => {
                cmd.entity(entity).insert(Point(position));
            }
            ("Point", _) => panic!("Point must be a plain node"),

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
                        T::LocalActionKind::from_str(&action).unwrap_or_else(
                            |_| panic!("InspectLabel action not valid"),
                        ),
                    );
                }

                cmd.entity(entity).insert(label);

                if let Some(zone) = child_node.metadata.remove("zone") {
                    spawner.map_zone_to_inspect_label_entity(
                        T::LocalZoneKind::from_str(&zone)
                            .unwrap_or_else(|_| panic!("Zone not valid")),
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

    bevy::log::trace!("Spawning {entity:?} {name:?} from scene file",);
    spawner.on_spawned(cmd, entity, name, translation);
}
