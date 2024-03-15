//! Test with a basic example that I setup.
//! Contains nested nodes, metadata properties and spritesheets.

use bevy::utils::default;

const TSCN: &str = include_str!("basic.tscn");

#[test]
fn it_does_not_panic() {
    let state = common_rscn::parse(TSCN, &default());

    assert_eq!(4, state.root.children.len());
    for child_name in
        ["Cupboard", "HallwayBg", "PlayerApartmentBg", "Elevator"].iter()
    {
        assert!(state
            .root
            .children
            .get(*child_name)
            .unwrap()
            .in_2d
            .is_some());
    }

    assert_eq!(
        -49.5,
        state
            .root
            .children
            .get("Elevator")
            .as_ref()
            .unwrap()
            .in_2d
            .as_ref()
            .unwrap()
            .position
            .y
    );
}
