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
//! Build a [`ZoneTileKindGraph`] from a tile map with the
//! [`ZoneTileKindGraph::compute_from`].
//!
//! You can visualize how the graph with the [`ZoneTileKindGraph::as_dotgraph`]
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

use super::{TileKindMeta, ZoneGroup};
use crate::top_down::{TileKind, TileMap};

/// Map of tile kind variant to those other variants (not including
/// itself - proper supersets) whose instances fully contain it (the key.)
/// If square contains the key variant, it contains also all the variants in the
/// value set.
///
/// `(tile, its supersets)`
pub(crate) type SupersetsOf = HashMap<TileKind, HashSet<TileKind>>;

/// Map of tile kind variant to those other variants (not including
/// itself - proper subsets) whose instances are fully contained by it (the
/// key.) If square contains any of the value set variants, it contains also the
/// key variant.
///
/// `(tile, its subsets)`
pub(crate) type SubsetsOf = HashMap<TileKind, HashSet<TileKind>>;

/// Set of pairs of tile kind variants that overlap in the same square and
/// are not supersets of each other.
///
/// `(tile, another)`
pub(crate) type Overlaps = HashSet<(TileKind, TileKind)>;

/// Set of pairs of tile kind variants that are walkable neighbors and are
/// not supersets of each other neither overlap in the same square.
///
/// `(tile, another)`
pub(crate) type Neighbors = HashSet<(TileKind, TileKind)>;

/// Describes relationships between the tile kind variants in the tile map.
pub(crate) struct ZoneTileKindGraph {
    /// See the type alias `SupersetsOf`.
    pub(crate) supersets_of: SupersetsOf,
    /// See the type alias `SubsetsOf`.
    pub(crate) subsets_of: SubsetsOf,
    /// See the type alias `Overlaps`.
    pub(crate) overlaps: Overlaps,
    /// See the type alias `Neighbors`.
    pub(crate) neighbors: Neighbors,
    /// How many squares each zone comprises.
    pub(crate) zone_sizes: HashMap<TileKind, usize>,
}

/// Some useful methods for the [`Graph`] type.
pub(crate) trait GraphExt {
    /// Returns SVG bytes of the graph.
    fn into_svg(self) -> Result<Vec<u8>, std::io::Error>;

    /// Returns DOT string of the graph.
    fn as_dot(&self) -> String;
}

/// Series of steps to compute the relationships between the zone tile kind
/// variants in the tile map.
#[derive(Default)]
enum GraphComputeStep {
    /// First find for each tile kind variant all tiles that contain
    /// every single instance of it.
    /// This computes the [`SupersetsOf`].
    #[default]
    Supersets,
    /// Then from the previous result construct the inverse of it, which is
    /// for each tile kind variant all tiles that are contained in it.
    /// This computes the [`SubsetsOf`].
    Subsets { from_supersets: SupersetsOf },
    /// The find which tiles overlap in the same square and are not supersets
    /// of each other.
    /// This computes the [`Overlaps`].
    Overlaps {
        from_supersets: SupersetsOf,
        from_subsets: SubsetsOf,
    },
    /// Check which non overlapping tiles are walkable neighbors and
    /// are not supersets of each other.
    /// This computes the [`Neighbors`].
    Neighbors {
        from_supersets: SupersetsOf,
        from_subsets: SubsetsOf,
        from_overlaps: Overlaps,
    },
    /// Calculate how many squares each zone comprises.
    Sizes {
        from_supersets: SupersetsOf,
        from_subsets: SubsetsOf,
        from_overlaps: Overlaps,
        from_neighbors: Neighbors,
    },
    /// All computation for the graph finished, ready with the result.
    Done(ZoneTileKindGraph),
}

/// Whether the computation is done or not.
enum GraphComputeResult {
    NextStep(GraphComputeStep),
    Done(ZoneTileKindGraph),
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

impl ZoneTileKindGraph {
    /// Find all relationships between the zone tile kind variants in the
    /// tile map.
    pub(crate) fn compute_from(tilemap: &TileMap) -> Self {
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

    pub(crate) fn calculate_zone_tile_metadata(
        &self,
    ) -> BTreeMap<TileKind, TileKindMeta> {
        let mut metas: BTreeMap<TileKind, TileKindMeta> = default();

        // Group zones into zones that are connected to each other.
        // This means that if zone A overlaps, neighbors is a subset or superset
        // of zone B, they both belong to the same group.
        // Ie. if there's a set of edges that lead from zone A to zone B, they
        // are in the same group.

        // the index is going to be the zone group unique value in the end
        let mut zone_groups: Vec<BTreeSet<TileKind>> = default();
        let mut successors: HashMap<TileKind, Vec<TileKind>> = default();

        for zone in TileKind::zones_iter() {
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

                if related != zone {
                    successors.entry(zone).or_default().push(related);
                }
            }
        }

        // store zone group, size and successors if the zone is present in the
        // map
        for (zone, size) in &self.zone_sizes {
            if *size == 0 {
                continue;
            }

            if let Some(zone_group) =
                zone_groups.iter().position(|group| group.contains(zone))
            {
                let mut successors =
                    successors.get(zone).cloned().unwrap_or_default();
                successors.sort();

                let entry = metas.entry(*zone).or_default();
                entry.zone_size = *size;
                entry.zone_successors = successors.into_iter().collect();
                entry.zone_group = ZoneGroup(zone_group);
            }
        }

        metas
    }

    /// Returns a [`Graph`] representation of the relationships between the
    /// zone tile kind variants in the tile map.
    ///
    /// The ID of the graph will be `graph_{name}`.
    ///
    /// We order everything to make the graph deterministic.
    pub(crate) fn as_dotgraph(
        &self,
        name: impl Display,
    ) -> graphviz_rust::dot_structures::Graph {
        let mut g = graph!(di id!(format!("graph_{name}")));
        // some breathing room
        g.add_stmt(attr!("nodesep", 0.5).into());
        g.add_stmt(attr!("ranksep", 1.0).into());

        fn dotgraph_node_name(kind: TileKind) -> String {
            match kind {
                TileKind::Actor(_)
                | TileKind::Trail
                | TileKind::Empty
                | TileKind::Wall => unreachable!("Tile {kind:?} is not a zone"),
                TileKind::Zone(zone) => zone.to_string().to_lowercase(),
            }
        }

        // map tile kinds to nodes
        let nodes: BTreeMap<TileKind, _> = TileKind::zones_iter()
            .filter(|kind| {
                // we only care about zones that are present in this map
                self.zone_sizes.get(kind).copied().unwrap_or_default() > 0
            })
            .map(|kind| (kind, node!({ dotgraph_node_name(kind) })))
            .collect();
        // add nodes straight away - some might not be in any relationship, and
        // we want them in the graph
        for node in nodes.values() {
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
                        subgraph!(id!(format!(
                            "cluster_{}",
                            dotgraph_node_name(*superset)
                        ))),
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

impl std::fmt::Debug for ZoneTileKindGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ZoneTileKindGraph")?;
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

impl GraphComputeStep {
    fn next_step(self, map: &TileMap) -> GraphComputeResult {
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
            } => Self::Sizes {
                from_neighbors: find_neighbors(
                    map,
                    &from_supersets,
                    &from_subsets,
                    &from_overlaps,
                ),
                from_supersets,
                from_subsets,
                from_overlaps,
            },
            Self::Sizes {
                from_supersets,
                from_subsets,
                from_overlaps,
                from_neighbors,
            } => Self::Done(ZoneTileKindGraph {
                neighbors: from_neighbors,
                supersets_of: from_supersets,
                subsets_of: from_subsets,
                overlaps: from_overlaps,
                zone_sizes: count_zone_sizes(map),
            }),
            Self::Done(graph) => return GraphComputeResult::Done(graph),
        };

        GraphComputeResult::NextStep(next_step)
    }
}

/// Find which tiles are supersets of which.
fn find_supersets(map: &TileMap) -> SupersetsOf {
    let mut supersets_of: SupersetsOf = default();
    for tiles in map.squares().values() {
        let zones: HashSet<_> = get_zones(tiles).collect();

        for zone in zones.iter().copied() {
            let zone_supersets =
                supersets_of.entry(zone).or_insert_with(|| {
                    TileKind::zones_iter()
                        .filter(|superset| {
                            superset != &zone && superset.is_zone()
                        })
                        .collect()
                });

            zone_supersets.retain(|another| zones.contains(another));
        }
    }
    supersets_of.retain(|_, supersets| !supersets.is_empty());

    supersets_of
}

/// Find which tiles are subsets of which.
fn find_subsets(supersets_of: &SupersetsOf) -> SubsetsOf {
    let mut subsets: SubsetsOf = default();

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
fn find_overlaps(
    map: &TileMap,
    supersets_of: &SupersetsOf,
    subsets_of: &SubsetsOf,
) -> Overlaps {
    let mut overlaps: Overlaps = default();
    for tiles in map.squares().values() {
        let zones = get_zones(tiles).collect_vec();

        for zone in zones.clone() {
            let zone_supersets = supersets_of.get(&zone);
            let zone_subsets = subsets_of.get(&zone);

            for another in zones.clone() {
                if zone == another
                    || zone_supersets.is_some_and(|s| s.contains(&another))
                    || zone_subsets.is_some_and(|s| s.contains(&another))
                {
                    continue;
                }

                let pair = (zone.min(another), another.max(zone));
                overlaps.insert(pair);
            }
        }
    }

    overlaps
}

/// Check which non overlapping tiles are walkable neighbors but are not
/// supersets of each other
fn find_neighbors(
    map: &TileMap,
    supersets_of: &SupersetsOf,
    subsets_of: &SubsetsOf,
    overlaps: &Overlaps,
) -> Neighbors {
    let mut neighbors: Neighbors = default();

    for (sq, tiles) in map.squares().iter() {
        let zones = get_zones(tiles).collect_vec();

        for neighbor_sq in sq.neighbors_with_diagonal() {
            let Some(neighbor_tiles) = map.squares().get(&neighbor_sq) else {
                continue;
            };

            if !map.is_walkable(neighbor_sq, Entity::PLACEHOLDER) {
                continue;
            }

            let neighbor_zones = get_zones(neighbor_tiles).collect_vec();

            for zone in zones.clone() {
                let zone_supersets = supersets_of.get(&zone);
                let zone_subsets = subsets_of.get(&zone);

                for another in neighbor_zones.clone() {
                    if zone == another
                        || zone_supersets.is_some_and(|s| s.contains(&another))
                        || zone_subsets.is_some_and(|s| s.contains(&another))
                    {
                        continue;
                    }

                    let pair = (zone.min(another), another.max(zone));
                    if !overlaps.contains(&pair) {
                        neighbors.insert(pair);
                    }
                }
            }
        }
    }

    neighbors
}

fn count_zone_sizes(map: &TileMap) -> HashMap<TileKind, usize> {
    map.squares()
        .values()
        .flat_map(|tiles| get_zones(tiles))
        .fold(HashMap::new(), |mut acc, zone| {
            *acc.entry(zone).or_insert(0) += 1;
            acc
        })
}

fn get_zones(tiles: &[TileKind]) -> impl Iterator<Item = TileKind> + '_ {
    tiles.iter().filter(|tile| tile.is_zone()).map(|tile| *tile)
}
