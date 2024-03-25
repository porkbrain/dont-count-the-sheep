use bevy::render::view::RenderLayers;
use common_loading_screen::LoadingScreenSettings;
use common_store::{ApartmentStore, GlobalStore};
use common_top_down::{
    actor::CharacterExt, layout::LAYOUT, ActorMovementEvent, ActorTarget,
    TileKind,
};
use common_visuals::camera::{render_layer, MainCamera};
use main_game_lib::{common_ext::QueryExt, cutscene::IntoCutscene};

use super::{cutscenes, ApartmentAction};
use crate::{
    layout::{ApartmentTileKind, Elevator, MeditatingHint, SleepingHint},
    prelude::*,
    Apartment,
};

pub(crate) fn spawn(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    store: &GlobalStore,
) -> Vec<Entity> {
    let initial_position = store
        .position_on_load()
        .get()
        .unwrap_or(DEFAULT_INITIAL_POSITION);
    store.position_on_load().remove();

    let walking_to = store
        .walk_to_onload()
        .get()
        .map(|pos| LAYOUT.world_pos_to_square(pos))
        .map(ActorTarget::new);
    store.walk_to_onload().remove();

    let step_time = store.step_time_onload().get();
    store.step_time_onload().remove();

    let mut player =
        cmd.spawn((Player, RenderLayers::layer(render_layer::OBJ)));
    common_story::Character::Winnie
        .bundle_builder()
        .is_player(true)
        .with_initial_position(initial_position)
        .with_walking_to(walking_to)
        .with_initial_step_time(step_time)
        .insert(asset_server, &mut player);
    let player = player.id();

    vec![player]
}

/// Will change the game state to meditation minigame.
pub(super) fn start_meditation_minigame_if_near_chair(
    mut cmd: Commands,
    mut action_events: EventReader<ApartmentAction>,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    store: Res<GlobalStore>,

    player: Query<Entity, With<Player>>,
) {
    let is_triggered = action_events
        .read()
        .any(|action| matches!(action, ApartmentAction::StartMeditation));

    if is_triggered && let Some(entity) = player.get_single_or_none() {
        // when we come back, we want to be next to the chair
        store
            .position_on_load()
            .set(POSITION_ON_LOAD_FROM_MEDITATION);
        store.walk_to_onload().set(WALK_TO_ONLOAD_FROM_MEDITATION);
        store
            .step_time_onload()
            .set(STEP_TIME_ONLOAD_FROM_MEDITATION);

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
    points: Query<(&Name, &common_rscn::Point)>,
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
                let (_, common_rscn::Point(pos)) = points
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
