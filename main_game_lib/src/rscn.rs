//! As of now, bevy has no built-in editor.
//! Plugins are available with very simple implementations.
//! After experimenting with Godot, I ended up liking the editor a lot.
//!
//! The decision has been made to use Godot for the editor and bevy for the game
//! engine. This crate parses `.tscn` files and provides a way to load them into
//! bevy.
//!
//! Everything aggressively panics.
//! We support very limited subset of what Godot supports, only things that are
//! relevant to our use case.
//!
//! The tree structure is parsed and converted into bevy entities.
//! 2D nodes are entities (with relevant components) and child-parent
//! relationships are preserved. Plain nodes are typically components.
//! See the wiki for current status of what's supported and what custom nodes
//! are available.

mod intermediate_repr;
mod loader;
mod spawner;
mod token;
mod tree;

use std::borrow::Cow;

use bevy::{
    asset::{Asset, AssetServer, Assets, Handle},
    core::Name,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    hierarchy::DespawnRecursiveExt,
    math::{Rect, Vec2},
    reflect::TypePath,
    render::color::Color,
    utils::HashMap,
};
use common_ext::QueryExt;
pub use loader::{LoaderError, TscnLoader};
use serde::{Deserialize, Serialize};
pub use spawner::TscnSpawner;

use crate::top_down::TopDownScene;

/// A helper component that is always in an entity with
/// [`bevy::prelude::SpatialBundle`].
/// Translated a simple point from Godot.
/// To add this component, add a child Godot `Node` named `Point` to a parent
/// Godot `Node2D`.
#[derive(
    Component,
    Clone,
    Copy,
    Debug,
    Deserialize,
    PartialEq,
    bevy::reflect::Reflect,
    Serialize,
)]
pub struct Point(pub Vec2);

/// Configure how the scene is converted from godot to bevy.
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// We assert each asset path starts with this prefix.
    /// Then we strip it.
    pub asset_path_prefix: String,
}

/// A godot scene is a tree of nodes.
/// This representation is used to populate bevy world.
/// We are very selective about what we support.
/// We panic on unsupported content aggressively.
///
/// See [`parse`] and [`TscnTree::spawn_into`].
#[derive(Asset, TypePath, Debug, PartialEq, Serialize, Deserialize)]
pub struct TscnTree {
    /// The root node of the scene as defined in Godot.
    pub root_node_name: NodeName,
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
    /// fed to the [`crate::top_down::layout::ysort`] function is the global
    /// position of that "YSort", i.e. the position of the 2D node plus the
    /// position of the "YSort".
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
    /// Whether the sprite is visible or hidden.
    pub visible: bool,
    /// Changed by the Self Modulate property in Godot.
    pub color: Option<Color>,
    /// We only support sprite frames that are part of an atlas (single file
    /// texture.)
    pub animation: Option<SpriteFrames>,
    /// If the texture should be flipped horizontally.
    pub flip_horizontally: bool,
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

/// Marks scene as "can be loaded from .tscn".
///
/// Autoimplemented for [`TopDownScene`]s.
pub trait TscnInBevy: Send + Sync + 'static {
    /// Asset path of the `.tscn` file associated with this scene.
    fn tscn_asset_path() -> String;
}

/// Used for loading of [`TscnTree`] from a .tscn file.
#[derive(Component)]
pub struct TscnTreeHandle<T> {
    /// Will be used to despawn this handle on consumption.
    entity: Entity,
    /// The actual handle that can be used to load the file.
    /// Will be set to [`None`] once the scene is spawned.
    handle: Option<Handle<TscnTree>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: TopDownScene> TscnInBevy for T {
    fn tscn_asset_path() -> String {
        format!("scenes/{}.tscn", T::name())
    }
}

/// Parses Godot's .tscn file with very strict requirements on the content.
/// We only support nodes and parameters that are relevant to our game.
/// We panic on unsupported content aggressively.
///
/// See also [`TscnTree::spawn_into`].
pub fn parse(tscn: &str, config: &Config) -> TscnTree {
    tree::from_state(token::parse(tscn), config)
}

/// Run this system on enter to a scene to start loading the `.tscn` file.
/// Use then [`tscn_loaded_but_not_spawned`] condition to guard the
/// system that spawns the scene after loading is done.
pub fn start_loading_tscn<T: TscnInBevy>(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut e = cmd.spawn(Name::new(".tscn tree handle"));
    e.insert(TscnTreeHandle::<T> {
        entity: e.id(),
        handle: Some(asset_server.load(T::tscn_asset_path())),
        _phantom: Default::default(),
    });
}

/// Guard condition for when spawning of the scene hasn't started but can be.
pub fn tscn_loaded_but_not_spawned<T: TscnInBevy>(
) -> impl FnMut(Query<&TscnTreeHandle<T>>, Res<AssetServer>) -> bool {
    move |tscn: Query<&TscnTreeHandle<T>>, asset_server: Res<AssetServer>| {
        tscn.get_single_or_none()
            .and_then(|tscn| {
                Some(
                    asset_server
                        .is_loaded_with_dependencies(tscn.handle.as_ref()?),
                )
            })
            .unwrap_or(false)
    }
}

impl<T> TscnTreeHandle<T> {
    /// Consume the handle and return the loaded scene.
    /// After this, the handle is useless and the entity is despawned.
    /// Also, the scene is removed from the asset server.
    pub fn consume(
        &mut self,
        cmd: &mut Commands,
        assets: &mut Assets<TscnTree>,
    ) -> TscnTree {
        let handle = self.handle.take().expect("Handle already consumed");
        let tscn = assets.remove(handle).expect("Handle not loaded");
        cmd.entity(self.entity).despawn_recursive();
        tscn
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            asset_path_prefix: "res://assets/".to_string(),
        }
    }
}

impl<'a> From<NodeName> for Cow<'a, str> {
    fn from(NodeName(name): NodeName) -> Self {
        Cow::Owned(name)
    }
}

impl std::borrow::Borrow<str> for NodeName {
    fn borrow(&self) -> &str {
        &self.0
    }
}
