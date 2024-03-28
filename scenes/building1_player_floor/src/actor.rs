//! Player and NPCs.

use bevy::ecs::event::event_update_condition;
use common_loading_screen::LoadingScreenSettings;
use common_story::dialog::DialogGraph;
use common_visuals::camera::MainCamera;
use main_game_lib::{
    common_ext::QueryExt,
    cutscene::{self, in_cutscene},
};
use top_down::{
    actor::{emit_movement_events, movement_event_emitted},
    ActorMovementEvent, TileKind,
};

use crate::{
    layout::{Elevator, MeditatingHint, SleepingHint},
    prelude::*,
};

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (start_meditation_minigame_if_near_chair, enter_the_elevator)
                .run_if(event_update_condition::<Building1PlayerFloorAction>)
                .run_if(Building1PlayerFloor::in_running_state())
                .run_if(not(in_cutscene())),
        );

        app.add_systems(
            Update,
            toggle_zone_hints
                .run_if(movement_event_emitted::<Building1PlayerFloor>())
                .run_if(Building1PlayerFloor::in_running_state())
                .after(emit_movement_events::<Building1PlayerFloor>),
        );
    }
}

/// Will change the game state to meditation minigame.
fn start_meditation_minigame_if_near_chair(
    mut cmd: Commands,
    mut action_events: EventReader<Building1PlayerFloorAction>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,

    player: Query<Entity, With<Player>>,
) {
    let is_triggered = action_events.read().any(|action| {
        matches!(action, Building1PlayerFloorAction::StartMeditation)
    });

    if is_triggered && let Some(entity) = player.get_single_or_none() {
        cmd.entity(entity).despawn_recursive();

        cmd.insert_resource(LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::Space),
            stare_at_loading_screen_for_at_least: Some(
                WHEN_ENTERING_MEDITATION_SHOW_LOADING_IMAGE_FOR_AT_LEAST,
            ),
            ..default()
        });

        *transition =
            GlobalGameStateTransition::Building1PlayerFloorToMeditation;
        next_state.set(Building1PlayerFloor::quitting());
    }
}

/// By entering the elevator, the player can this scene.
fn enter_the_elevator(
    mut cmd: Commands,
    mut action_events: EventReader<Building1PlayerFloorAction>,
    mut assets: ResMut<Assets<DialogGraph>>,

    player: Query<Entity, With<Player>>,
    elevator: Query<Entity, With<Elevator>>,
    camera: Query<Entity, With<MainCamera>>,
    points: Query<(&Name, &rscn::Point)>,
) {
    let is_triggered = action_events.read().any(|action| {
        matches!(action, Building1PlayerFloorAction::EnterElevator)
    });

    if is_triggered && let Some(player) = player.get_single_or_none() {
        let point_in_elevator = {
            let (_, rscn::Point(pos)) = points
                .iter()
                .find(|(name, _)| **name == Name::new("InElevator"))
                .expect("InElevator point not found");

            *pos
        };

        cutscene::enter_an_elevator::spawn(
            &mut cmd,
            &mut assets,
            player,
            elevator.single(),
            camera.single(),
            point_in_elevator,
            &[
                (
                    GlobalGameStateTransition::Building1PlayerFloorToDowntown,
                    "go to downtown",
                ),
                (
                    GlobalGameStateTransition::Building1PlayerFloorToBuilding1Basement1,
                    "go to basement",
                ),
            ],
        );
    }
}

/// Shows hint for bed or for meditating when player is in the zone to actually
/// interact with those objects.
fn toggle_zone_hints(
    mut events: EventReader<
        ActorMovementEvent<
            <Building1PlayerFloor as TopDownScene>::LocalTileKind,
        >,
    >,

    mut sleeping: Query<
        &mut Visibility,
        (With<SleepingHint>, Without<MeditatingHint>),
    >,
    mut meditating: Query<
        &mut Visibility,
        (With<MeditatingHint>, Without<SleepingHint>),
    >,
) {
    for event in events.read().filter(|event| event.is_player()) {
        match event {
            ActorMovementEvent::ZoneEntered { zone, .. } => match *zone {
                TileKind::Local(
                    Building1PlayerFloorTileKind::MeditationZone,
                ) => {
                    *meditating.single_mut() = Visibility::Visible;
                }
                TileKind::Local(Building1PlayerFloorTileKind::BedZone) => {
                    *sleeping.single_mut() = Visibility::Visible;
                }
                _ => {}
            },
            ActorMovementEvent::ZoneLeft { zone, .. } => match *zone {
                TileKind::Local(
                    Building1PlayerFloorTileKind::MeditationZone,
                ) => {
                    *meditating.single_mut() = Visibility::Hidden;
                }
                TileKind::Local(Building1PlayerFloorTileKind::BedZone) => {
                    *sleeping.single_mut() = Visibility::Hidden;
                }
                _ => {}
            },
        }
    }
}
