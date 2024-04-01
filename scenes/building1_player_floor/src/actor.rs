//! Player and NPCs.

use common_loading_screen::LoadingScreenSettings;
use common_story::{
    dialog::DialogGraph,
    emoji::{DisplayEmojiEvent, DisplayEmojiEventConsumer, EmojiKind},
};
use common_visuals::camera::MainCamera;
use main_game_lib::{
    common_ext::QueryExt,
    cutscene::{self, in_cutscene},
    hud::daybar::DayBar,
    top_down::inspect_and_interact::{
        ChangeHighlightedInspectLabelEvent,
        ChangeHighlightedInspectLabelEventConsumer, SpawnLabelBgAndTextParams,
        ZoneToInspectLabelEntity,
    },
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
            (
                start_meditation_minigame
                    .before(DisplayEmojiEventConsumer)
                    .before(ChangeHighlightedInspectLabelEventConsumer),
                enter_the_elevator,
            )
                .run_if(on_event::<Building1PlayerFloorAction>())
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
fn start_meditation_minigame(
    mut cmd: Commands,
    mut action_events: EventReader<Building1PlayerFloorAction>,
    mut emoji_events: EventWriter<DisplayEmojiEvent>,
    mut inspect_label_events: EventWriter<ChangeHighlightedInspectLabelEvent>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    zone_to_inspect_label_entity: Res<
        ZoneToInspectLabelEntity<Building1PlayerFloorTileKind>,
    >,
    daybar: Res<DayBar>,

    player: Query<Entity, With<Player>>,
) {
    let is_triggered = action_events.read().any(|action| {
        matches!(action, Building1PlayerFloorAction::StartMeditation)
    });

    if is_triggered {
        if daybar.is_depleted() {
            if let Some(entity) = zone_to_inspect_label_entity
                .map
                .get(&Building1PlayerFloorTileKind::MeditationZone)
                .copied()
            {
                inspect_label_events.send(ChangeHighlightedInspectLabelEvent {
                    entity,
                    spawn_params: SpawnLabelBgAndTextParams {
                        highlighted: true,
                        overwrite_font_color: Some(Color::rgb(1.0, 0.7, 0.7)),
                        overwrite_display_text: Some("(too tired)".to_string()),
                    },
                });
            } else {
                error!("Cannot find meditation zone inspect label entity");
            }

            if let Some(on_parent) = player.get_single_or_none() {
                emoji_events.send(DisplayEmojiEvent {
                    emoji: EmojiKind::Tired,
                    on_parent,
                    offset_for: common_story::Character::Winnie,
                });
            } else {
                error!("Cannot find player entity");
            }

            return;
        }

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
