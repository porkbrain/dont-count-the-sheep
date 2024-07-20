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
pub mod scene_configs;

use actor::{emit_movement_events, BeginDialogEvent};
pub use actor::{npc, player::Player, Actor, ActorMovementEvent, ActorTarget};
use bevy::prelude::*;
pub use inspect_and_interact::{InspectLabel, InspectLabelCategory};
pub use layout::{TileKind, TileMap};
use leafwing_input_manager::plugin::InputManagerSystem;

use self::inspect_and_interact::ChangeHighlightedInspectLabelEvent;
use crate::{
    cutscene::in_cutscene, in_top_down_loading_state,
    in_top_down_running_state,
    top_down::inspect_and_interact::ChangeHighlightedInspectLabelEventConsumer,
    InTopDownScene,
};

/// Does not add any systems, only registers generic-less types.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<npc::PlanPathEvent>()
            .add_event::<BeginDialogEvent>()
            .add_event::<ChangeHighlightedInspectLabelEvent>()
            .add_event::<ActorMovementEvent>();

        app.add_plugins(environmental_objects::Plugin);

        //
        // Assets
        //

        app.init_asset_loader::<common_assets::ron_loader::Loader<TileMap>>()
            .init_asset::<TileMap>();

        app.add_systems(
                OnEnter(InTopDownScene::loading()),
                common_assets::store::insert_as_resource::<common_story::StoryAssets>,
            )
            .add_systems(
                OnExit(InTopDownScene::leaving()),
                common_assets::store::remove_as_resource::<common_story::StoryAssets>,
            );

        //
        // TileMap
        //

        app.add_systems(
            OnExit(InTopDownScene::running()),
            layout::systems::remove_resources,
        )
        .add_systems(
            OnEnter(InTopDownScene::loading()),
            layout::systems::start_loading_map,
        )
        .add_systems(
            First,
            layout::systems::try_insert_map_as_resource
                .run_if(in_top_down_loading_state()),
        )
        .add_systems(
            FixedUpdate,
            actor::animate_movement.run_if(in_top_down_running_state()),
        )
        .add_systems(
            Update,
            actor::emit_movement_events
                .run_if(in_top_down_running_state())
                // so that we can emit this event on current frame
                .after(actor::player::move_around),
        )
        .add_systems(
            Update,
            actor::player::move_around
                .run_if(in_top_down_running_state())
                .run_if(common_action::move_action_pressed())
                .run_if(not(crate::dialog::fe::portrait::in_portrait_dialog())),
        )
        .add_systems(
            Update,
            (
                actor::npc::drive_behavior,
                actor::npc::plan_path
                    .run_if(on_event::<actor::npc::PlanPathEvent>()),
                actor::npc::run_path,
            )
                .chain()
                .run_if(in_top_down_running_state()),
        );

        //
        // Camera
        //

        app.add_systems(
            OnEnter(InTopDownScene::loading()),
            common_visuals::camera::spawn,
        )
        .add_systems(
            OnExit(InTopDownScene::leaving()),
            common_visuals::camera::despawn,
        )
        .add_systems(
            FixedUpdate,
            cameras::track_player_with_main_camera
                .after(actor::animate_movement)
                .run_if(in_top_down_running_state())
                .run_if(not(in_cutscene()))
                .run_if(not(crate::dialog::fe::portrait::in_portrait_dialog())),
        );

        //
        // Inspect and interact systems
        //

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
                .run_if(in_top_down_running_state()),
        )
        .add_systems(
            Update,
            inspect_and_interact::schedule_hide_all
                .run_if(in_top_down_running_state())
                .run_if(common_action::inspect_just_released()),
        );
        app.add_systems(
            PreUpdate,
            inspect_and_interact::interact
                .run_if(in_top_down_running_state())
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
                .run_if(not(crate::dialog::fe::portrait::in_portrait_dialog()))
                .after(InputManagerSystem::Update),
        )
        .add_systems(
            Update,
            (
                actor::npc::mark_nearby_as_ready_for_interaction,
                actor::npc::begin_dialog
                    .run_if(on_event::<BeginDialogEvent>())
                    .run_if(not(
                        crate::dialog::fe::portrait::in_portrait_dialog(),
                    )),
            )
                .run_if(in_top_down_running_state()),
        )
        .add_systems(
            Update,
            inspect_and_interact::match_interact_label_with_action_event
                .run_if(in_top_down_running_state())
                .run_if(on_event::<ActorMovementEvent>())
                .after(emit_movement_events),
        );

        //
        // HUD
        //

        app.add_systems(
            OnEnter(InTopDownScene::running()),
            (crate::hud::daybar::spawn, crate::hud::notification::spawn),
        )
        .add_systems(
            OnExit(InTopDownScene::running()),
            (
                crate::hud::daybar::despawn,
                crate::hud::notification::despawn,
            ),
        )
        .add_systems(
            Update,
            (crate::hud::notification::update)
                .run_if(in_top_down_running_state()),
        );

        #[cfg(feature = "devtools")]
        {
            use bevy_inspector_egui::quick::ResourceInspectorPlugin;
            use layout::map_maker::TileMapMakerToolbar as Toolbar;

            app.register_type::<Toolbar>()
                .register_type::<Actor>()
                .register_type::<ActorTarget>()
                .register_type::<TileKind>()
                .register_type::<ActorMovementEvent>()
                .register_type::<InspectLabel>()
                .register_type::<InspectLabelCategory>()
                .register_type::<npc::NpcInTheMap>()
                .register_type::<npc::PlanPathEvent>()
                .register_type::<TileMap>()
                .register_type::<npc::BehaviorLeaf>()
                .register_type::<npc::BehaviorPaused>();

            app.add_plugins(
                ResourceInspectorPlugin::<Toolbar>::new()
                    .run_if(resource_exists::<Toolbar>),
            );

            // You can press `Enter` to export the map.
            // This will overwrite the RON file.
            // We draw an overlay with tiles that you can edit with left and
            // right mouse buttons.
            app.add_systems(
                OnEnter(InTopDownScene::running()),
                layout::map_maker::spawn_debug_grid_root,
            )
            .add_systems(
                Update,
                layout::map_maker::show_tiles_around_cursor
                    .run_if(in_top_down_running_state()),
            )
            .add_systems(
                Update,
                (
                    layout::map_maker::change_square_kind,
                    layout::map_maker::recolor_squares,
                    layout::map_maker::update_ui,
                )
                    .run_if(in_top_down_running_state())
                    .chain(),
            )
            .add_systems(
                OnExit(InTopDownScene::leaving()),
                layout::map_maker::destroy_map,
            );
        }
    }
}
