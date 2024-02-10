#!/bin/bash

# TODO: store output artefacts in a gitignored directory
# TODO: assets are not being loaded
# TODO: create a feature that disables dynlinking of bevy

CARGO_PROFILE_RELEASE_DEBUG=true \
    RUSTFLAGS='-C force-frame-pointers=y' \
    cargo flamegraph
