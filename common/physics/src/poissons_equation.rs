//! https://mattferraro.dev/posts/poissons-equation

pub(crate) mod systems;
pub(crate) mod types;

use bevy::app::App;

/// Add a new Poisson's equation system to the app.
pub fn register<T: Send + Sync + 'static>(app: &mut App) {
    app.add_event::<types::PoissonsEquationUpdateEvent<T>>()
        .add_systems(bevy::app::Last, systems::update::<T>);
}

#[cfg(feature = "poissons-eq-visualization")]
pub fn register_visualization<
    T: Send + Sync + 'static,
    W: types::WorldDimensions + 'static,
    P: From<bevy::prelude::Transform> + Into<types::GridCoords> + 'static,
>(
    app: &mut App,
) {
    app.add_systems(bevy::app::Startup, systems::spawn_visualization::<T, W>)
        .add_systems(bevy::app::Update, systems::update_visualization::<T, P>);
}
