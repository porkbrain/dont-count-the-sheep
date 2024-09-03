//! Spawns a parsed tscn file into a bevy world.
//!
//! The restrictions:
//! - a non 2D node can only be a leaf node with no children
//! - the root node must be 2D node

use std::time::Duration;

use bevy::utils::EntityHashMap;
use common_visuals::{AtlasAnimation, AtlasAnimationEnd, AtlasAnimationTimer};

use crate::{
    prelude::*,
    rscn::{In2D, NodeName, RscnNode, SpriteTexture, TscnTree},
};

/// Maps entity to its component description.
pub type EntityDescriptionMap = EntityHashMap<Entity, EntityDescription>;

/// All components that are managed by the scene spawner implementation.
#[derive(Default)]
#[allow(missing_docs)]
pub struct EntityDescription {
    pub visibility: Visibility,
    pub translation: Vec2,
    pub z_index: Option<f32>,
    pub texture: Option<Handle<Image>>,
    pub sprite: Option<Sprite>,
    pub texture_atlas: Option<Handle<TextureAtlasLayout>>,
    pub atlas_animation: Option<AtlasAnimation>,
    pub atlas_animation_timer: Option<AtlasAnimationTimer>,
}

/// Guides the spawning process of a scene.
///
/// Use the [`TscnTree::spawn_into`] method to spawn the scene into a world.
///
/// For scene dependent behavior, the implementation defer to the user by
/// providing hooks [`TscnSpawner::handle_plain_node`] and
/// [`TscnSpawner::handle_2d_node`].
///
/// The implementation aggressively panics on invalid `.tscn` tree.
/// We recommend to do the same in the hooks.
pub trait TscnSpawnHooks {
    /// Called just before all components from the entity description are
    /// inserted into the entity.
    ///
    /// If you remove any entity from the descriptions map, it will not be
    /// spawned and neither will its children.
    /// You may not insert any new descriptions into the map, only remove them
    /// or modify them.
    fn handle_2d_node(
        &mut self,
        cmd: &mut Commands,
        descriptions: &mut EntityDescriptionMap,
        parent: Option<(Entity, NodeName)>,
        this: (Entity, NodeName),
    );

    /// Called when a node is not a 2D node.
    /// Plain nodes are leaf nodes, we don't walk their children.
    ///
    /// If you remove any entity from the descriptions map, it will not be
    /// spawned and neither will its children.
    /// You may not insert any new descriptions into the map, only remove them
    /// or modify them.
    fn handle_plain_node(
        &mut self,
        _cmd: &mut Commands,
        _descriptions: &mut EntityDescriptionMap,
        _parent: (Entity, NodeName),
        this: (NodeName, RscnNode),
    ) {
        unimplemented!("Scene does not support plain nodes, found {this:?}");
    }
}

impl TscnTree {
    /// Spawns the tree of nodes into the world guided by the scene
    /// implementation.
    pub fn spawn_into(
        self,
        cmd: &mut Commands,
        atlases: &mut Assets<TextureAtlasLayout>,
        asset_server: &AssetServer,
        hooks: &mut impl TscnSpawnHooks,
    ) {
        let mut ctx = Context {
            atlases,
            asset_server,
            entity_descriptions: Default::default(),
        };
        let root = cmd.spawn(Name::new(self.root_node_name.clone())).id();
        node_to_entity(
            &mut ctx,
            hooks,
            cmd,
            None, // no parent
            (root, self.root_node_name),
            self.root,
        );

        if !ctx.entity_descriptions.is_empty() {
            error!(
                "There are {} improperly spawned entities",
                ctx.entity_descriptions.len()
            );
        }
    }
}

/// Context data to the tree walk in [node_to_entity].
struct Context<'a> {
    atlases: &'a mut Assets<TextureAtlasLayout>,
    asset_server: &'a AssetServer,
    entity_descriptions: EntityDescriptionMap,
}

fn node_to_entity(
    ctx: &mut Context<'_>,
    hooks: &mut impl TscnSpawnHooks,
    cmd: &mut Commands,
    parent: Option<(Entity, NodeName)>,
    (entity, name): (Entity, NodeName),
    node: RscnNode,
) {
    let In2D {
        position,
        z_index,
        texture,
    } = node.in_2d.expect("only 2D nodes represent entities");

    let mut description = EntityDescription {
        translation: position,
        z_index,
        ..Default::default()
    };

    if let Some(SpriteTexture {
        path,
        animation,
        visible,
        color,
        flip_horizontally,
        flip_vertically,
    }) = texture
    {
        let texture = ctx.asset_server.load(&path);
        description.texture = Some(texture);
        description.sprite = Some(Sprite {
            color: color.unwrap_or(Color::WHITE),
            flip_x: flip_horizontally,
            flip_y: flip_vertically,
            ..default()
        });

        if !visible {
            description.visibility = Visibility::Hidden;
        }

        if let Some(animation) = animation {
            let mut layout =
                TextureAtlasLayout::new_empty(animation.size.as_uvec2());
            let frames_count = animation.frames.len();
            assert_ne!(0, frames_count);
            for frame in animation.frames {
                layout.add_texture(frame.as_urect());
            }

            let layout = ctx.atlases.add(layout);
            description.texture_atlas = Some(layout);
            description.atlas_animation = Some(AtlasAnimation {
                on_last_frame: if animation.should_endless_loop {
                    AtlasAnimationEnd::LoopIndefinitely
                } else {
                    AtlasAnimationEnd::RemoveTimer
                },
                // When animation resets this answers: "What's the first frame?"
                // Even though the first frame that's shown is the first_index.
                first: 0,
                last: frames_count - 1,
                ..default()
            });

            if animation.should_autoload {
                description.atlas_animation_timer =
                    Some(AtlasAnimationTimer::new(
                        Duration::from_secs_f32(1.0 / animation.fps),
                        TimerMode::Repeating,
                    ));
            }
        }
    }

    ctx.entity_descriptions.insert(entity, description);

    for (child_name, child_node) in node.children {
        if child_node.in_2d.is_some() {
            // recursively spawn 2D children

            let child_id = cmd.spawn(Name::new(child_name.clone())).id();
            cmd.entity(entity).add_child(child_id);
            node_to_entity(
                ctx,
                hooks,
                cmd,
                Some((entity, name.clone())),
                (child_id, child_name),
                child_node,
            );
        } else {
            hooks.handle_plain_node(
                cmd,
                &mut ctx.entity_descriptions,
                (entity, name.clone()),
                (child_name, child_node),
            );
        }
    }

    trace!("Handling 2D entity {name:?}");
    hooks.handle_2d_node(
        cmd,
        &mut ctx.entity_descriptions,
        parent,
        (entity, name),
    );

    let mut entity_cmd = cmd.entity(entity);

    let Some(EntityDescription {
        visibility,
        translation,
        z_index,
        texture,
        sprite,
        texture_atlas,
        atlas_animation,
        atlas_animation_timer,
    }) = ctx.entity_descriptions.remove(&entity)
    else {
        entity_cmd.despawn_recursive();
        return;
    };

    entity_cmd.insert(SpatialBundle {
        // default zindex is 0 as per Godot, but we use f32::EPSILON to avoid z
        // fighting between nested nodes (parent vs child)
        transform: Transform::from_translation(
            translation.extend(z_index.unwrap_or(f32::EPSILON)),
        ),
        visibility,
        ..default()
    });

    if let Some(texture) = texture {
        entity_cmd.insert(texture);
    }
    if let Some(sprite) = sprite {
        entity_cmd.insert(sprite);
    }
    if let Some(texture_atlas) = texture_atlas {
        entity_cmd.insert(texture_atlas);
    }
    if let Some(atlas_animation) = atlas_animation {
        entity_cmd.insert(atlas_animation);
    }
    if let Some(atlas_animation_timer) = atlas_animation_timer {
        entity_cmd.insert(atlas_animation_timer);
    }
}
