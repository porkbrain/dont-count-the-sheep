#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(trivial_bounds)]
#![feature(let_chains)]
#![allow(clippy::type_complexity)]

pub mod actor;
pub mod cameras;
pub mod environmental_objects;
pub mod inspect_and_interact;
pub mod layout;

pub use actor::{npc, player::Player, Actor, ActorMovementEvent, ActorTarget};
use bevy::{ecs::event::event_update_condition, prelude::*};
pub use inspect_and_interact::{InspectLabel, InspectLabelCategory};
pub use layout::{TileKind, TileMap, TopDownScene};
use leafwing_input_manager::plugin::InputManagerSystem;

use crate::actor::{emit_movement_events, BeginDialogEvent};

/// Does not add any systems, only registers generic-less types.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<npc::PlanPathEvent>()
            .add_event::<BeginDialogEvent>();

        #[cfg(feature = "devtools")]
        app.register_type::<Actor>()
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
/// - [`crate::actor::animate_movement`]
/// - [`crate::actor::emit_movement_events`]
/// - [`crate::actor::npc::drive_behavior`]
/// - [`crate::actor::npc::plan_path`]
/// - [`crate::actor::npc::run_path`]
/// - [`crate::actor::player::move_around`]
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
        .init_asset_loader::<common_assets::ron_loader::Loader<TileMap<T>>>()
        .init_asset::<TileMap<T>>()
        .register_type::<TileKind<T::LocalTileKind>>()
        .register_type::<TileMap<T>>()
        .register_type::<ActorMovementEvent<T::LocalTileKind>>();

    app.add_systems(OnEnter(loading), layout::systems::start_loading_map::<T>)
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
                .run_if(not(
                    common_story::dialog::fe::portrait::in_portrait_dialog(),
                )),
        )
        .add_systems(
            Update,
            (
                actor::npc::drive_behavior,
                actor::npc::plan_path::<T>.run_if(
                    event_update_condition::<actor::npc::PlanPathEvent>,
                ),
                actor::npc::run_path::<T>,
            )
                .chain()
                .run_if(in_state(running)),
        )
        .add_systems(OnExit(running), layout::systems::remove_resources::<T>);

    debug!("Adding inspect ability for {}", T::type_path());

    app.register_type::<InspectLabel>()
        .add_systems(
            Update,
            inspect_and_interact::show_all_in_vicinity
                .run_if(in_state(running))
                .run_if(common_action::inspect_pressed()),
        )
        .add_systems(
            Update,
            inspect_and_interact::schedule_hide_all
                .run_if(in_state(running))
                .run_if(common_action::inspect_just_released()),
        )
        .add_systems(
            Update,
            inspect_and_interact::cancel_hide_all
                .run_if(in_state(running))
                .run_if(common_action::inspect_just_pressed()),
        );

    debug!("Adding interaction systems for {}", T::type_path());

    app.add_systems(
        PreUpdate,
        inspect_and_interact::interact
            .run_if(in_state(running))
            .run_if(common_action::interaction_just_pressed())
            // Without this condition, the dialog will start when the player
            // exists the previous one because:
            // 1. The interact system runs, interact is just pressed, and so
            //    emits the event.
            // 2. Player finishes the dialog by pressing interaction. This
            //    consumes the interact action.
            // 3. Consuming the action did fuck all because the event was
            //    already emitted earlier. Since the commands to remove the
            //    dialog resource were applied, the condition to not run the
            //    begin_dialog system will not prevent rerun
            .run_if(not(
                common_story::dialog::fe::portrait::in_portrait_dialog(),
            ))
            .after(InputManagerSystem::Update),
    )
    .add_systems(
        Update,
        (
            actor::npc::mark_nearby_as_ready_for_interaction,
            actor::npc::begin_dialog
                .run_if(event_update_condition::<BeginDialogEvent>)
                .run_if(not(
                    common_story::dialog::fe::portrait::in_portrait_dialog(),
                )),
        )
            .run_if(in_state(running)),
    )
    .add_systems(
        Update,
        inspect_and_interact::match_interact_label_with_action_event::<T>
            .run_if(in_state(running))
            .run_if(
                event_update_condition::<ActorMovementEvent<T::LocalTileKind>>,
            )
            .after(emit_movement_events::<T>),
    );
}

/// You can press `Enter` to export the map.
/// This will overwrite the RON file.
/// We draw an overlay with tiles that you can edit with left and right mouse
/// buttons.
#[cfg(feature = "devtools")]
pub fn dev_default_setup_for_scene<T: TopDownScene, S: States>(
    app: &mut App,
    running: S,
    quitting: S,
) where
    T::LocalTileKind: Ord + bevy::reflect::GetTypeRegistration,
{
    use bevy_inspector_egui::quick::ResourceInspectorPlugin;
    use layout::map_maker::TileMapMakerToolbar as Toolbar;

    app.register_type::<T::LocalTileKind>();

    // we insert the toolbar along with the map
    app.register_type::<Toolbar<T::LocalTileKind>>()
        .add_plugins(
            ResourceInspectorPlugin::<Toolbar<T::LocalTileKind>>::new()
                .run_if(resource_exists::<Toolbar<T::LocalTileKind>>),
        );

    app.add_systems(
        OnEnter(running.clone()),
        layout::map_maker::visualize_map::<T>,
    )
    .add_systems(
        Update,
        (
            layout::map_maker::change_square_kind::<T>,
            layout::map_maker::recolor_squares::<T>,
            layout::map_maker::update_ui::<T>,
        )
            .run_if(in_state(running.clone()))
            .chain(),
    )
    .add_systems(
        OnExit(quitting.clone()),
        layout::map_maker::destroy_map::<T>,
    );
}
