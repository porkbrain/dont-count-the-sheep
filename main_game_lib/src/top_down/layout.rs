//! Framework for defining the layout of scenes.
//! Where can the character go? Where are the walls? Where are the immovable
//! objects?

#[cfg(feature = "devtools")]
mod build_pathfinding_graph;
#[cfg(feature = "devtools")]
pub(crate) mod map_maker;
pub(crate) mod systems;

use bevy::{
    asset::Asset,
    ecs::{entity::Entity, system::Resource},
    log::{trace, warn},
    math::{vec2, Vec2},
    prelude::ReflectDefault,
    reflect::Reflect,
    utils::hashbrown::HashMap,
};
use bevy_grid_squared::{Square, SquareLayout};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use strum::IntoEnumIterator;

/// Each scene adheres to the same layout definition.
/// That's because the amount of space the character takes in the tile grid
/// is constant and tailored to the square size.
pub const LAYOUT: SquareLayout = SquareLayout {
    square_size: 4.0,
    // an arbitrary origin
    origin: vec2(36.0, 4.0),
};

/// A tile is uniquely identified by (`x`, `y`) of the square and a layer index.
pub type TileIndex = (Square, usize);

/// Holds the tiles in a hash map.
#[derive(Asset, Resource, Serialize, Deserialize, Reflect, Clone, Debug)]
pub struct TileMap {
    /// Size in number of tiles.
    /// `[left, right, bottom, top]`
    #[serde(default = "default_bounds")]
    bounds: [i32; 4],
    /// Metadata about zones used for pathfinding.
    #[serde(default)]
    zones: TileKindMetas,
    /// There can be multiple layers of tiles on a single square.
    pub(crate) squares: HashMap<Square, SmallVec<[TileKind; 3]>>,
}

/// You can change these in the .ron file of the map if you need larger map.
/// `[left, right, bottom, top]`
fn default_bounds() -> [i32; 4] {
    [-1_000, 1_000, -1_000, 1_000]
}

/// Maps a tile kind to its metadata that's useful for NPC pathfinding.
#[derive(Serialize, Deserialize, Reflect, Default, Clone, Debug)]
struct TileKindMetas {
    /// We could also use a vector and index is with some sort of conversion
    /// from the enum to usize.
    #[serde(default)]
    inner: HashMap<TileKind, TileKindMeta>,
}

/// These values are calculated when the map maker exports the map.
#[derive(Serialize, Deserialize, Reflect, Default, Clone, Debug)]
struct TileKindMeta {
    #[serde(default)]
    zone_group: ZoneGroup,
    #[serde(default)]
    zone_size: usize,
    #[serde(default)]
    zone_successors: SmallVec<[TileKind; 5]>,
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
    Ord,
    PartialOrd,
    Serialize,
)]
#[reflect(Default)]
pub enum TileKind {
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
    /// UPDATE: With bevy 0.13 the entity alignment has been changed,
    /// storing entity in an enum is more expensive than before.
    Actor(Entity),
    /// Specific for a given map.
    Zone(ZoneTileKind),
}

/// Named zones in the .ron files that are used in particular's scene logic to
/// achieve some behavior specific to that scene.
/// The zones can be re-used in multiple scenes.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Reflect,
    Serialize,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
)]
#[reflect(Default)]
#[allow(missing_docs)]
pub enum ZoneTileKind {
    /// Arbitrarily the default value.
    #[default]
    Exit,

    /// In interior scenes there might be multiple apartments.
    /// This represents a second apartment in such scenes.
    Apartment2,
    Apartment2BathroomDoor,
    Apartment2Door,

    // Some standard zones for interior scenes.
    Bed,
    Door,
    Elevator,
    Hallway,

    // Entrances to places
    Building1Entrance,
    ClinicEntrance,
    ClinicWardEntrance,
    CompoundEntrance,
    GoToDowntown,
    MallEntrance,
    PlantShopEntrance,
    SewersEntrance,
    TowerEntrance,
    TwinpeaksApartmentEntrance,

    // Mall zones
    Aisle1,
    Aisle2,
    Aisle3,
    Aisle4,
    Fridges,
    FruitsAndVeggies,
    GoodWater,

    // Player apartment zones
    PlayerApartment,
    Meditation,
    PlayerDoor,
    Tea,

    // Others
    BasementDoor,
    UpperApartmentDoor,
    UpperApartmentWallHidden,
    NearbyOcean,
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

/// Group zones into zones that are connected to each other.
/// This means that if zone A overlaps, neighbors is a subset or superset
/// of zone B, they both belong to the same group.
/// Ie. if there's a set of edges that lead from zone A to zone B, they are
/// in the same group.
///
/// The usize is an opaque unique value assigned to the group with no meaning.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Reflect, Serialize,
)]
struct ZoneGroup(usize);

/// Helper function that exports z coordinate given y coordinate.
///
/// It's domain in pixels is from -100_000 to 100_000.
///
/// It's range is from -0.1 to 1.1.
pub fn ysort(Vec2 { y, .. }: Vec2) -> f32 {
    // it's easier to just hardcode the range than pass around values
    //
    // this gives us sufficient buffer for maps of all sizes
    let (min, max) = (-100_000.0, 100_000.0);
    let size = max - min;

    // we allow for a tiny leeway for positions outside of the bounding box
    ((max - y) / size).clamp(-0.1, 1.1)
}

impl TileKind {
    #[inline]
    /// Whether the tile can be stepped on by an actor with given entity.
    pub fn is_walkable(&self, by: Entity) -> bool {
        match self {
            Self::Empty => true,
            Self::Wall => false,
            Self::Trail => true,
            Self::Actor(entity) if *entity == by => true,
            Self::Actor(_) => false, // don't walk over others
            Self::Zone(_) => true,
        }
    }

    /// Returns [`None`] if not walkable, otherwise the cost of walking to the
    /// tile.
    /// This is useful for pathfinding.
    /// The higher the cost, the less likely the character will want to walk
    /// over it.
    #[inline]
    pub fn walk_cost(&self, by: Entity) -> Option<TileWalkCost> {
        match self {
            Self::Wall => None,
            Self::Empty => Some(TileWalkCost::Normal),
            Self::Trail => Some(TileWalkCost::Preferred),
            Self::Actor(entity) if *entity == by => Some(TileWalkCost::Normal),
            Self::Actor(_) => None, // don't walk over others
            Self::Zone(_) => Some(TileWalkCost::Normal),
        }
    }

    /// Whether a tile represents a zone.
    /// A zone is a group of tiles that are connected to each other and entities
    /// enter and leave them.
    /// This is used to emit events about entering/leaving zones.
    #[inline]
    pub fn is_zone(&self) -> bool {
        matches!(self, Self::Zone(_))
    }

    /// Returns an iterator over all the zone tiles.
    /// This is used to automatically construct graph of zone relationships
    /// for pathfinding.
    #[inline]
    pub fn zones_iter() -> impl Iterator<Item = Self> {
        ZoneTileKind::iter().map(Self::Zone)
    }
}

impl TileMap {
    /// Whether the given square is inside the map.
    #[inline]
    pub fn contains(&self, square: Square) -> bool {
        let [min_x, max_x, min_y, max_y] = self.bounds;

        square.x >= min_x
            && square.x <= max_x
            && square.y >= min_y
            && square.y <= max_y
    }

    /// Get the kind of a tile.
    #[inline]
    pub fn get(&self, square: Square) -> Option<&[TileKind]> {
        if !self.contains(square) {
            return None;
        }

        self.squares.get(&square).map(|tiles| tiles.as_slice())
    }

    /// Whether the given square has the given kind of tile in any layer.
    #[inline]
    pub fn is_on(&self, square: Square, kind: impl Into<TileKind>) -> bool {
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
            self.contains(square)
        }
    }

    /// Whether the predicate matches any tile on the given square.
    /// Returns `false` if the square is out of bounds or has no tiles.
    #[inline]
    pub fn any_on(
        &self,
        square: Square,
        predicate: impl Fn(TileKind) -> bool,
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
        predicate: impl Fn(TileKind) -> bool,
    ) -> bool {
        self.squares
            .get(&square)
            .map(|tiles| tiles.iter().all(|tile| predicate(*tile)))
            .unwrap_or(false)
    }

    /// For given square, find the first [`TileKind::Empty`] tile or insert a
    /// new layer. Then return the index of the layer.
    ///
    /// Must not be called with [`TileKind::Empty`].
    #[inline]
    pub fn add_tile_to_first_empty_layer(
        &mut self,
        to: Square,
        tile: impl Into<TileKind>,
    ) -> Option<usize> {
        if !self.contains(to) {
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
                tiles.push(into_tile);
                tiles.len() - 1
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
        kind: impl Into<TileKind>,
    ) -> Option<TileKind> {
        if !self.contains(of) {
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

    /// If the mapping anon fn returns [`None`] then nothing happens.
    pub fn map_tile(
        &mut self,
        of: Square,
        layer: usize,
        map: impl FnOnce(TileKind) -> Option<TileKind>,
    ) -> Option<TileKind> {
        if !self.contains(of) {
            return None;
        }

        let tiles = self.squares.entry(of).or_default();

        if tiles.len() <= layer {
            tiles.resize(layer + 1, TileKind::Empty);
        }

        let tile = &mut tiles[layer]; // safe cuz we just resized
        let current = *tile;

        if let Some(new_kind) = map(current) {
            *tile = new_kind;
            Some(current)
        } else {
            None
        }
    }

    /// Map each tile on the given square to the given kind.
    #[inline]
    pub fn map_tiles(
        &mut self,
        of: Square,
        map: impl Fn(TileKind) -> TileKind,
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
        } else if self.contains(square) {
            Some(TileWalkCost::Normal)
        } else {
            None
        }
    }

    /// Access the map of squares to tiles.
    pub fn squares(&self) -> &HashMap<Square, SmallVec<[TileKind; 3]>> {
        &self.squares
    }
}

/// Pathfinding logic.
impl TileMap {
    /// No matter how many layers there are, all tile kinds within a single
    /// square must belong to the same zone group.
    #[inline]
    fn zone_group(&self, at: Square) -> Option<ZoneGroup> {
        self.squares.get(&at).and_then(|tiles| {
            tiles.iter().find_map(|tile| self.zones.group_of(tile))
        })
    }

    /// Path finding algorithm that returns partial path to the target.
    ///
    /// It's good when both squares are in some zone group as we can find
    /// minimum spanning tree between zones in the same group.
    pub fn find_partial_path(
        &self,
        who: Entity,
        from: Square,
        to: Square,
    ) -> Option<Vec<Square>> {
        if from == to {
            return Some(vec![]);
        }

        trace!("find_partial_path {from} -> {to}");

        // 3 possible situations:
        //
        // a) current and target are in the same zone group
        //    - let's find the first square that's strictly better than what we
        //      have now
        // b) they are in a different zone groups
        //    - search for first square that matches the target zone group to
        //      get us to the scenario a)
        // c) target has no zone group
        //    - without the target zone group information, current zone is
        //      irrelevant
        //    - brute force A* search bounded by a limit

        if let Some(to_zone_group) = self.zone_group(to) {
            // if from and to square are in the same zone group, that means we
            // can find minimum spanning tree between the zones
            let are_in_same_zone_group =
                self.zone_group(from).is_some_and(|from_zone_group| {
                    from_zone_group == to_zone_group
                });

            if are_in_same_zone_group {
                // a)

                // short-circuiting with "?" is alright bcs we already got a
                // positive result from `zone_group` for both squares

                let smallest_to_zone = self
                    .get(to)?
                    .iter()
                    .filter_map(|tile| Some((tile, self.zones.size_of(tile)?)))
                    .min_by_key(|(_, size)| *size)
                    .map(|(zone, _)| *zone)?;

                let any_from_zone = self
                    .get(from)?
                    .iter()
                    .find(|tile| tile.is_zone())
                    .copied()?;

                // It can happen that at runtime some path is blocked by
                // some object, which we could not foresee when we were building
                // the graph of zone relationships.
                // Hence, test multiple routes to find some that can be made
                // progress on.
                let test_max_alternative_routes = 3;
                for sequence_of_zones in self.find_sequences_of_zones_between(
                    any_from_zone,
                    smallest_to_zone,
                    test_max_alternative_routes,
                ) {
                    let from_tile_kinds = self.get(from)?;
                    let strictly_better_zones: Vec<_> = sequence_of_zones
                        .iter()
                        .filter(|to_tile| !from_tile_kinds.contains(to_tile))
                        .copied()
                        .collect();

                    if strictly_better_zones.is_empty() {
                        // we've already used all group info we could

                        return self.astar_and_stay_in_zone(
                            who,
                            from,
                            to,
                            smallest_to_zone,
                        );
                    } else if let Some(solution_to_better_zone) = self
                        .astar_into_strictly_better_zone(
                            who,
                            from,
                            to,
                            &sequence_of_zones,
                            &strictly_better_zones,
                        )
                    {
                        return Some(solution_to_better_zone);
                    }
                }

                None
            } else {
                // b)

                self.astar_into_zone_group(who, from, to, to_zone_group)
            }
        } else {
            // c)

            warn!("expensive partial_astar {from} -> {to}");
            self.partial_astar(who, from, to)
        }
    }

    /// The default success cond is max iterations or reaching the target.
    fn partial_astar(
        &self,
        who: Entity,
        from: Square,
        to: Square,
    ) -> Option<Vec<Square>> {
        /// Every time the search explores successors of a square, it increments
        /// an iteration counter.
        /// If the counter grows over this limit, the next square with better
        /// distance to the target than found so far is returned.
        const MAX_PARTIAL_ASTAR_EXPLORED_SQUARES: usize = 100;

        // see MAX_PARTIAL_ASTAR_EXPLORED_SQUARES
        let mut explored_squares = 0;
        // the best distance found so far with Manhattan distance
        let mut shortest_distance = i32::MAX;

        pathfinding::prelude::astar(
            &from,
            // successors
            |square: &Square| {
                square
                    .neighbors_no_diagonal()
                    .filter_map(|neighbor| {
                        self.walk_cost(neighbor, who)
                            .map(|cost| (neighbor, cost as i32))
                    })
                    .chain(square.neighbors_only_diagonal().filter_map(
                        // diagonal movement is costs more
                        |neighbor| {
                            self.walk_cost(neighbor, who)
                                .map(|cost| (neighbor, cost as i32 + 1))
                        },
                    ))
            },
            // heuristic
            |square: &Square| square.manhattan_distance(to),
            // success
            |square| {
                if explored_squares < MAX_PARTIAL_ASTAR_EXPLORED_SQUARES {
                    explored_squares += 1;
                    shortest_distance =
                        shortest_distance.min(square.manhattan_distance(to));

                    square == &to
                } else {
                    square.manhattan_distance(to) <= shortest_distance
                }
            },
        )
        .map(|(path, _)| path)
    }

    fn astar_and_stay_in_zone(
        &self,
        who: Entity,
        from: Square,
        to: Square,
        zone_to_stay_in: TileKind,
    ) -> Option<Vec<Square>> {
        debug_assert!(zone_to_stay_in.is_zone());

        pathfinding::prelude::astar(
            &from,
            // successors
            |square: &Square| {
                square
                    .neighbors_no_diagonal()
                    .filter_map(|neighbor| {
                        self.walk_cost(neighbor, who)
                            .map(|cost| (neighbor, cost as i32))
                    })
                    .chain(square.neighbors_only_diagonal().filter_map(
                        // diagonal movement is costs more
                        |neighbor| {
                            self.walk_cost(neighbor, who)
                                .map(|cost| (neighbor, cost as i32 + 1))
                        },
                    ))
                    .filter(|(neighbor, _)| {
                        self.is_on(*neighbor, zone_to_stay_in)
                    })
            },
            // heuristic
            |square: &Square| square.manhattan_distance(to),
            // success
            |square| square == &to,
        )
        .map(|(path, _)| path)
    }

    fn astar_into_strictly_better_zone(
        &self,
        who: Entity,
        from: Square,
        to: Square,
        allowed_zones: &[TileKind],
        strictly_better_zones: &[TileKind],
    ) -> Option<Vec<Square>> {
        pathfinding::prelude::astar(
            &from,
            // successors
            |square: &Square| {
                square
                    .neighbors_no_diagonal()
                    .filter_map(|neighbor| {
                        self.walk_cost(neighbor, who)
                            .map(|cost| (neighbor, cost as i32))
                    })
                    .chain(square.neighbors_only_diagonal().filter_map(
                        // diagonal movement is costs more
                        |neighbor| {
                            self.walk_cost(neighbor, who)
                                .map(|cost| (neighbor, cost as i32 + 1))
                        },
                    ))
                    .filter(|(neighbor, _)| {
                        self.any_on(*neighbor, |tile| {
                            allowed_zones.contains(&tile)
                        })
                    })
            },
            // heuristic
            |square: &Square| square.manhattan_distance(to),
            // success
            |square| {
                self.any_on(*square, |tile| {
                    strictly_better_zones.contains(&tile)
                })
            },
        )
        .map(|(path, _)| path)
    }

    fn astar_into_zone_group(
        &self,
        who: Entity,
        from: Square,
        to: Square,
        zone_group: ZoneGroup,
    ) -> Option<Vec<Square>> {
        pathfinding::prelude::astar(
            &from,
            // successors
            |square: &Square| {
                square.neighbors_with_diagonal().filter_map(|neighbor| {
                    self.walk_cost(neighbor, who)
                        .map(|cost| (neighbor, cost as i32))
                })
            },
            // heuristic
            |square: &Square| square.manhattan_distance(to),
            // success
            |square| self.zone_group(*square) == Some(zone_group),
        )
        .map(|(path, _)| path)
    }

    /// Given that the graphs are not going to be large (a handful of nodes),
    /// this should be very cheap compared to the actual pathfinding over many
    /// many squares.
    ///
    /// `k` is the number of different solutions to find (if they exist).
    /// It must be 1 or more.
    /// At least one solution is guaranteed to be found because there always
    /// must be a path between two zones in the same group.
    ///
    /// Note that even though the function returns an iterator, it's not lazy.
    fn find_sequences_of_zones_between(
        &self,
        from_zone: TileKind,
        to_zone: TileKind,
        k: usize,
    ) -> impl Iterator<Item = Vec<TileKind>> {
        debug_assert_ne!(0, k);
        // here I want to make sure that if two zones are neighbors and there's
        // a 3rd overlapping both, it should be included in the path
        // perhaps zone size is not the right metric to use

        pathfinding::prelude::yen(
            &from_zone,
            // successors
            |zone| {
                self.zones
                    .successors_of(zone)
                    .unwrap_or_default()
                    .iter()
                    .filter_map(move |s| {
                        Some((*s, self.zones.size_of(s)? as i32))
                    })
            },
            // success
            |zone| zone == &to_zone,
            k,
        )
        .into_iter()
        .map(|(path, _)| path)
    }
}

impl From<ZoneTileKind> for TileKind {
    fn from(l: ZoneTileKind) -> Self {
        Self::Zone(l)
    }
}

impl From<&ZoneTileKind> for TileKind {
    fn from(l: &ZoneTileKind) -> Self {
        Self::Zone(*l)
    }
}

impl TileKindMetas {
    /// Returns the zone group of the tile if it's a zone.
    /// This is useful for pathfinding.
    ///
    /// It must be the case that if two zones lie on the same square, they
    /// belong to the same group.
    ///
    /// Returns [`None`] if not present.
    fn group_of(&self, kind: &TileKind) -> Option<ZoneGroup> {
        self.inner.get(kind).map(|meta| meta.zone_group)
    }

    /// How many square does the zone comprise?
    ///
    /// Returns [`None`] if not present.
    fn size_of(&self, kind: &TileKind) -> Option<usize> {
        self.inner.get(kind).map(|meta| meta.zone_size)
    }

    /// Returns the zone successors of the tile if it's a zone.
    /// That is, what zones can be reached from this zone by either being
    /// subsets/supersets, neighbours or overlapping.
    ///
    /// Returns [`None`] if not present.
    fn successors_of(&self, kind: &TileKind) -> Option<&[TileKind]> {
        self.inner
            .get(kind)
            .map(|meta| meta.zone_successors.as_slice())
    }
}

impl Default for TileMap {
    fn default() -> Self {
        Self {
            bounds: default_bounds(),
            zones: TileKindMetas::default(),
            squares: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::utils::default;
    use bevy_grid_squared::sq;
    use smallvec::smallvec;

    use super::*;

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
        let mut tilemap = TileMap::default();

        // out of bounds returns none
        assert_eq!(tilemap.walk_cost(sq(-1001, 0), Entity::PLACEHOLDER), None);

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
        let mut tilemap = TileMap::default();
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
        let mut tilemap = TileMap::default();
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
        let mut tilemap = TileMap::default();

        assert_eq!(
            tilemap.add_tile_to_first_empty_layer(sq(-1001, 0), TileKind::Wall),
            None
        );

        assert_eq!(tilemap.set_tile_kind(sq(1001, 0), 0, TileKind::Wall), None);
    }

    /// Useful to track to prevent regressions.
    #[test]
    fn it_has_const_size_of_tilekind() {
        assert_eq!(std::mem::size_of::<TileKind>(), 16);

        let square: SmallVec<[TileKind; 3]> =
            smallvec![default(), default(), default(), default(), default()];
        assert_eq!(std::mem::size_of_val(&square), 56);
    }

    /// Test map such that:
    /// ```text
    ///  E<-D<-A=B<->C<->F->G
    ///
    ///  H=I<->J->K   and K<->I
    /// ```
    ///
    /// * `x=y` x and y are neighbors
    /// * `x<->y` x and y overlap
    /// * `x<-y` x is subset of y
    ///
    /// where
    /// A = Aisle1,
    /// B = Aisle2,
    /// C = Aisle3,
    /// D = Aisle4,
    /// E = Bed,
    /// F = Door,
    /// G = Elevator,
    /// H = Exit,
    /// I = Tea,
    /// J = Fridges,
    /// K = Hallway
    const DEV_MAP_TEST_RON: &str = r#"(
        zones: (inner: {
            Zone(Exit):(zone_group:(1),zone_size:4,zone_successors:[Zone(Tea)]),
            Zone(Aisle1):(zone_group:(0),zone_size:25,zone_successors:[Zone(Aisle2),Zone(Aisle4),Zone(Bed)]),
            Zone(Door):(zone_group:(0),zone_size:12,zone_successors:[Zone(Aisle3),Zone(Elevator)]),
            Zone(Tea):(zone_group:(1),zone_size:10,zone_successors:[Zone(Exit),Zone(Fridges),Zone(Hallway)]),
            Zone(Hallway):(zone_group:(1),zone_size:4,zone_successors:[Zone(Tea),Zone(Fridges)]),
            Zone(Aisle3):(zone_group:(0),zone_size:12,zone_successors:[Zone(Aisle2),Zone(Door)]),
            Zone(Elevator):(zone_group:(0),zone_size:1,zone_successors:[Zone(Door)]),
            Zone(Bed):(zone_group:(0),zone_size:1,zone_successors:[Zone(Aisle1),Zone(Aisle4)]),
            Zone(Aisle4):(zone_group:(0),zone_size:9,zone_successors:[Zone(Aisle1),Zone(Bed)]),
            Zone(Fridges):(zone_group:(1),zone_size:8,zone_successors:[Zone(Tea),Zone(Hallway)]),
            Zone(Aisle2):(zone_group:(0),zone_size:8,zone_successors:[Zone(Aisle1),Zone(Aisle3)])
        }),
        squares: {
        (x: -10, y: 16): [Zone(Fridges)],
        (x: -10, y: 17): [Zone(Fridges)],
        (x: -10, y: 19): [Zone(Exit)],
        (x: -10, y: 20): [Zone(Exit)],
        (x: -10, y: 23): [Zone(Aisle1)],
        (x: -10, y: 24): [Zone(Aisle1)],
        (x: -10, y: 25): [Zone(Aisle1)],
        (x: -10, y: 26): [Zone(Aisle1)],
        (x: -10, y: 27): [Zone(Aisle1)],
        (x: -9, y: 16): [Zone(Fridges), Zone(Hallway)],
        (x: -9, y: 17): [Zone(Fridges), Zone(Hallway)],
        (x: -9, y: 19): [Zone(Exit)],
        (x: -9, y: 20): [Zone(Exit)],
        (x: -9, y: 23): [Zone(Aisle1)],
        (x: -9, y: 24): [Zone(Aisle1), Zone(Aisle4)],
        (x: -9, y: 25): [Zone(Aisle1), Zone(Aisle4)],
        (x: -9, y: 26): [Zone(Aisle1), Zone(Aisle4)],
        (x: -9, y: 27): [Zone(Aisle1)],
        (x: -8, y: 16): [Zone(Tea), Zone(Fridges), Zone(Hallway)],
        (x: -8, y: 17): [Zone(Tea), Zone(Fridges), Zone(Hallway)],
        (x: -8, y: 18): [Zone(Tea)],
        (x: -8, y: 19): [Zone(Tea)],
        (x: -8, y: 20): [Zone(Tea)],
        (x: -8, y: 23): [Zone(Aisle1)],
        (x: -8, y: 24): [Zone(Aisle1), Zone(Aisle4)],
        (x: -8, y: 25): [Zone(Aisle1), Zone(Aisle4), Zone(Bed)],
        (x: -8, y: 26): [Zone(Aisle1), Zone(Aisle4)],
        (x: -8, y: 27): [Zone(Aisle1)],
        (x: -7, y: 16): [Zone(Tea), Zone(Fridges)],
        (x: -7, y: 17): [Zone(Tea), Zone(Fridges)],
        (x: -7, y: 18): [Zone(Tea)],
        (x: -7, y: 19): [Zone(Tea)],
        (x: -7, y: 20): [Zone(Tea)],
        (x: -7, y: 23): [Zone(Aisle1)],
        (x: -7, y: 24): [Zone(Aisle1), Zone(Aisle4)],
        (x: -7, y: 25): [Zone(Aisle1), Zone(Aisle4)],
        (x: -7, y: 26): [Zone(Aisle1), Zone(Aisle4)],
        (x: -7, y: 27): [Zone(Aisle1)],
        (x: -6, y: 23): [Zone(Aisle1)],
        (x: -6, y: 24): [Zone(Aisle1)],
        (x: -6, y: 25): [Zone(Aisle1)],
        (x: -6, y: 26): [Zone(Aisle1)],
        (x: -6, y: 27): [Zone(Aisle1)],
        (x: -5, y: 26): [Zone(Aisle2)],
        (x: -5, y: 27): [Zone(Aisle2)],
        (x: -4, y: 26): [Zone(Aisle2)],
        (x: -4, y: 27): [Zone(Aisle2)],
        (x: -3, y: 19): [Zone(Door)],
        (x: -3, y: 20): [Zone(Door)],
        (x: -3, y: 21): [Zone(Door)],
        (x: -3, y: 22): [Zone(Door)],
        (x: -3, y: 26): [Zone(Aisle2)],
        (x: -3, y: 27): [Zone(Aisle2)],
        (x: -2, y: 19): [Zone(Door)],
        (x: -2, y: 20): [Zone(Door), Zone(Elevator)],
        (x: -2, y: 21): [Zone(Door)],
        (x: -2, y: 22): [Zone(Door), Zone(Aisle3)],
        (x: -2, y: 23): [Zone(Aisle3)],
        (x: -2, y: 24): [Zone(Aisle3)],
        (x: -2, y: 25): [Zone(Aisle3)],
        (x: -2, y: 26): [Zone(Aisle2), Zone(Aisle3)],
        (x: -2, y: 27): [Zone(Aisle2), Zone(Aisle3)],
        (x: -1, y: 19): [Zone(Door)],
        (x: -1, y: 20): [Zone(Door)],
        (x: -1, y: 21): [Zone(Door)],
        (x: -1, y: 22): [Zone(Door), Zone(Aisle3)],
        (x: -1, y: 23): [Zone(Aisle3)],
        (x: -1, y: 24): [Zone(Aisle3)],
        (x: -1, y: 25): [Zone(Aisle3)],
        (x: -1, y: 26): [Zone(Aisle3)],
        (x: -1, y: 27): [Zone(Aisle3)],
    },)"#;

    #[test]
    fn it_finds_path_between_interesting_square_pairs() {
        #[derive(Default)]
        struct ExampleSetting {
            expected_partial_steps: Option<usize>,
        }

        let pairs = &[
            (
                sq(-7, 15),
                sq(-9, 24),
                "Takes a lot of steps during testing",
                ExampleSetting {
                    expected_partial_steps: Some(6),
                },
            ),
            (
                sq(-5, 15),
                sq(-10, 16),
                "Values at the edge of the map",
                default(),
            ),
            (
                sq(-8, 21),  // nowhere
                sq(-10, 16), // Fridges
                "Taking lots of steps during testing",
                ExampleSetting {
                    expected_partial_steps: Some(4),
                    ..default()
                },
            ),
            (sq(-2, 19), sq(-2, 20), "Going from F to (F, G)", default()),
            (sq(-10, 25), sq(-9, 25), "Going from A to (A, D)", default()),
            (
                sq(-9, 25),
                sq(-8, 25),
                "Going from (A, D) to (A, D, E)",
                default(),
            ),
            (
                sq(-9, 25),
                sq(-2, 20),
                "Going from (A, D) to (F, G)",
                ExampleSetting {
                    expected_partial_steps: Some(4),
                    ..default()
                },
            ),
            (
                sq(-2, 26),
                sq(-2, 22),
                "Going from (B, C) to (C, F)",
                default(),
            ),
            (
                sq(-10, 19),
                sq(-8, 17),
                "Going from H to (I, J, K)",
                default(),
            ),
            (sq(-10, 25), sq(-5, 26), "Going from A to B", default()),
        ];

        let tilemap: TileMap = ron::de::from_str(DEV_MAP_TEST_RON).unwrap();

        for (
            from,
            to,
            desc,
            ExampleSetting {
                expected_partial_steps,
            },
        ) in pairs
        {
            println!("{desc}: from {from} to {to}");

            let mut partial_from = *from;
            let mut jumps = vec![];
            while partial_from != *to {
                search_for_partial_path(
                    &tilemap,
                    1,
                    *from,
                    *to,
                    &mut partial_from,
                );

                jumps.push(partial_from);
            }

            if let Some(expected_partial_steps) = expected_partial_steps {
                assert_eq!(
                    *expected_partial_steps,
                    jumps.len(),
                    "Each partial step: {jumps:?}"
                );
            }
        }
    }

    #[test]
    fn it_finds_path_from_each_square_to_every_other() {
        let tilemap: TileMap = ron::de::from_str(DEV_MAP_TEST_RON).unwrap();

        let all_squares =
            || bevy_grid_squared::shapes::rectangle([-11, 0, 15, 28]);

        for to in all_squares() {
            println!("Finding path from all squares to {to}");

            for from in all_squares() {
                let mut partial_from = from;
                const MAX_PARTIAL_STEPS: usize = 7;

                let found = search_for_partial_path(
                    &tilemap,
                    MAX_PARTIAL_STEPS,
                    from,
                    to,
                    &mut partial_from,
                );

                assert!(
                    found,
                    "Finding path from {from} to {to} took more \
                    than {MAX_PARTIAL_STEPS} partial steps"
                );
            }
        }
    }

    fn search_for_partial_path(
        tilemap: &TileMap,
        max_partial_steps: usize,
        from: Square,
        to: Square,
        partial_from: &mut Square,
    ) -> bool {
        for _ in 0..max_partial_steps {
            let path = tilemap.find_partial_path(
                Entity::PLACEHOLDER,
                *partial_from,
                to,
            );
            assert!(
                path.is_some(),
                "from {from} (partial {partial_from}) to {to}"
            );
            let ends_up_on = path.unwrap().last().copied();
            if ends_up_on.is_none() {
                assert_eq!(to, *partial_from);
                return true;
            }

            *partial_from = ends_up_on.unwrap();
        }

        false
    }
}
