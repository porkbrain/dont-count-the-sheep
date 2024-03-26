use common_loading_screen::LoadingScreenSettings;
use common_visuals::camera::MainCamera;
use main_game_lib::{common_ext::QueryExt, cutscene::IntoCutscene};
use top_down::{ActorMovementEvent, TileKind};

use super::{cutscenes, ApartmentAction};
use crate::{
    layout::{ApartmentTileKind, Elevator, MeditatingHint, SleepingHint},
    prelude::*,
    Apartment,
};

/// Will change the game state to meditation minigame.
pub(super) fn start_meditation_minigame_if_near_chair(
    mut cmd: Commands,
    mut action_events: EventReader<ApartmentAction>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,

    player: Query<Entity, With<Player>>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, ApartmentAction::StartMeditation));

    if is_triggered && let Some(entity) = player.get_single_or_none() {
        cmd.entity(entity).despawn_recursive();

        cmd.insert_resource(LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::Space),
            stare_at_loading_screen_for_at_least: Some(
                WHEN_ENTERING_MEDITATION_SHOW_LOADING_IMAGE_FOR_AT_LEAST,
            ),
            ..default()
        });

        *transition = GlobalGameStateTransition::ApartmentToMeditation;
        next_state.set(GlobalGameState::ApartmentQuitting);
    }
}

/// By entering the elevator, the player can this scene.
pub(super) fn enter_the_elevator(
    mut cmd: Commands,
    mut action_events: EventReader<ApartmentAction>,

    player: Query<Entity, With<Player>>,
    elevator: Query<Entity, With<Elevator>>,
    camera: Query<Entity, With<MainCamera>>,
    points: Query<(&Name, &rscn::Point)>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, ApartmentAction::EnterElevator));

    if is_triggered && let Some(entity) = player.get_single_or_none() {
        cutscenes::EnterTheElevator {
            player: entity,
            elevator: elevator.single(),
            camera: camera.single(),
            point_in_elevator: {
                let (_, rscn::Point(pos)) = points
                    .iter()
                    .find(|(name, _)| **name == Name::new("InElevator"))
                    .expect("InElevator point not found");

                *pos
            },
        }
        .spawn(&mut cmd);
    }
}

/// Shows hint for bed or for meditating when player is in the zone to actually
/// interact with those objects.
pub(super) fn toggle_zone_hints(
    mut events: EventReader<
        ActorMovementEvent<<Apartment as TopDownScene>::LocalTileKind>,
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
                TileKind::Local(ApartmentTileKind::MeditationZone) => {
                    *meditating.single_mut() = Visibility::Visible;
                }
                TileKind::Local(ApartmentTileKind::BedZone) => {
                    *sleeping.single_mut() = Visibility::Visible;
                }
                _ => {}
            },
            ActorMovementEvent::ZoneLeft { zone, .. } => match *zone {
                TileKind::Local(ApartmentTileKind::MeditationZone) => {
                    *meditating.single_mut() = Visibility::Hidden;
                }
                TileKind::Local(ApartmentTileKind::BedZone) => {
                    *sleeping.single_mut() = Visibility::Hidden;
                }
                _ => {}
            },
        }
    }
}
