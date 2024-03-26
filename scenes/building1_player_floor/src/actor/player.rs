use common_loading_screen::LoadingScreenSettings;
use common_store::{DialogStore, GlobalStore};
use common_story::dialog;
use common_visuals::camera::MainCamera;
use main_game_lib::{
    common_ext::QueryExt,
    cutscene::{self, IntoCutscene},
};
use top_down::{ActorMovementEvent, TileKind};

use super::Building1PlayerFloorAction;
use crate::{
    layout::{
        Building1PlayerFloorTileKind, Elevator, MeditatingHint, SleepingHint,
    },
    prelude::*,
    Building1PlayerFloor,
};

const EXIT_ELEVATOR_NODE_NAME: &str = "exit_elevator";
const GO_TO_DOWNTOWN_NODE_NAME: &str = "go_to_downtown";

/// Will change the game state to meditation minigame.
pub(super) fn start_meditation_minigame_if_near_chair(
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
        next_state.set(GlobalGameState::QuittingBuilding1PlayerFloor);
    }
}

/// By entering the elevator, the player can this scene.
pub(super) fn enter_the_elevator(
    mut cmd: Commands,
    mut action_events: EventReader<Building1PlayerFloorAction>,

    player: Query<Entity, With<Player>>,
    elevator: Query<Entity, With<Elevator>>,
    camera: Query<Entity, With<MainCamera>>,
    points: Query<(&Name, &rscn::Point)>,
) {
    fn did_choose_to_cancel_and_exit_the_elevator(store: &GlobalStore) -> bool {
        store.was_this_the_last_dialog((
            dialog::TypedNamespace::EnterTheApartmentElevator,
            EXIT_ELEVATOR_NODE_NAME,
        ))
    }

    fn on_took_the_elevator(cmd: &mut Commands, store: &GlobalStore) {
        cmd.insert_resource(LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::random()),
            stare_at_loading_screen_for_at_least: Some(from_millis(2000)),
            ..default()
        });
        cmd.insert_resource(NextState(Some(
            GlobalGameState::QuittingBuilding1PlayerFloor,
        )));

        let go_to_downtown = store.was_this_the_last_dialog((
            dialog::TypedNamespace::EnterTheApartmentElevator,
            GO_TO_DOWNTOWN_NODE_NAME,
        ));

        let transition = if go_to_downtown {
            GlobalGameStateTransition::Building1PlayerFloorToDowntown
        } else {
            // only other possible transition
            GlobalGameStateTransition::Building1PlayerFloorToBuilding1Basement1
        };

        cmd.insert_resource(transition);
    }

    let is_triggered = action_events.read().any(|action| {
        matches!(action, Building1PlayerFloorAction::EnterElevator)
    });

    if is_triggered && let Some(entity) = player.get_single_or_none() {
        cutscene::enter_an_elevator::EnterAnElevator {
            player: entity,
            elevator: elevator.single(),
            camera: camera.single(),
            dialog: dialog::TypedNamespace::EnterTheApartmentElevator,
            is_cancelled: did_choose_to_cancel_and_exit_the_elevator,
            on_took_the_elevator,
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

#[cfg(test)]
mod tests {
    use common_story::dialog::DialogGraph;
    use itertools::Itertools;

    use super::*;

    #[test]
    fn it_has_exit_elevator_node_and_go_to_downtown_node() {
        let namespace = dialog::TypedNamespace::EnterTheApartmentElevator;

        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = format!(
            "{manifest}/../../main_game/assets/dialogs/{namespace}.toml"
        );
        let toml = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{path}: {e}"));

        let dialog = DialogGraph::subgraph_from_raw(namespace.into(), &toml);

        let nodes = dialog
            .node_names()
            .filter_map(|node| {
                let (_, name) = node.as_namespace_and_node_name_str()?;
                Some(name.to_owned())
            })
            .collect_vec();

        nodes
            .iter()
            .find(|name| name.as_str() == EXIT_ELEVATOR_NODE_NAME)
            .expect("Exit elevator node not found");

        nodes
            .iter()
            .find(|name| name.as_str() == GO_TO_DOWNTOWN_NODE_NAME)
            .expect("Go to downtown node not found");
    }
}
