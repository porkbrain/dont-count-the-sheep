//! Test with a basic example that I setup.
//! Contains nested nodes, metadata properties and spritesheets.

const TSCN: &str = include_str!("basic.tscn");

#[test]
fn it_does_not_panic() {
    rscn::parse(TSCN);

    panic!("-------------------------------------");
}
