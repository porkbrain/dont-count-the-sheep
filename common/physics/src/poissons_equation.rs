//! <https://mattferraro.dev/posts/poissons-equation>

pub(crate) mod systems;
pub(crate) mod types;

use bevy::prelude::*;

/// Add a new Poisson's equation system to the app.
/// You still need to insert the Poisson's equation `T` resource.
pub fn register<T: Send + Sync + 'static, S: States>(app: &mut App, state: S) {
    app.add_event::<types::PoissonsEquationUpdateEvent<T>>()
        .add_systems(
            bevy::app::Last,
            systems::update::<T>.run_if(in_state(state)),
        );
}

#[cfg(feature = "poissons-eq-visualization")]
pub fn register_visualization<
    T: Send + Sync + 'static,
    W: types::WorldDimensions + 'static,
    P: From<bevy::prelude::Transform> + Into<types::GridCoords> + 'static,
    S: States,
>(
    app: &mut App,
    state: S,
) {
    app.add_systems(
        bevy::app::Startup,
        systems::spawn_visualization::<T, W>.run_if(in_state(state.clone())),
    )
    .add_systems(
        bevy::app::Update,
        systems::update_visualization::<T, P>.run_if(in_state(state)),
    );
}
