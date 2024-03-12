//! Test with a basic example that I setup.
//! Contains nested nodes, metadata properties and spritesheets.

use rscn::Config;

const TSCN: &str = include_str!("basic.tscn");

#[test]
fn it_does_not_panic() {
    let state = rscn::parse(
        TSCN,
        Config {
            ysort: |v| v.extend(0.0),
            asset_path_prefix: "res://assets/",
        },
    );

    println!("{state:#?}");

    panic!("-------------------------------------");
}
