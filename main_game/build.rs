const MAPS: &[str] = &[(Apartment, "apartment/map.ron")];

fn main() {
    for (kind, path) in MAP_PATHS {
        xd(kind, path);
    }

    panic!("ok");
}

fn xd<T: IntoMap>(kind: T, path: &str) {
    let mut map_path = format!("main_game/assets/{MAP_PATH}");
    const MAX_DEPTH: usize = 5;
    for _ in 0..MAX_DEPTH {
        if std::path::Path::new(&map_path).exists() {
            break;
        }
        map_path = format!("../{map_path}");
    }

    println!("cargo:rerun-if-changed={map_path}");

    let map_bytes = std::fs::read(&map_path).expect("map RON file in assets");

    // TODO
}
