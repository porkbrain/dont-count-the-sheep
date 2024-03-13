//! Test with a basic example that I setup.
//! Contains nested nodes, metadata properties and spritesheets.

use common_rscn::{Config, NodeName};

const TSCN: &str = include_str!("basic.tscn");

#[test]
fn it_does_not_panic() {
    let state = common_rscn::parse(
        TSCN,
        Config {
            asset_path_prefix: "res://assets/",
        },
    );

    assert_eq!(4, state.root.children.len());
    for child_name in
        ["Cupboard", "HallwayBg", "PlayerApartmentBg", "Elevator"].iter()
    {
        assert!(state
            .root
            .children
            .get(&NodeName(child_name.to_string()))
            .unwrap()
            .in_2d
            .is_some());
    }
}
