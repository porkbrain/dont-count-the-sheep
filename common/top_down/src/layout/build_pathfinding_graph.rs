//! When working with top down tile maps, we often end up with zone tiles that
//! identify some kind of area.
//! This can be a door, a room, etc.
//! Each map has its own special zones, and we want to know how they relate to
//! each other.
//!
//! We look for the following relationships:
//! - [`SupersetsOf`]
//! - [`SubsetsOf`]
//! - [`Overlaps`]
//! - [`Neighbors`]
//!
//! Build a [`LocalTileKindGraph`] from a tile map with the
//! [`LocalTileKindGraph::compute_from`].
//!
//! You can visualize how the graph with the [`LocalTileKindGraph::as_dotgraph`]
//! method.
//! The graph can be either converted to an SVG with the [`GraphExt::into_svg`]
//! or a [DOT][wiki-dot] string with the [`GraphExt::as_dot`] method.
//!
//! [wiki-dot]: https://en.wikipedia.org/wiki/DOT_(graph_description_language)

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    iter,
};

use bevy::{
    ecs::entity::Entity,
    utils::{
        default,
        hashbrown::{HashMap, HashSet},
    },
};
use graphviz_rust::{dot_generator::*, dot_structures::*};
use itertools::Itertools;

use crate::{layout::Tile, TileKind, TileMap, TopDownScene};

/// Map of tile kind variant `L` to those other variants (not including
/// itself - proper supersets) whose instances fully contain it (the key.)
/// If square contains the key variant, it contains also all the variants in the
/// value set.
///
/// `(tile, its supersets)`
pub type SupersetsOf<L> = HashMap<L, HashSet<L>>;

/// Map of tile kind variant `L` to those other variants (not including
/// itself - proper subsets) whose instances are fully contained by it (the
/// key.) If square contains any of the value set variants, it contains also the
/// key variant.
///
/// `(tile, its subsets)`
pub type SubsetsOf<L> = HashMap<L, HashSet<L>>;

/// Set of pairs of tile kind variants `L` that overlap in the same square and
/// are not supersets of each other.
///
/// `(tile, another)`
pub type Overlaps<L> = HashSet<(L, L)>;

/// Set of pairs of tile kind variants `L` that are walkable neighbors and are
/// not supersets of each other neither overlap in the same square.
///
/// `(tile, another)`
pub type Neighbors<L> = HashSet<(L, L)>;

/// Describes relationships between the local tile kind variants `L` in the
/// tile map.
/// That is, for a `T: TopDownScene` the `L` is `T::LocalTileKind`.
pub struct LocalTileKindGraph<L> {
    /// See the type alias `SupersetsOf`.
    pub supersets_of: SupersetsOf<L>,
    /// See the type alias `SubsetsOf`.
    pub subsets_of: SubsetsOf<L>,
    /// See the type alias `Overlaps`.
    pub overlaps: Overlaps<L>,
    /// See the type alias `Neighbors`.
    pub neighbors: Neighbors<L>,
}

/// Some useful methods for the [`Graph`] type.
pub trait GraphExt {
    /// Returns SVG bytes of the graph.
    fn into_svg(self) -> Result<Vec<u8>, std::io::Error>;

    /// Returns DOT string of the graph.
    fn as_dot(&self) -> String;
}

/// Series of steps to compute the relationships between the local tile kind
/// variants `L` in the tile map.
#[derive(Default)]
enum GraphComputeStep<L> {
    /// First find for each tile kind variant `L` all tiles that contain
    /// every single instance of it.
    /// This computes the [`SupersetsOf<L>`].
    #[default]
    Supersets,
    /// Then from the previous result construct the inverse of it, which is
    /// for each tile kind variant `L` all tiles that are contained in it.
    /// This computes the [`SubsetsOf<L>`].
    Subsets { from_supersets: SupersetsOf<L> },
    /// The find which tiles overlap in the same square and are not supersets
    /// of each other.
    /// This computes the [`Overlaps<L>`].
    Overlaps {
        from_supersets: SupersetsOf<L>,
        from_subsets: SubsetsOf<L>,
    },
    /// Finally check which non overlapping tiles are walkable neighbors and
    /// are not supersets of each other.
    /// This computes the [`Neighbors<L>`].
    Neighbors {
        from_supersets: SupersetsOf<L>,
        from_subsets: SubsetsOf<L>,
        from_overlaps: Overlaps<L>,
    },
    /// All computation for the graph finished, ready with the result.
    Done(LocalTileKindGraph<L>),
}

/// Whether the computation is done or not.
enum GraphComputeResult<L> {
    NextStep(GraphComputeStep<L>),
    Done(LocalTileKindGraph<L>),
}

impl GraphExt for Graph {
    fn into_svg(self) -> Result<Vec<u8>, std::io::Error> {
        use graphviz_rust::cmd::Format;
        graphviz_rust::exec(
            self,
            &mut graphviz_rust::printer::PrinterContext::default(),
            vec![Format::Svg.into()],
        )
    }

    fn as_dot(&self) -> String {
        use graphviz_rust::printer::DotPrinter;
        self.print(&mut graphviz_rust::printer::PrinterContext::default())
    }
}

impl<L: Tile> LocalTileKindGraph<L>
where
    L: Ord,
{
    /// Find all relationships between the local tile kind variants `L` in the
    /// tile map `T`.
    pub fn compute_from<T: TopDownScene<LocalTileKind = L>>(
        tilemap: &TileMap<T>,
    ) -> Self {
        let mut compute_step = GraphComputeStep::default();
        loop {
            match compute_step.next_step(tilemap) {
                GraphComputeResult::NextStep(next_step) => {
                    compute_step = next_step;
                }
                GraphComputeResult::Done(graph) => {
                    return graph;
                }
            }
        }
    }

    /// Group zones into zones that are connected to each other.
    /// This means that if zone A overlaps, neighbors is a subset or superset
    /// of zone B, they both belong to the same group.
    /// Ie. if there's a set of edges that lead from zone A to zone B, they are
    /// in the same group.
    ///
    /// This function returns an implementation of [`L::zone_group_autogen`].
    /// The output is something like
    ///
    /// ```example
    /// impl crate::my::layout::MyTileKind {
    ///     fn zone_group_autogen(&self) -> Option<common_top_down::layout::ZoneGroup> {
    ///         use common_top_down::layout::ZoneGroup;
    ///         match self {
    ///             Self::SomeZone => Some(ZoneGroup(0)),
    ///             Self::AnotherZone => Some(ZoneGroup(0)),
    ///             Self::YetAnotherZone => Some(ZoneGroup(1)),
    ///             #[allow(unreachable_patterns)]
    ///             _ => None,
    ///         }
    ///     }
    /// }
    /// ```
    pub fn generate_zone_groups_rs(&self) -> String {
        // first merge all zones into their relevant groups

        // the index is going to be the zone group unique value in the end
        let mut zone_groups: Vec<HashSet<L>> = default();

        for zone in L::zones_iter() {
            let group = if let Some((index, _)) = zone_groups
                .iter()
                .find_position(|group| group.contains(&zone))
            {
                index
            } else {
                zone_groups.push(default());
                zone_groups.len() - 1
            };

            let all_related = iter::once(zone)
                .chain(
                    self.overlaps
                        .iter()
                        .chain(self.neighbors.iter())
                        .copied()
                        .filter_map(|(a, b)| {
                            if a == zone {
                                Some(b)
                            } else if b == zone {
                                Some(a)
                            } else {
                                None
                            }
                        }),
                )
                .chain(
                    self.supersets_of
                        .get(&zone)
                        .map(|supersets| {
                            supersets.iter().copied().collect_vec()
                        })
                        .unwrap_or_default()
                        .into_iter(),
                )
                .chain(
                    self.subsets_of
                        .get(&zone)
                        .map(|subsets| subsets.iter().copied().collect_vec())
                        .unwrap_or_default()
                        .into_iter(),
                );

            for related in all_related {
                zone_groups[group].insert(related);
            }
        }

        // now generate the Rust impl

        let autogen = stringify!(
            /// This is an autogenerated implementation by the map maker tool.

            #[rustfmt::skip]
            impl %L% {
                #[inline]
                fn zone_group_autogen(&self) -> Option<common_top_down::layout::ZoneGroup> {
                    use common_top_down::layout::ZoneGroup;
                    match self {
                        %ZONE_GROUPS%
                        #[allow(unreachable_patterns)]
                        _ => None,
                    }
                }
            }
        );

        let mut zone_groups_str = String::new();
        for zone in L::zones_iter() {
            // the index is the group
            let group = zone_groups
                .iter()
                .position(|group| group.contains(&zone))
                .unwrap();

            zone_groups_str.push_str(&format!(
                "Self::{zone:?} => Some(ZoneGroup({group})),\n",
            ));
        }

        autogen
            .replace(
                "%L%",
                &iter::once("crate")
                    .chain(L::type_path().split("::").skip(1))
                    .join("::"),
            )
            .replace("%ZONE_GROUPS%", &zone_groups_str)
    }

    /// Returns a [`Graph`] representation of the relationships between the
    /// local tile kind variants `L` in the tile map.
    ///
    /// The ID of the graph will be `graph_{name}`.
    ///
    /// We order everything to make the graph deterministic.
    pub fn as_dotgraph(
        &self,
        name: impl Display,
    ) -> graphviz_rust::dot_structures::Graph {
        let mut g = graph!(di id!(format!("graph_{name}")));
        // some breathing room
        g.add_stmt(attr!("nodesep", 0.5).into());
        g.add_stmt(attr!("ranksep", 1.0).into());

        // map tile kinds to nodes
        let nodes: BTreeMap<L, _> = L::zones_iter()
            .filter(|kind| kind.is_zone())
            .map(|kind| (kind, node!({ format!("{kind:?}").to_lowercase() })))
            .collect();
        // add nodes straight away - some might not be in any relationship, and
        // we want them in the graph
        for (_, node) in &nodes {
            g.add_stmt(node.clone().into());
        }

        // tiles that are supersets of others but have no supersets themselves
        // are called top level, and have their own subgraph
        let mut top_level_subgraphs: BTreeMap<_, _> = self
            .subsets_of
            .iter()
            .filter_map(|(superset, _)| {
                let own_supersets = self.supersets_of.get(superset);

                if own_supersets.is_none() {
                    Some((
                        *superset,
                        subgraph!(id!(
                            format!("cluster_{superset:?}").to_lowercase()
                        )),
                    ))
                } else {
                    None
                }
            })
            .collect();

        // this is all single directional edges, going from subset -> superset
        let subsets_of: BTreeMap<_, _> = self.subsets_of.iter().collect();
        for (superset, subsets) in subsets_of {
            let own_supersets = self.supersets_of.get(superset);

            // which subgraph do we belong to?
            let subgraph =
                if let Some(s) = top_level_subgraphs.get_mut(superset) {
                    s
                } else if own_supersets.is_none() {
                    unreachable!(
                        "{superset:?} cannot have subsets ({subsets:?}), \
                        no supersets but also not a top level subgraph"
                    );
                } else {
                    let top_level_superset = own_supersets
                        .unwrap() // safe cuz else if ^
                        .iter()
                        .find(|own_superset| {
                            top_level_subgraphs.contains_key(*own_superset)
                        })
                        .unwrap();

                    // safe cuz contains_key ^
                    top_level_subgraphs.get_mut(top_level_superset).unwrap()
                };

            let superset_node = nodes.get(superset).unwrap();

            let subsets: BTreeSet<_> = subsets.iter().collect();
            for subset in subsets {
                // determines whether there exists ANOTHER subset of the
                // superset that is also a superset of THIS subset
                // if that's the case `is_broadest_subset` will be false
                let is_broadest_subset = {
                    let subset_supersets =
                        self.supersets_of.get(subset).unwrap();
                    let is_the_only_superset = subset_supersets.len() == 1
                        && subset_supersets.contains(superset);

                    is_the_only_superset
                        || own_supersets
                            .map(|own_supersets| {
                                subset_supersets
                                    .difference(own_supersets)
                                    .count()
                            })
                            // if there's only one superset of the subset
                            // that's not in superset's own supersets, then
                            // it's the broadest subset
                            // (the one subset's superset being the superset
                            // itself, ie collecting the difference would yield
                            // the superset)
                            .is_some_and(|difference| difference == 1)
                };

                if !is_broadest_subset {
                    // for the tested subset, there exists another subset of the
                    // superset that is also a superset of the tested subset
                    continue;
                }

                let subset_node = nodes.get(subset).unwrap();

                subgraph.stmts.push(
                    Edge {
                        ty: EdgeTy::Pair(
                            Vertex::N(superset_node.id.clone()),
                            Vertex::N(subset_node.id.clone()),
                        ),
                        attributes: vec![],
                    }
                    .into(),
                );
            }
        }
        // finally add all these nodes
        for (_, subgraph) in top_level_subgraphs {
            g.add_stmt(subgraph.into());
        }

        // bidirectional relationships
        let overlaps: BTreeSet<_> = self.overlaps.iter().collect();
        for (a, b) in overlaps {
            let a = nodes.get(a).unwrap();
            let b = nodes.get(b).unwrap();

            g.add_stmt(
                Edge {
                    ty: EdgeTy::Pair(
                        Vertex::N(a.id.clone()),
                        Vertex::N(b.id.clone()),
                    ),
                    attributes: vec![attr!("dir", "both")],
                }
                .into(),
            );
        }

        // also bidirectional relationships
        let neighbors: BTreeSet<_> = self.neighbors.iter().collect();
        for (a, b) in neighbors {
            let a = nodes.get(a).unwrap();
            let b = nodes.get(b).unwrap();

            g.add_stmt(
                Edge {
                    ty: EdgeTy::Pair(
                        Vertex::N(a.id.clone()),
                        Vertex::N(b.id.clone()),
                    ),
                    attributes: vec![
                        attr!("arrowhead", "tee"),
                        attr!("arrowtail", "tee"),
                        attr!("dir", "both"),
                    ],
                }
                .into(),
            );
        }

        g
    }
}

impl<L: std::fmt::Debug> std::fmt::Debug for LocalTileKindGraph<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "LocalTileKindGraph")?;
        for (a, b) in &self.supersets_of {
            writeln!(f, "{a:?} is subset of {b:?}")?;
        }
        for (a, b) in &self.subsets_of {
            writeln!(f, "{a:?} is superset of {b:?}")?;
        }
        for (a, b) in &self.overlaps {
            writeln!(f, "{a:?} overlaps with {b:?}")?;
        }
        for (a, b) in &self.neighbors {
            writeln!(f, "{a:?} neighbors with {b:?}")?;
        }
        writeln!(f)
    }
}

impl<L: Tile + Ord> GraphComputeStep<L> {
    fn next_step<T: TopDownScene<LocalTileKind = L>>(
        self,
        map: &TileMap<T>,
    ) -> GraphComputeResult<L> {
        let next_step = match self {
            Self::Supersets => Self::Subsets {
                from_supersets: find_supersets(map),
            },
            Self::Subsets { from_supersets } => Self::Overlaps {
                from_subsets: find_subsets(&from_supersets),
                from_supersets,
            },
            Self::Overlaps {
                from_supersets,
                from_subsets,
            } => Self::Neighbors {
                from_overlaps: find_overlaps(
                    map,
                    &from_supersets,
                    &from_subsets,
                ),
                from_subsets,
                from_supersets,
            },
            Self::Neighbors {
                from_overlaps,
                from_subsets,
                from_supersets,
            } => Self::Done(LocalTileKindGraph {
                neighbors: find_neighbors(
                    map,
                    &from_supersets,
                    &from_subsets,
                    &from_overlaps,
                ),
                supersets_of: from_supersets,
                subsets_of: from_subsets,
                overlaps: from_overlaps,
            }),
            Self::Done(graph) => return GraphComputeResult::Done(graph),
        };

        GraphComputeResult::NextStep(next_step)
    }
}

/// Find which tiles are supersets of which.
fn find_supersets<T: TopDownScene>(
    map: &TileMap<T>,
) -> SupersetsOf<T::LocalTileKind> {
    let mut supersets_of: SupersetsOf<_> = default();
    for tiles in map.squares().values() {
        let locals: HashSet<_> = get_local_zones(tiles).collect();

        for local in locals.iter().copied() {
            let local_supersets =
                supersets_of.entry(local).or_insert_with(|| {
                    T::LocalTileKind::zones_iter()
                        .filter(|superset| {
                            superset != &local && superset.is_zone()
                        })
                        .collect()
                });

            local_supersets.retain(|another| locals.contains(another));
        }
    }
    supersets_of.retain(|_, supersets| !supersets.is_empty());

    supersets_of
}

/// Find which tiles are subsets of which.
fn find_subsets<L: Tile>(supersets_of: &SupersetsOf<L>) -> SubsetsOf<L> {
    let mut subsets: SubsetsOf<L> = default();

    for (superset, subset) in
        supersets_of.iter().flat_map(|(subset, supersets)| {
            supersets.iter().map(move |superset| (*superset, *subset))
        })
    {
        subsets
            .entry(superset)
            .or_insert_with(HashSet::new)
            .insert(subset);
    }

    subsets
}

/// Find which tiles overlap in the same square and are not supersets
/// of each other
fn find_overlaps<T: TopDownScene>(
    map: &TileMap<T>,
    supersets_of: &SupersetsOf<T::LocalTileKind>,
    subsets_of: &SubsetsOf<T::LocalTileKind>,
) -> Overlaps<T::LocalTileKind>
where
    T::LocalTileKind: Ord,
{
    let mut overlaps: Overlaps<T::LocalTileKind> = default();
    for tiles in map.squares().values() {
        let locals = get_local_zones(tiles).collect_vec();

        for local in locals.clone() {
            let local_supersets = supersets_of.get(&local);
            let local_subsets = subsets_of.get(&local);

            for another in locals.clone() {
                if local == another
                    || local_supersets.is_some_and(|s| s.contains(&another))
                    || local_subsets.is_some_and(|s| s.contains(&another))
                {
                    continue;
                }

                let pair = (local.min(another), another.max(local));
                overlaps.insert(pair);
            }
        }
    }

    overlaps
}

/// Check which non overlapping tiles are walkable neighbors but are not
/// supersets of each other
fn find_neighbors<T: TopDownScene>(
    map: &TileMap<T>,
    supersets_of: &SupersetsOf<T::LocalTileKind>,
    subsets_of: &SubsetsOf<T::LocalTileKind>,
    overlaps: &Overlaps<T::LocalTileKind>,
) -> Neighbors<T::LocalTileKind>
where
    T::LocalTileKind: Ord,
{
    let mut neighbors: Neighbors<T::LocalTileKind> = default();

    for (sq, tiles) in map.squares().iter() {
        let locals = get_local_zones(tiles).collect_vec();

        for neighbor_sq in sq.neighbors_with_diagonal() {
            let Some(neighbor_locals) = map.squares().get(&neighbor_sq) else {
                continue;
            };

            if !map.is_walkable(neighbor_sq, Entity::PLACEHOLDER) {
                continue;
            }

            let neighbor_locals =
                get_local_zones(neighbor_locals).collect_vec();

            for local in locals.clone() {
                let local_supersets = supersets_of.get(&local);
                let local_subsets = subsets_of.get(&local);

                for another in neighbor_locals.clone() {
                    if local == another
                        || local_supersets.is_some_and(|s| s.contains(&another))
                        || local_subsets.is_some_and(|s| s.contains(&another))
                    {
                        continue;
                    }

                    let pair = (local.min(another), another.max(local));
                    if !overlaps.contains(&pair) {
                        neighbors.insert(pair);
                    }
                }
            }
        }
    }

    neighbors
}

fn get_local_zones<L: Tile>(
    tiles: &[TileKind<L>],
) -> impl Iterator<Item = L> + '_ {
    tiles
        .iter()
        .filter(|tile| tile.is_zone())
        .filter_map(|tile| tile.into_local())
}
