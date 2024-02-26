#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(trivial_bounds)]
#![feature(let_chains)]
#![allow(clippy::type_complexity)]

pub mod actor;
pub mod cameras;
pub mod inspect_ability;
pub mod interactable;
pub mod layout;

pub use actor::{npc, player::Player, Actor, ActorMovementEvent, ActorTarget};
use bevy::{ecs::event::event_update_condition, prelude::*};
pub use inspect_ability::{InspectLabel, InspectLabelCategory};
pub use layout::{TileKind, TileMap, TopDownScene};

/// Does not add any systems, only registers generic-less types.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<npc::PlanPathEvent>()
            .register_type::<Actor>()
            .register_type::<ActorTarget>()
            .register_type::<InspectLabelCategory>()
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
/// - [`common_story::despawn_camera`]
/// - [`common_story::portrait_dialog::advance`]
/// - [`common_story::portrait_dialog::change_selection`]
/// - [`common_story::spawn_camera`]
/// - [`crate::actor::animate_movement`]
/// - [`crate::actor::emit_movement_events`]
/// - [`crate::actor::npc::drive_behavior`]
/// - [`crate::actor::npc::plan_path`]
/// - [`crate::actor::npc::run_path`]
/// - [`crate::actor::player::move_around`]
/// - [`common_visuals::systems::advance_atlas_animation`]
/// - [`common_visuals::systems::interpolate`]
pub fn default_setup_for_scene<T: TopDownScene, S: States + Copy>(
    app: &mut App,
    loading: S,
    running: S,
    quitting: S,
) where
    T::LocalTileKind: layout::ZoneTile<Successors = T::LocalTileKind>,
{
    debug!("Adding assets for {}", T::type_path());

    app.add_systems(
        OnEnter(loading),
        common_assets::store::insert_as_resource::<common_story::StoryAssets>,
    )
    .add_systems(
        OnExit(quitting),
        common_assets::store::remove_as_resource::<common_story::StoryAssets>,
    );

    debug!("Adding map layout for {}", T::type_path());

    app.add_event::<ActorMovementEvent<T::LocalTileKind>>()
        .init_asset_loader::<common_assets::RonLoader<TileMap<T>>>()
        .init_asset::<TileMap<T>>()
        .register_type::<TileKind<T::LocalTileKind>>()
        .register_type::<TileMap<T>>()
        .register_type::<ActorMovementEvent<T::LocalTileKind>>();

    app.add_systems(
        OnEnter(loading),
        layout::systems::start_loading_map::<T>,
    )
    .add_systems(
        First,
        layout::systems::try_insert_map_as_resource::<T>
            .run_if(in_state(loading)),
    )
    .add_systems(
        FixedUpdate,
        actor::animate_movement::<T>.run_if(in_state(running)),
    )
    .add_systems(
        Update,
        actor::emit_movement_events::<T>
            .run_if(in_state(running))
            // so that we can emit this event on current frame
            .after(actor::player::move_around::<T>),
    )
    .add_systems(
        Update,
        actor::player::move_around::<T>
            .run_if(in_state(running))
            .run_if(common_action::move_action_pressed())
            .run_if(common_story::portrait_dialog::not_in_portrait_dialog()),
    )
    .add_systems(
        Update,
        (
            actor::npc::drive_behavior,
            actor::npc::plan_path::<T>
                .run_if(event_update_condition::<actor::npc::PlanPathEvent>),
            actor::npc::run_path::<T>,
        )
            .chain()
            .run_if(in_state(running)),
    )
    .add_systems(
        OnExit(running),
        layout::systems::remove_resources::<T>,
    );

    debug!("Adding visuals for {}", T::type_path());

    app.add_systems(
        FixedUpdate,
        (
            common_visuals::systems::advance_atlas_animation,
            common_visuals::systems::interpolate,
        )
            .run_if(in_state(running)),
    );

    debug!("Adding story for {}", T::type_path());

    app.add_systems(OnEnter(loading), common_story::spawn_camera)
        .add_systems(
            Update,
            common_story::portrait_dialog::change_selection
                .run_if(in_state(running))
                .run_if(common_story::portrait_dialog::in_portrait_dialog())
                .run_if(common_action::move_action_just_pressed()),
        )
        .add_systems(
            Last,
            common_story::portrait_dialog::advance
                .run_if(in_state(running))
                .run_if(common_story::portrait_dialog::in_portrait_dialog())
                .run_if(common_action::interaction_just_pressed()),
        )
        .add_systems(OnEnter(quitting), common_story::despawn_camera);

    debug!("Adding inspect ability for {}", T::type_path());

    app.add_event::<T::LocalActionEvent>()
        .register_type::<InspectLabel>()
        .add_systems(
            Update,
            inspect_ability::show_all_in_vicinity
                .run_if(in_state(running))
                .run_if(common_action::inspect_pressed()),
        )
        .add_systems(
            Update,
            inspect_ability::schedule_hide_all
                .run_if(in_state(running))
                .run_if(common_action::inspect_just_released()),
        )
        .add_systems(
            Update,
            inspect_ability::cancel_hide_all
                .run_if(in_state(running))
                .run_if(common_action::inspect_just_pressed()),
        );
}

/// You can press `Enter` to export the map.
/// This will overwrite the RON file.
/// We draw an overlay with tiles that you can edit with left and right mouse
/// buttons.
#[cfg(feature = "dev")]
pub fn dev_default_setup_for_scene<T: TopDownScene, S: States>(
    app: &mut App,
    running: S,
    quitting: S,
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
    )
    .add_systems(
        Update,
        (
            layout::map_maker::change_square_kind::<T>,
            layout::map_maker::recolor_squares::<T>,
        )
            .run_if(in_state(running.clone()))
            .chain(),
    )
    .add_systems(
        Update,
        layout::map_maker::export_map::<T>
            .run_if(input_just_pressed(KeyCode::Enter))
            .run_if(in_state(running.clone())),
    )
    .add_systems(
        OnExit(quitting.clone()),
        layout::map_maker::destroy_map::<T>,
    );
}
