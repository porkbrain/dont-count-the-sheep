//! Framework for defining the layout of scenes.
//! Where can the character go? Where are the walls? Where are the immovable
//! objects?

use std::{marker::PhantomData, ops::RangeInclusive};

use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_grid_squared::{square, Square, SquareLayout};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use common_assets::RonLoader;
use common_ext::QueryExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{
    actor::{self, player},
    ActorMovementEvent,
};

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

    /// Path to the map .ron asset.
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
        let min_y = Self::layout().square_to_world_pos(square(0, bottom)).y;
        let max_y = Self::layout().square_to_world_pos(square(0, top)).y;

        min_y..=max_y
    }
}

/// Holds the tiles in a hash map.
#[derive(
    Asset, Resource, Serialize, Deserialize, Reflect, InspectorOptions, Default,
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
    Character(Entity),
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

        app.init_resource::<map_maker::TileMapMakerToolbar<T::LocalTileKind>>()
            .register_type::<map_maker::TileMapMakerToolbar<T::LocalTileKind>>()
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
    fn is_walkable(&self, by: Entity) -> bool {
        match self {
            Self::Empty => true,
            Self::Wall => false,
            Self::Trail => true,
            Self::Character(entity) if *entity == by => true,
            Self::Character(_) => false, // don't walk over others
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
            Self::Character(entity) if *entity == by => {
                Some(TileWalkCost::Normal)
            }
            Self::Character(_) => None, // don't walk over others
            Self::Local(l) => l.walk_cost(by),
        }
    }
}

impl<T: IntoMap> TileMap<T> {
    /// Get the kind of a tile.
    pub fn get(
        &self,
        square: &Square,
    ) -> Option<&[TileKind<T::LocalTileKind>]> {
        self.squares.get(square).map(|tiles| tiles.as_slice())
    }

    /// Whether the given square has the given kind of tile in any layer.
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
    pub fn is_walkable(&self, square: Square, by: Entity) -> bool {
        if let Some(tiles) = self.squares.get(&square) {
            tiles.iter().all(|tile| tile.is_walkable(by))
        } else {
            T::contains(square)
        }
    }

    /// For given square, find the first `None` tile or insert a new layer.
    /// Then return the index of the layer.
    ///
    /// Must not be called with `TileKind::None`.
    pub fn add_tile_to_first_empty_layer(
        &mut self,
        to: Square,
        tile: impl Into<TileKind<T::LocalTileKind>>,
    ) -> usize {
        let into_tile = tile.into();
        debug_assert_ne!(into_tile, TileKind::Empty);

        let tiles = self.squares.entry(to).or_default();
        tiles
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
            })
    }

    /// Set the kind of a tile in a specific layer.
    /// If the layer does not exist, it will be created.
    /// Returns the previous kind of the tile, or [`TileKind::Empty`] if it did
    /// not exist.
    pub fn set_tile_kind_layer(
        &mut self,
        of: Square,
        layer: usize,
        kind: impl Into<TileKind<T::LocalTileKind>>,
    ) -> TileKind<T::LocalTileKind> {
        let tiles = self.squares.entry(of).or_default();

        if tiles.len() <= layer {
            tiles.resize(layer + 1, TileKind::Empty);
        }

        let tile = &mut tiles[layer]; // safe cuz we just resized
        let current = *tile;
        *tile = kind.into();

        current
    }

    /// Returns [`None`] if not walkable, otherwise the cost of walking to the
    /// tile.
    /// This is useful for pathfinding.
    /// The higher the cost, the less likely the character will want to walk
    /// over it.
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
                square.neighbors().filter_map(|neighbor| {
                    self.walk_cost(neighbor, who)
                        .map(|cost| (neighbor, cost as i32))
                })
            },
            |square| square.manhattan_distance(to),
            |square| square == &to,
        )?;

        Some(path)
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
        cmd.insert_resource(map_assets.remove(map).unwrap()); // safe ^
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

#[cfg(feature = "dev")]
mod map_maker {
    use std::collections::BTreeMap;

    use bevy::{
        render::view::RenderLayers, utils::HashSet, window::PrimaryWindow,
    };
    use ron::ser::PrettyConfig;

    use super::*;

    #[derive(Component)]
    pub(super) struct SquareSprite(Square);

    #[derive(Resource, Reflect, InspectorOptions, Default)]
    #[reflect(Resource, InspectorOptions)]
    pub(super) struct TileMapMakerToolbar<L: Tile> {
        // these are configurable
        // ~
        // ~
        /// What kind of tile to paint.
        paint: TileKind<L>,
        /// Each square has an associated list of tiles.
        /// Layer refers to the index in this list.
        /// We only manipulate the indexes of the tiles that equal to the
        /// `layer`.
        layer: usize,
        /// If set to true, will replace any tile kind.
        /// If set to false, will only paint over tiles that are `None`.
        paint_over_tiles: bool,

        // these are metadata used by the system
        // ~
        // ~
        /// We paint rectangles with this tool.
        /// When you click on a tile, it will start painting from there.
        /// When you release the mouse, it will stop painting and draw a
        /// rectangle of the `paint` kind from here to where you
        /// released the mouse.
        begin_rect_at: Option<Square>,
    }

    pub(super) fn visualize_map<T: IntoMap>(
        mut cmd: Commands,
        map: Res<TileMap<T>>,
    ) {
        let root = cmd
            .spawn((
                Name::new("Debug Layout Grid"),
                SpatialBundle {
                    transform: Transform::from_translation(
                        Vec2::ZERO.extend(10.0),
                    ),
                    ..default()
                },
            ))
            .id();

        for square in bevy_grid_squared::shapes::rectangle(T::bounds()) {
            let world_pos = T::layout().square_to_world_pos(square);

            let kind = map
                .squares
                .get(&square)
                .and_then(|tiles| tiles.first()) // default to first layer
                .copied()
                .unwrap_or(TileKind::Empty);

            let child = cmd
                .spawn((SquareSprite(square), Name::new(format!("{square}"))))
                .insert(SpriteBundle {
                    sprite: Sprite {
                        color: kind.color(),
                        // slightly smaller to show borders
                        custom_size: Some(T::layout().square() - 0.25),
                        ..default()
                    },
                    transform: Transform::from_translation(
                        world_pos.extend(0.0),
                    ),
                    ..default()
                })
                .id();
            cmd.entity(root).add_child(child);
        }
    }

    pub(super) fn change_square_kind<T: IntoMap>(
        mouse: Res<Input<MouseButton>>,
        mut map: ResMut<TileMap<T>>,
        mut toolbar: ResMut<TileMapMakerToolbar<T::LocalTileKind>>,
        keyboard: Res<Input<KeyCode>>,

        windows: Query<&Window, With<PrimaryWindow>>,
        cameras: Query<(&Camera, &GlobalTransform, Option<&RenderLayers>)>,
    ) {
        let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft);
        let just_pressed_left = mouse.just_pressed(MouseButton::Left);
        let just_released_left = mouse.just_released(MouseButton::Left);
        let just_pressed_right = mouse.just_pressed(MouseButton::Right);

        // a) hold ctrl + press left to paint rect
        let start_painting_rect = ctrl_pressed
            && just_pressed_left
            && toolbar.begin_rect_at.is_none();
        // b) if painting rect, release left to stop painting
        let stop_painting_rect =
            toolbar.begin_rect_at.is_some() && just_released_left;
        // c) press right to paint single square
        let paint_single_square = just_pressed_right;

        // if neither of these, then early return
        if !start_painting_rect && !stop_painting_rect && !paint_single_square {
            return;
        }

        let Some(clicked_at) = cursor_to_square(T::layout(), windows, cameras)
        else {
            return;
        };

        if start_painting_rect {
            toolbar.begin_rect_at = Some(clicked_at);
        } else if stop_painting_rect
            && let Some(begin_rect_at) = toolbar.begin_rect_at.take()
        {
            trace!("Painting rect from {begin_rect_at} to {clicked_at}");
            for square in selection_rect(begin_rect_at, clicked_at) {
                let tiles = map.squares.entry(square).or_default();
                if tiles.len() <= toolbar.layer {
                    tiles.resize(toolbar.layer + 1, TileKind::Empty);
                }

                if toolbar.paint_over_tiles
                    || tiles[toolbar.layer] == TileKind::Empty
                {
                    tiles[toolbar.layer] = toolbar.paint;
                }
            }
        } else if paint_single_square {
            let tiles = map.squares.entry(clicked_at).or_default();
            if tiles.len() <= toolbar.layer {
                tiles.resize(toolbar.layer + 1, TileKind::Empty);
            }

            if toolbar.paint_over_tiles
                || tiles[toolbar.layer] == TileKind::Empty
            {
                tiles[toolbar.layer] = toolbar.paint;
            }
        }
    }

    pub(super) fn recolor_squares<T: IntoMap>(
        map: ResMut<TileMap<T>>,
        toolbar: Res<TileMapMakerToolbar<T::LocalTileKind>>,

        mut squares: Query<(&SquareSprite, &mut Sprite)>,
        windows: Query<&Window, With<PrimaryWindow>>,
        cameras: Query<(&Camera, &GlobalTransform, Option<&RenderLayers>)>,
    ) {
        let squares_painted: Option<HashSet<_>> =
            toolbar.begin_rect_at.and_then(|begin_rect_at| {
                let clicked_at =
                    cursor_to_square(T::layout(), windows, cameras)?;

                Some(selection_rect(begin_rect_at, clicked_at).collect())
            });

        for (SquareSprite(at), mut sprite) in squares.iter_mut() {
            let tile_kind = map
                .squares
                .get(at)
                .and_then(|tiles| tiles.get(toolbar.layer))
                .copied()
                .unwrap_or_default();

            // show where we're painting unless we're not allowed to
            // paint over tiles
            let color = if squares_painted
                .as_ref()
                .map_or(false, |s| s.contains(at))
                && (toolbar.paint_over_tiles || tile_kind == TileKind::Empty)
            {
                toolbar.paint.color()
            } else {
                tile_kind.color()
            };

            sprite.color = color;
        }
    }

    // TODO: only store what the user has changed with the map editor
    pub(super) fn export_map<T: IntoMap>(mut map: ResMut<TileMap<T>>) {
        // filter out needless squares
        map.squares.retain(|_, v| {
            v.iter_mut().for_each(|tile| {
                if matches!(tile, TileKind::Character(_)) {
                    *tile = TileKind::Empty;
                }
            });

            while v.ends_with(&[TileKind::Empty]) {
                v.pop();
            }

            !v.is_empty()
        });

        // equivalent to tile map, but sorted so that we can serialize it
        // and the output is deterministic
        //
        // this struct MUST serialize to a compatible ron output as TileMap
        #[derive(Serialize)]
        struct SortedTileMap<T: IntoMap> {
            squares:
                BTreeMap<Square, SmallVec<[TileKind<T::LocalTileKind>; 3]>>,
            #[serde(skip)]
            _phantom: PhantomData<T>,
        }

        let tilemap_but_sorted: SortedTileMap<T> = SortedTileMap {
            squares: map.squares.clone().into_iter().collect(),
            _phantom: default(),
        };

        // for internal use only so who cares
        std::fs::write(
            "map.ron",
            ron::ser::to_string_pretty(
                &tilemap_but_sorted,
                PrettyConfig::default()
                    .compact_arrays(true)
                    .separate_tuple_members(false)
                    .indentor(" ".to_string())
                    .depth_limit(2),
            )
            .unwrap(),
        )
        .unwrap();
    }

    impl<L> TileKind<L> {
        fn color(self) -> Color {
            match self {
                Self::Empty => Color::BLACK.with_a(0.25),
                Self::Wall => Color::BLACK.with_a(0.8),
                Self::Trail => Color::WHITE.with_a(0.25),
                Self::Character(_) => Color::GOLD.with_a(0.25),
                Self::Local(_) => Color::RED.with_a(0.25),
            }
        }
    }

    fn selection_rect(
        begin_rect_at: Square,
        clicked_at: Square,
    ) -> impl ExactSizeIterator<Item = Square> {
        let left = begin_rect_at.x.min(clicked_at.x);
        let right = begin_rect_at.x.max(clicked_at.x);
        let bottom = begin_rect_at.y.min(clicked_at.y);
        let top = begin_rect_at.y.max(clicked_at.y);

        bevy_grid_squared::shapes::rectangle([left, right, bottom, top])
    }

    fn cursor_to_square(
        layout: &SquareLayout,
        windows: Query<&Window, With<PrimaryWindow>>,
        cameras: Query<(&Camera, &GlobalTransform, Option<&RenderLayers>)>,
    ) -> Option<Square> {
        let cursor_pos = windows.single().cursor_position()?;

        let (camera, camera_transform, _) = cameras
            .iter()
            .filter(|(_, _, l)| {
                l.map(|l| l.intersects(&RenderLayers::layer(0)))
                    .unwrap_or(true)
            })
            .next()?;
        let world_pos =
            camera.viewport_to_world_2d(camera_transform, cursor_pos)?;

        Some(layout.world_pos_to_square(world_pos))
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
            "test_scene.ron"
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

        let sq = square(0, 0);
        let mut tilemap = TileMap::<TestScene>::default();

        // out of bounds returns none
        assert_eq!(tilemap.walk_cost(square(-1, 0), Entity::PLACEHOLDER), None);

        // in bounds, but no tile returns normal
        assert_eq!(tilemap.walk_cost(sq, Entity::PLACEHOLDER), Some(Normal));

        // if there's a wall returns none
        tilemap.squares.insert(sq, smallvec![Tk::Empty, Tk::Wall]);
        assert_eq!(tilemap.walk_cost(sq, Entity::PLACEHOLDER), None);
        tilemap.squares.insert(sq, smallvec![Tk::Wall, Tk::Empty]);
        assert_eq!(tilemap.walk_cost(sq, Entity::PLACEHOLDER), None);

        // if there's a trail returns preferred
        tilemap.squares.insert(sq, smallvec![Tk::Empty, Tk::Trail]);
        assert_eq!(tilemap.walk_cost(sq, Entity::PLACEHOLDER), Some(Preferred));
        tilemap.squares.insert(sq, smallvec![Tk::Trail, Tk::Empty,]);
        assert_eq!(tilemap.walk_cost(sq, Entity::PLACEHOLDER), Some(Preferred));

        // if there's a matching character and trail, returns preferred
        tilemap.squares.insert(
            sq,
            smallvec![Tk::Character(Entity::PLACEHOLDER), Tk::Trail],
        );
        assert_eq!(tilemap.walk_cost(sq, Entity::PLACEHOLDER), Some(Preferred));

        // if there's a matching character and wall, returns none
        tilemap.squares.insert(
            sq,
            smallvec![Tk::Character(Entity::PLACEHOLDER), Tk::Wall],
        );
        assert_eq!(tilemap.walk_cost(sq, Entity::PLACEHOLDER), None);

        // if there's a different character and a trail, returns none
        tilemap.squares.insert(
            sq,
            smallvec![Tk::Character(Entity::from_raw(10)), Tk::Trail],
        );
        assert_eq!(tilemap.walk_cost(sq, Entity::PLACEHOLDER), None);
    }

    #[test]
    fn it_adds_tiles_to_first_empty_layer() {
        let mut tilemap = TileMap::<TestScene>::default();
        let sq = square(0, 0);

        assert_eq!(
            tilemap.add_tile_to_first_empty_layer(sq, TileKind::Wall),
            0
        );
        assert_eq!(
            tilemap.add_tile_to_first_empty_layer(sq, TileKind::Wall),
            1
        );

        tilemap.squares.insert(
            sq,
            smallvec![TileKind::Wall, TileKind::Empty, TileKind::Wall],
        );
        assert_eq!(
            tilemap.add_tile_to_first_empty_layer(sq, TileKind::Wall),
            1
        );
    }

    #[test]
    fn it_sets_tile_kind_layer() {
        let mut tilemap = TileMap::<TestScene>::default();
        let sq = square(0, 0);

        assert_eq!(
            tilemap.set_tile_kind_layer(sq, 0, TileKind::Wall),
            TileKind::Empty
        );
        assert_eq!(
            tilemap.set_tile_kind_layer(sq, 0, TileKind::Wall),
            TileKind::Wall
        );
        assert_eq!(
            tilemap.set_tile_kind_layer(sq, 1, TileKind::Wall),
            TileKind::Empty
        );
        assert_eq!(
            tilemap.set_tile_kind_layer(sq, 1, TileKind::Wall),
            TileKind::Wall
        );
        assert_eq!(
            tilemap.set_tile_kind_layer(sq, 5, TileKind::Wall),
            TileKind::Empty
        );
        assert_eq!(
            tilemap.set_tile_kind_layer(sq, 4, TileKind::Wall),
            TileKind::Empty
        );
    }
}
