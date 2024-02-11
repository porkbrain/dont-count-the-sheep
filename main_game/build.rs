use std::fs;

use common_top_down::{
    layout::build_pathfinding_graph::{self, GraphExt},
    IntoMap,
};
use scene_apartment::Apartment;

fn main() {
    build_tile_graph::<Apartment>();
}

/// Build a graph of the tilemap and store it in the assets folder.
/// Also, updates the docs folder with a .svg file of the graph.
///
/// Does this only if the tilemap file has changed or doesn't exist.
fn build_tile_graph<T: IntoMap>()
where
    T::LocalTileKind: Ord,
{
    let map_path = go_back_in_dir_tree_until_path_found(format!(
        "main_game/assets/{}",
        T::asset_path()
    ));

    println!("cargo:rerun-if-changed={map_path}");

    // TODO: find the graph if assets folder. if exists, compare hash in the
    // file header with the hash of the tilemap file. if it's the same, skip

    let tilemap_bytes = fs::read(&map_path).expect("map RON file in assets");

    let g = build_pathfinding_graph::LocalTileKindGraph::compute_from::<T>(
        &tilemap_bytes,
    );

    // TODO: store the graph in the assets folder as a .ron file and include
    // hash in the file header.

    let dot_g = g.as_dotgraph(T::name());
    let svg = dot_g.into_svg().unwrap();

    let scene_path =
        go_back_in_dir_tree_until_path_found(format!("scenes/{}", T::name()));
    fs::write(format!("{scene_path}/docs/tile-graph.svg"), svg).unwrap();
}

fn go_back_in_dir_tree_until_path_found(mut path: String) -> String {
    const MAX_DEPTH: usize = 5;
    for _ in 0..MAX_DEPTH {
        if std::path::Path::new(&path).exists() {
            return path;
        }
        path = format!("../{path}");
    }

    panic!("Could not find path to {}", path);
}
