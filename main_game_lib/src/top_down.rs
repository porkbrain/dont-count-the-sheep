//! Deals with the most common view in the game: the top down.
//! It contains logic for layout of the scenes, and the actors in them.
//!
//! ## `layout`
//!
//! What kind of different tiles are there. There can be multiple tiles assigned
//! to a single square. Squares are coordinates and tiles dictate what happens
//! on a square. A tile is uniquely identified by `x` and `y` of the square and
//! a layer index.
//!
//! ## `actor`
//!
//! Moving around the pixel world, managing NPCs and the player character.

pub mod actor;
pub mod cameras;
pub mod environmental_objects;
pub mod inspect_and_interact;
pub mod layout;

use actor::{emit_movement_events, BeginDialogEvent};
pub use actor::{npc, player::Player, Actor, ActorMovementEvent, ActorTarget};
use bevy::prelude::*;
pub use inspect_and_interact::{InspectLabel, InspectLabelCategory};
pub use layout::{TileKind, TileMap, TopDownScene};
use leafwing_input_manager::plugin::InputManagerSystem;

use self::inspect_and_interact::ChangeHighlightedInspectLabelEvent;
use crate::{
    cutscene::in_cutscene,
    top_down::inspect_and_interact::ChangeHighlightedInspectLabelEventConsumer,
    StandardStateSemantics, WithStandardStateSemantics,
};

/// Does not add any systems, only registers generic-less types.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<npc::PlanPathEvent>()
            .add_event::<BeginDialogEvent>()
            .add_event::<ChangeHighlightedInspectLabelEvent>();

        #[cfg(feature = "devtools")]
        app.register_type::<Actor>()
            .register_type::<ActorTarget>()
            .register_type::<InspectLabel>()
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
/// - [`crate::top_down::actor::animate_movement`]
/// - [`crate::top_down::actor::emit_movement_events`]
/// - [`crate::top_down::actor::npc::drive_behavior`]
/// - [`crate::top_down::actor::npc::plan_path`]
/// - [`crate::top_down::actor::npc::run_path`]
/// - [`crate::top_down::actor::player::move_around`]
pub fn default_setup_for_scene<T>(app: &mut App)
where
    T: TopDownScene + WithStandardStateSemantics,
    T::LocalTileKind: layout::ZoneTile<Successors = T::LocalTileKind>,
{
    debug!("Adding assets for {}", T::type_path());

    let StandardStateSemantics {
        running,
        loading,
        quitting,
        ..
    } = T::semantics();

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
                actor::npc::plan_path::<T>
                    .run_if(on_event::<actor::npc::PlanPathEvent>()),
                actor::npc::run_path::<T>,
            )
                .chain()
                .run_if(in_state(running)),
        )
        .add_systems(OnExit(running), layout::systems::remove_resources::<T>);

    debug!("Adding inspect ability for {}", T::type_path());

    app.add_systems(
        Update,
        (
            inspect_and_interact::highlight_what_would_be_interacted_with,
            inspect_and_interact::change_highlighted_label
                .in_set(ChangeHighlightedInspectLabelEventConsumer)
                .run_if(on_event::<ChangeHighlightedInspectLabelEvent>()),
            inspect_and_interact::show_all_in_vicinity
                .run_if(common_action::inspect_pressed()),
        )
            .chain() // easier to reason about
            .run_if(in_state(running)),
    )
    .add_systems(
        Update,
        inspect_and_interact::schedule_hide_all
            .run_if(in_state(running))
            .run_if(common_action::inspect_just_released()),
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
                .run_if(on_event::<BeginDialogEvent>())
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
            .run_if(on_event::<ActorMovementEvent<T::LocalTileKind>>())
            .after(emit_movement_events::<T>),
    );

    debug!("Adding camera");

    app.add_systems(OnEnter(loading), common_visuals::camera::spawn)
        .add_systems(OnExit(quitting), common_visuals::camera::despawn)
        .add_systems(
            FixedUpdate,
            cameras::track_player_with_main_camera
                .after(actor::animate_movement::<T>)
                .run_if(in_state(running))
                .run_if(not(in_cutscene()))
                .run_if(not(
                    common_story::dialog::fe::portrait::in_portrait_dialog(),
                )),
        );

    debug!("Adding HUD");

    app.add_systems(
        OnEnter(running),
        (crate::hud::daybar::spawn, crate::hud::notification::spawn),
    )
    .add_systems(
        OnExit(running),
        (
            crate::hud::daybar::despawn,
            crate::hud::notification::despawn,
        ),
    )
    .add_systems(
        Update,
        (crate::hud::notification::update).run_if(in_state(running)),
    );
}

/// You can press `Enter` to export the map.
/// This will overwrite the RON file.
/// We draw an overlay with tiles that you can edit with left and right mouse
/// buttons.
#[cfg(feature = "devtools")]
pub fn dev_default_setup_for_scene<T>(app: &mut App)
where
    T: TopDownScene + WithStandardStateSemantics,
    T::LocalTileKind: Ord + bevy::reflect::GetTypeRegistration,
{
    use bevy_inspector_egui::quick::ResourceInspectorPlugin;
    use layout::map_maker::TileMapMakerToolbar as Toolbar;

    let StandardStateSemantics {
        running, quitting, ..
    } = T::semantics();

    app.register_type::<T::LocalTileKind>();

    // we insert the toolbar along with the map
    app.register_type::<Toolbar<T::LocalTileKind>>()
        .add_plugins(
            ResourceInspectorPlugin::<Toolbar<T::LocalTileKind>>::new()
                .run_if(resource_exists::<Toolbar<T::LocalTileKind>>),
        );

    app.add_systems(
        OnEnter(running),
        layout::map_maker::spawn_debug_grid_root::<T>,
    )
    .add_systems(
        Update,
        layout::map_maker::show_tiles_around_cursor::<T>
            .run_if(in_state(running)),
    )
    .add_systems(
        Update,
        (
            layout::map_maker::change_square_kind::<T>,
            layout::map_maker::recolor_squares::<T>,
            layout::map_maker::update_ui::<T>,
        )
            .run_if(in_state(running))
            .chain(),
    )
    .add_systems(OnExit(quitting), layout::map_maker::destroy_map::<T>);
}
