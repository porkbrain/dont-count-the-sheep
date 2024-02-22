#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(trivial_bounds)]
#![feature(let_chains)]
#![allow(clippy::type_complexity)]

pub mod actor;
pub mod cameras;
pub mod interactable;
pub mod layout;

pub use actor::{npc, player::Player, Actor, ActorMovementEvent, ActorTarget};
use bevy::prelude::*;
pub use layout::{TileKind, TileMap, TopDownScene};

/// Does not add any systems, only registers generic-less types.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Actor>().register_type::<ActorTarget>();

        app.add_event::<npc::PlanPathEvent>()
            .register_type::<npc::NpcInTheMap>()
            .register_type::<npc::PlanPathEvent>()
            .register_type::<npc::BehaviorLeaf>()
            .register_type::<npc::BehaviorPaused>();
    }
}

/// Registers unique `T` types, asset loader for the map RON file, and systems
/// including from other packages:
/// - [`common_assets::store::insert_as_resource`]
/// - [`common_assets::store::remove_as_resource`]
/// - [`actor::emit_movement_events`]
/// - [`common_visuals::systems::advance_atlas_animation`]
/// - [`common_visuals::systems::interpolate`]
/// - [`common_story::spawn_camera`]
/// - [`common_story::portrait_dialog::change_selection`]
/// - [`common_story::portrait_dialog::advance`]
/// - [`common_story::despawn_camera`]
pub fn default_setup_for_scene<T: TopDownScene, S: States>(
    app: &mut App,
    loading: S,
    running: S,
    quitting: S,
) {
    debug!("Adding assets for {}", T::type_path());

    app.add_systems(
        OnEnter(loading.clone()),
        common_assets::store::insert_as_resource::<common_story::DialogAssets>,
    )
    .add_systems(
        OnExit(quitting.clone()),
        common_assets::store::remove_as_resource::<common_story::DialogAssets>,
    );

    debug!("Adding map layout for {}", T::type_path());

    app.add_event::<ActorMovementEvent<T::LocalTileKind>>()
        .init_asset_loader::<common_assets::RonLoader<TileMap<T>>>()
        .init_asset::<TileMap<T>>()
        .register_type::<TileKind<T::LocalTileKind>>()
        .register_type::<TileMap<T>>()
        .register_type::<ActorMovementEvent<T::LocalTileKind>>();

    app.add_systems(
        OnEnter(loading.clone()),
        layout::systems::start_loading_map::<T>,
    )
    .add_systems(
        First,
        layout::systems::try_insert_map_as_resource::<T>
            .run_if(in_state(loading.clone())),
    )
    .add_systems(
        Update,
        actor::emit_movement_events::<T>
            .run_if(in_state(running.clone()))
            // so that we can emit this event on current frame
            .after(actor::player::move_around::<T>),
    )
    .add_systems(
        OnExit(running.clone()),
        layout::systems::remove_resources::<T>,
    );

    debug!("Adding visuals for {}", T::type_path());

    app.add_systems(
        FixedUpdate,
        (
            common_visuals::systems::advance_atlas_animation,
            common_visuals::systems::interpolate,
        )
            .run_if(in_state(running.clone())),
    );

    debug!("Adding story for {}", T::type_path());

    app.add_systems(OnEnter(loading.clone()), common_story::spawn_camera);
    app.add_systems(
        Update,
        common_story::portrait_dialog::change_selection
            .run_if(in_state(running.clone()))
            .run_if(common_story::portrait_dialog::in_portrait_dialog())
            .run_if(common_action::move_action_just_pressed()),
    );
    app.add_systems(
        Last,
        common_story::portrait_dialog::advance
            .run_if(in_state(running.clone()))
            .run_if(common_story::portrait_dialog::in_portrait_dialog())
            .run_if(common_action::interaction_just_pressed()),
    );
    app.add_systems(OnEnter(quitting.clone()), common_story::despawn_camera);
}

#[cfg(feature = "dev")]
/// You can press `Enter` to export the map.
/// This will overwrite the RON file.
/// We draw an overlay with tiles that you can edit with left and right mouse
/// buttons.
///
/// The `Ord` bound is required for the map maker export.
/// If needed, this function can be reorganized to avoid the bound in
/// production.
pub fn dev_default_setup_for_scene<T: TopDownScene, S: States>(
    app: &mut App,
    running: S,
) where
    T::LocalTileKind: Ord,
{
    use bevy::input::common_conditions::input_just_pressed;
    use bevy_inspector_egui::quick::ResourceInspectorPlugin;

    // we insert the toolbar along with the map
    app.register_type::<layout::map_maker::TileMapMakerToolbar<T::LocalTileKind>>()
        .add_plugins(ResourceInspectorPlugin::<
            layout::map_maker::TileMapMakerToolbar<T::LocalTileKind>,
        >::default());

    app.add_systems(
        OnEnter(running.clone()),
        layout::map_maker::visualize_map::<T>,
    );
    app.add_systems(
        Update,
        (
            layout::map_maker::change_square_kind::<T>,
            layout::map_maker::recolor_squares::<T>,
        )
            .run_if(in_state(running.clone()))
            .chain(),
    );
    app.add_systems(
        Update,
        layout::map_maker::export_map::<T>
            .run_if(input_just_pressed(KeyCode::Enter))
            .run_if(in_state(running)),
    );
}
