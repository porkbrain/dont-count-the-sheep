//! Framework for defining the layout of scenes.
//! Where can the character go? Where are the walls? Where are the immovable
//! objects?

#[cfg(feature = "dev")]
mod map_maker;

use std::{marker::PhantomData, ops::RangeInclusive};

use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_grid_squared::{sq, Square, SquareLayout};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use common_assets::RonLoader;
use common_ext::QueryExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{
    actor::{self, player},
    ActorMovementEvent,
};

/// A tile is uniquely identified by (`x`, `y`) of the square and a layer index.
pub type TileIndex = (Square, usize);

/// Some map.
pub trait IntoMap: 'static + Send + Sync + TypePath + Default {
    /// Tile kind that is unique to this map.
    /// Will parametrize the [`TileKind::Local`] enum's variant.
    ///
    /// If the map has some sort of special tiles, use an enum here.
    /// Otherwise, set to unit type.
    type LocalTileKind: Tile;

    /// Size in number of tiles.
    /// `[left, right, bottom, top]`
    fn bounds() -> [i32; 4];

    /// How large is a tile and how do we translate between world coordinates
    /// and tile coordinates?
    fn layout() -> &'static SquareLayout;

    /// Path to the map .ron asset relative to the assets directory.
    fn asset_path() -> &'static str;

    /// Given a position on the map, add a z coordinate.
    #[inline]
    fn extend_z(Vec2 { x, y }: Vec2) -> Vec3 {
        let (min, max) = Self::y_range().into_inner();
        let size = max - min;
        debug_assert!(size > 0.0, "{max} - {min} <= 0.0");

        // we allow for a tiny leeway for positions outside of the bounding box
        let z = ((max - y) / size).clamp(-0.1, 1.1);

        Vec3::new(x, y, z)
    }

    /// Given a position on the map, add a z coordinate as if the y coordinate
    /// was offset by `offset`.
    fn extend_z_with_y_offset(Vec2 { x, y }: Vec2, offset: f32) -> Vec3 {
        let z = Self::extend_z(Vec2 { x, y: y + offset }).z;
        Vec3::new(x, y, z)
    }

    /// Whether the given square is inside the map.
    #[inline]
    fn contains(square: Square) -> bool {
        let [min_x, max_x, min_y, max_y] = Self::bounds();

        square.x >= min_x
            && square.x <= max_x
            && square.y >= min_y
            && square.y <= max_y
    }

    /// Range of y world pos coordinates.
    fn y_range() -> RangeInclusive<f32> {
        let [_, _, bottom, top] = Self::bounds();
        let min_y = Self::layout().square_to_world_pos(sq(0, bottom)).y;
        let max_y = Self::layout().square_to_world_pos(sq(0, top)).y;

        min_y..=max_y
    }
}

/// Holds the tiles in a hash map.
#[derive(
    Asset,
    Resource,
    Serialize,
    Deserialize,
    Reflect,
    InspectorOptions,
    Default,
    Clone,
    Debug,
)]
#[reflect(Resource, InspectorOptions)]
pub struct TileMap<T: IntoMap> {
    /// There can be multiple layers of tiles on a single square.
    squares: HashMap<Square, SmallVec<[TileKind<T::LocalTileKind>; 3]>>,
    #[serde(skip)]
    #[reflect(ignore)]
    _phantom: PhantomData<T>,
}

/// What kind of tiles do we support?
///
/// Each map can have its own unique `L`ocal tiles.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
    Reflect,
    Serialize,
)]
#[reflect(Default)]
pub enum TileKind<L> {
    /// No tile.
    #[default]
    Empty,
    /// A wall that cannot be passed.
    /// Can be actual wall, an object etc.
    Wall,
    /// NPCs will preferably follow the trail when moving.
    Trail,
    /// Personal space of a character.
    /// A single character will be assigned to multiple tiles based on their
    /// size.
    ///
    /// We use [`Entity`] to make it apparent that this will be dynamically
    /// updated on runtime.
    /// This variant mustn't be loaded from map ron file.
    ///
    /// OPTIMIZE: To reduce storage overhead, we could store the entity in an
    /// array on the tilemap and use a u32 index to reference it here.
    /// Getting rid of 4 bytes per tile would mean we'd fetch 12 less bytes on
    /// each square access.
    /// The entity array access should be cheap.
    Actor(Entity),
    /// Specific for a given map.
    Local(L),
}

/// Defines tile behavior.
pub trait Tile:
    TypePath
    + Clone
    + Copy
    + Default
    + DeserializeOwned
    + Eq
    + FromReflect
    + PartialEq
    + Serialize
    + std::fmt::Debug
    + std::hash::Hash
{
    /// Whether the tile can be stepped on by an actor with given entity.
    fn is_walkable(&self, by: Entity) -> bool;

    /// Whether a tile represents a zone.
    /// A zone is a group of tiles that are connected to each other and entities
    /// enter and leave them.
    /// This is used to emit events about entering/leaving zones.
    fn is_zone(&self) -> bool;

    /// Returns [`None`] if not walkable, otherwise the cost of walking to the
    /// tile.
    /// This is useful for pathfinding.
    /// The higher the cost, the less likely the character will want to walk
    /// over it.
    fn walk_cost(&self, by: Entity) -> Option<TileWalkCost> {
        self.is_walkable(by).then_some(TileWalkCost::Normal)
    }
}

/// Useful for pathfinding to prefer some tiles over others.
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TileWalkCost {
    /// The tile is preferred to be walked over.
    Preferred = 1,
    /// The tile is normal to walk over.
    #[default]
    Normal = 3,
}

/// Registers layout map for `T` where `T` is a type implementing [`IntoMap`].
/// This would be your level layout.
/// When [`crate::Actor`]s enter a zone within the map,
/// [`crate::ActorMovementEvent`] event is emitted.
///
/// If the `dev` feature is enabled, you can press `Enter` to export the map
/// to `map.ron` in the current directory.
/// We draw an overlay with tiles that you can edit with left and right mouse
/// buttons.
pub fn register<T: IntoMap, S: States>(app: &mut App, loading: S, running: S) {
    app.init_asset_loader::<RonLoader<TileMap<T>>>()
        .init_asset::<TileMap<T>>()
        .register_type::<TileKind<T::LocalTileKind>>()
        .register_type::<TileMap<T>>()
        .register_type::<RonLoader<TileMap<T>>>()
        .add_event::<ActorMovementEvent<T::LocalTileKind>>()
        .register_type::<ActorMovementEvent<T::LocalTileKind>>();

    app.add_systems(OnEnter(loading.clone()), start_loading_map::<T>)
        .add_systems(
            First,
            try_insert_map_as_resource::<T>.run_if(in_state(loading)),
        )
        .add_systems(
            Update,
            actor::emit_movement_events::<T>
                .run_if(in_state(running.clone()))
                // so that we can emit this event on current frame
                .after(player::move_around::<T>),
        )
        .add_systems(OnExit(running.clone()), remove_resources::<T>);

    #[cfg(feature = "dev")]
    {
        use bevy::input::common_conditions::input_just_pressed;
        use bevy_inspector_egui::quick::ResourceInspectorPlugin;

        // we insert the toolbar along with the map
        app.register_type::<map_maker::TileMapMakerToolbar<T::LocalTileKind>>()
            .add_plugins(ResourceInspectorPlugin::<
                map_maker::TileMapMakerToolbar<T::LocalTileKind>,
            >::default());

        app.add_systems(
            OnEnter(running.clone()),
            map_maker::visualize_map::<T>,
        );
        app.add_systems(
            Update,
            (
                map_maker::change_square_kind::<T>,
                map_maker::recolor_squares::<T>,
            )
                .run_if(in_state(running.clone()))
                .chain(),
        );
        app.add_systems(
            Update,
            map_maker::export_map::<T>
                .run_if(input_just_pressed(KeyCode::Return))
                .run_if(in_state(running)),
        );
    }
}

/// Allow implementation for unit type for convenience.
/// Maps can use this if they have no special tiles.
impl Tile for () {
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    fn is_zone(&self) -> bool {
        false
    }
}

impl<L: Tile> Tile for TileKind<L> {
    #[inline]
    fn is_walkable(&self, by: Entity) -> bool {
        match self {
            Self::Empty => true,
            Self::Wall => false,
            Self::Trail => true,
            Self::Actor(entity) if *entity == by => true,
            Self::Actor(_) => false, // don't walk over others
            Self::Local(l) => l.is_walkable(by),
        }
    }

    fn is_zone(&self) -> bool {
        match self {
            Self::Local(l) => l.is_zone(),
            _ => false,
        }
    }

    fn walk_cost(&self, by: Entity) -> Option<TileWalkCost> {
        match self {
            Self::Wall => None,
            Self::Empty => Some(TileWalkCost::Normal),
            Self::Trail => Some(TileWalkCost::Preferred),
            Self::Actor(entity) if *entity == by => Some(TileWalkCost::Normal),
            Self::Actor(_) => None, // don't walk over others
            Self::Local(l) => l.walk_cost(by),
        }
    }
}

impl<T: IntoMap> TileMap<T> {
    /// Get the kind of a tile.
    #[inline]
    pub fn get(&self, square: Square) -> Option<&[TileKind<T::LocalTileKind>]> {
        if !T::contains(square) {
            return None;
        }

        self.squares.get(&square).map(|tiles| tiles.as_slice())
    }

    /// Whether the given square has the given kind of tile in any layer.
    #[inline]
    pub fn is_on(
        &self,
        square: Square,
        kind: impl Into<TileKind<T::LocalTileKind>>,
    ) -> bool {
        let kind = kind.into();
        self.squares
            .get(&square)
            .map(|tiles| tiles.contains(&kind))
            .unwrap_or(false)
    }

    /// Whether there's something on the given square that cannot be walked over
    /// such as a wall, an object or a character.
    /// Also checks bounds.
    #[inline]
    pub fn is_walkable(&self, square: Square, by: Entity) -> bool {
        if let Some(tiles) = self.squares.get(&square) {
            tiles.iter().all(|tile| tile.is_walkable(by))
        } else {
            T::contains(square)
        }
    }

    /// Whether the predicate matches any tile on the given square.
    /// Returns `false` if the square is out of bounds or has no tiles.
    #[inline]
    pub fn any_on(
        &self,
        square: Square,
        predicate: impl Fn(TileKind<T::LocalTileKind>) -> bool,
    ) -> bool {
        self.squares
            .get(&square)
            .map(|tiles| tiles.iter().any(|tile| predicate(*tile)))
            .unwrap_or(false)
    }

    /// Whether the predicate matches all tiles on the given square.
    /// Returns `false` if the square is out of bounds or has no tiles.
    #[inline]
    pub fn all_on(
        &self,
        square: Square,
        predicate: impl Fn(TileKind<T::LocalTileKind>) -> bool,
    ) -> bool {
        self.squares
            .get(&square)
            .map(|tiles| tiles.iter().all(|tile| predicate(*tile)))
            .unwrap_or(false)
    }

    /// For given square, find the first `None` tile or insert a new layer.
    /// Then return the index of the layer.
    ///
    /// Must not be called with `TileKind::None`.
    #[inline]
    pub fn add_tile_to_first_empty_layer(
        &mut self,
        to: Square,
        tile: impl Into<TileKind<T::LocalTileKind>>,
    ) -> Option<usize> {
        if !T::contains(to) {
            return None;
        }

        let into_tile = tile.into();
        debug_assert_ne!(into_tile, TileKind::Empty);

        let tiles = self.squares.entry(to).or_default();
        let layer = tiles
            .iter_mut()
            .enumerate()
            .find_map(|(index, tile)| {
                if *tile == TileKind::Empty {
                    *tile = into_tile;
                    Some(index)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                let layer = tiles.len();
                tiles.push(into_tile);
                layer
            });

        Some(layer)
    }

    /// Set the kind of a tile in a specific layer.
    /// If the layer does not exist, it will be created.
    /// Returns the previous kind of the tile, or [`TileKind::Empty`] if it did
    /// not exist.
    #[inline]
    pub fn set_tile_kind(
        &mut self,
        of: Square,
        layer: usize,
        kind: impl Into<TileKind<T::LocalTileKind>>,
    ) -> Option<TileKind<T::LocalTileKind>> {
        if !T::contains(of) {
            return None;
        }

        let tiles = self.squares.entry(of).or_default();

        if tiles.len() <= layer {
            tiles.resize(layer + 1, TileKind::Empty);
        }

        let tile = &mut tiles[layer]; // safe cuz we just resized
        let current = *tile;
        *tile = kind.into();

        Some(current)
    }

    /// Map each tile on the given square to the given kind.
    #[inline]
    pub fn map_tiles(
        &mut self,
        of: Square,
        map: impl Fn(TileKind<T::LocalTileKind>) -> TileKind<T::LocalTileKind>,
    ) {
        let Some(tiles) = self.squares.get_mut(&of) else {
            return;
        };

        for tile in tiles.iter_mut() {
            *tile = map(*tile);
        }
    }

    /// Returns [`None`] if not walkable, otherwise the cost of walking to the
    /// tile.
    /// This is useful for pathfinding.
    /// The higher the cost, the less likely the character will want to walk
    /// over it.
    #[inline]
    pub fn walk_cost(
        &self,
        square: Square,
        by: Entity,
    ) -> Option<TileWalkCost> {
        if let Some(tiles) = self.squares.get(&square) {
            // return the lowest cost unless any of the tiles is not walkable
            tiles.iter().try_fold(
                TileWalkCost::Normal,
                |highest_cost_so_far, tile| {
                    Some(tile.walk_cost(by)?.min(highest_cost_so_far))
                },
            )
        } else if T::contains(square) {
            Some(TileWalkCost::Normal)
        } else {
            None
        }
    }

    /// Uses A* to find a path from `from` to `to`.
    ///
    /// TODO: constraint this to run only for X steps in a single frame to
    /// avoid clogging the CPU, or even run it outside of schedule
    pub fn find_path(
        &self,
        who: Entity,
        from: Square,
        to: Square,
    ) -> Option<Vec<Square>> {
        let (path, _cost) = pathfinding::prelude::astar(
            &from,
            |square| {
                square.neighbors_with_diagonal().filter_map(|neighbor| {
                    self.walk_cost(neighbor, who)
                        .map(|cost| (neighbor, cost as i32))
                })
            },
            |square| square.manhattan_distance(to),
            |square| square == &to,
        )?;

        Some(path)
    }

    /// Access the map of squares to tiles.
    pub fn squares(
        &self,
    ) -> &HashMap<Square, SmallVec<[TileKind<T::LocalTileKind>; 3]>> {
        &self.squares
    }
}

impl<L> TileKind<L> {
    /// If the tile is local, returns it.
    pub fn into_local(self) -> Option<L> {
        match self {
            Self::Local(l) => Some(l),
            _ => None,
        }
    }
}

/// Tells the game to start loading the map.
/// We need to keep checking for this to be done by calling
/// [`try_insert_map_as_resource`].
fn start_loading_map<T: IntoMap>(mut cmd: Commands, assets: Res<AssetServer>) {
    let handle: Handle<TileMap<T>> = assets.load(T::asset_path());
    cmd.spawn(handle);
}

/// Run this to wait for the map to be loaded and insert it as a resource.
/// Call it after [`start_loading_map`].
/// Idempotent.
///
/// You should then check for the map as a resource in your systems and continue
/// with your game.
fn try_insert_map_as_resource<T: IntoMap>(
    mut cmd: Commands,
    mut map_assets: ResMut<Assets<TileMap<T>>>,
    map: Query<(Entity, &Handle<TileMap<T>>)>,
) {
    let Some((entity, map)) = map.get_single_or_none() else {
        // if the map does no longer exist as a component handle, we either did
        // not spawn it or it's already a resource
        // the caller should check for the latter
        return;
    };

    // we cannot call remove straight away because panics - the handle is
    // removed, the map is not loaded yet and asset loader expects it to exist
    if map_assets.get(map).is_some() {
        let loaded_map = map_assets.remove(map).unwrap(); // safe ^

        #[cfg(feature = "dev")]
        {
            // include the loaded map in the toolbar, which will allow us to
            // store ONLY user changes, not dynamic changes made by the logic
            cmd.insert_resource(map_maker::TileMapMakerToolbar::new(
                loaded_map.squares.clone(),
            ));
        }

        cmd.insert_resource(loaded_map);
        cmd.init_resource::<actor::ActorZoneMap<T::LocalTileKind>>();
        cmd.entity(entity).despawn();
    }
}

fn remove_resources<T: IntoMap>(mut cmd: Commands) {
    cmd.remove_resource::<TileMap<T>>();
    cmd.remove_resource::<actor::ActorZoneMap<T::LocalTileKind>>();

    #[cfg(feature = "dev")]
    {
        cmd.remove_resource::<map_maker::TileMapMakerToolbar<T::LocalTileKind>>();
    }
}

impl<L> From<L> for TileKind<L> {
    fn from(l: L) -> Self {
        Self::Local(l)
    }
}

#[cfg(test)]
mod tests {
    use smallvec::smallvec;

    use super::*;

    #[derive(Default, Reflect)]
    struct TestScene;

    impl IntoMap for TestScene {
        type LocalTileKind = ();

        fn bounds() -> [i32; 4] {
            [0, 10, 0, 10]
        }

        fn layout() -> &'static SquareLayout {
            &SquareLayout {
                square_size: 1.0,
                origin: Vec2::ZERO,
            }
        }

        fn asset_path() -> &'static str {
            unreachable!()
        }
    }

    #[test]
    fn it_converts_tile_walk_cost_to_i32() {
        assert_eq!(TileWalkCost::Preferred as i32, 1);
        assert_eq!(TileWalkCost::Normal as i32, 3);
    }

    #[test]
    fn it_calculates_walk_cost() {
        use TileKind as Tk;
        use TileWalkCost::*;

        let o = sq(0, 0);
        let mut tilemap = TileMap::<TestScene>::default();

        // out of bounds returns none
        assert_eq!(tilemap.walk_cost(sq(-1, 0), Entity::PLACEHOLDER), None);

        // in bounds, but no tile returns normal
        assert_eq!(tilemap.walk_cost(o, Entity::PLACEHOLDER), Some(Normal));

        // if there's a wall returns none
        tilemap.squares.insert(o, smallvec![Tk::Empty, Tk::Wall]);
        assert_eq!(tilemap.walk_cost(o, Entity::PLACEHOLDER), None);
        tilemap.squares.insert(o, smallvec![Tk::Wall, Tk::Empty]);
        assert_eq!(tilemap.walk_cost(o, Entity::PLACEHOLDER), None);

        // if there's a trail returns preferred
        tilemap.squares.insert(o, smallvec![Tk::Empty, Tk::Trail]);
        assert_eq!(tilemap.walk_cost(o, Entity::PLACEHOLDER), Some(Preferred));
        tilemap.squares.insert(o, smallvec![Tk::Trail, Tk::Empty,]);
        assert_eq!(tilemap.walk_cost(o, Entity::PLACEHOLDER), Some(Preferred));

        // if there's a matching character and trail, returns preferred
        tilemap
            .squares
            .insert(o, smallvec![Tk::Actor(Entity::PLACEHOLDER,), Tk::Trail]);
        assert_eq!(tilemap.walk_cost(o, Entity::PLACEHOLDER), Some(Preferred));

        // if there's a matching character and wall, returns none
        tilemap
            .squares
            .insert(o, smallvec![Tk::Actor(Entity::PLACEHOLDER,), Tk::Wall]);
        assert_eq!(tilemap.walk_cost(o, Entity::PLACEHOLDER), None);

        // if there's a different character and a trail, returns none
        tilemap
            .squares
            .insert(o, smallvec![Tk::Actor(Entity::from_raw(10),), Tk::Trail]);
        assert_eq!(tilemap.walk_cost(o, Entity::PLACEHOLDER), None);
    }

    #[test]
    fn it_adds_tiles_to_first_empty_layer() {
        let mut tilemap = TileMap::<TestScene>::default();
        let sq = sq(0, 0);

        assert_eq!(
            tilemap.add_tile_to_first_empty_layer(sq, TileKind::Wall),
            Some(0)
        );
        assert_eq!(
            tilemap.add_tile_to_first_empty_layer(sq, TileKind::Wall),
            Some(1)
        );

        tilemap.squares.insert(
            sq,
            smallvec![TileKind::Wall, TileKind::Empty, TileKind::Wall],
        );
        assert_eq!(
            tilemap.add_tile_to_first_empty_layer(sq, TileKind::Wall),
            Some(1)
        );
    }

    #[test]
    fn it_sets_tile_kind_layer() {
        let mut tilemap = TileMap::<TestScene>::default();
        let sq = sq(0, 0);

        assert_eq!(
            tilemap.set_tile_kind(sq, 0, TileKind::Wall),
            Some(TileKind::Empty)
        );
        assert_eq!(
            tilemap.set_tile_kind(sq, 0, TileKind::Wall),
            Some(TileKind::Wall)
        );
        assert_eq!(
            tilemap.set_tile_kind(sq, 1, TileKind::Wall),
            Some(TileKind::Empty)
        );
        assert_eq!(
            tilemap.set_tile_kind(sq, 1, TileKind::Wall),
            Some(TileKind::Wall)
        );
        assert_eq!(
            tilemap.set_tile_kind(sq, 5, TileKind::Wall),
            Some(TileKind::Empty)
        );
        assert_eq!(
            tilemap.set_tile_kind(sq, 4, TileKind::Wall),
            Some(TileKind::Empty)
        );
    }

    #[test]
    fn it_doesnt_do_anything_outside_map_bounds() {
        let mut tilemap = TileMap::<TestScene>::default();

        assert_eq!(
            tilemap.add_tile_to_first_empty_layer(sq(-100, 0), TileKind::Wall),
            None
        );

        assert_eq!(tilemap.set_tile_kind(sq(100, 0), 0, TileKind::Wall), None);
    }

    #[derive(
        Default,
        Reflect,
        Hash,
        PartialEq,
        Eq,
        Debug,
        Serialize,
        Deserialize,
        Clone,
        Copy,
    )]
    struct TestTileKind;

    impl Tile for TestTileKind {
        fn is_walkable(&self, _: Entity) -> bool {
            true
        }

        fn is_zone(&self) -> bool {
            false
        }
    }

    #[test]
    fn it_has_small_size_of_tilekind() {
        assert_eq!(std::mem::size_of::<TileKind<TestTileKind>>(), 12);

        let square: SmallVec<[TileKind<TestTileKind>; 3]> =
            smallvec![default(), default(), default(), default(), default()];
        assert_eq!(std::mem::size_of_val(&square), 48);
    }
}
