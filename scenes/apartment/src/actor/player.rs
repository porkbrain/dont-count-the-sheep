use bevy::render::view::RenderLayers;
use common_loading_screen::LoadingScreenSettings;
use common_store::{ApartmentStore, GlobalStore};
use common_visuals::camera::render_layer;
use main_game_lib::{
    common_ext::QueryExt,
    common_top_down::{
        actor::CharacterExt, Actor, ActorMovementEvent, ActorTarget, IntoMap,
        TileKind,
    },
    cutscene::IntoCutscene,
    GlobalGameStateTransition, GlobalGameStateTransitionStack,
};

use super::{cutscenes, CharacterEntity};
use crate::{
    cameras::CameraEntity,
    consts::*,
    layout::{ApartmentTileKind, Elevator},
    prelude::*,
    Apartment,
};

/// When the character gets closer to certain zones, show UI to make it easier
/// to visually identify what's going on.
#[derive(Component, Reflect)]
pub(super) struct TransparentOverlay;

pub(super) fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    store: Res<GlobalStore>,
) {
    let initial_position = store
        .position_on_load()
        .get()
        .unwrap_or(DEFAULT_INITIAL_POSITION);
    store.position_on_load().remove();

    let walking_to = store
        .walk_to_onload()
        .get()
        .map(|pos| Apartment::layout().world_pos_to_square(pos))
        .map(ActorTarget::new);
    store.walk_to_onload().remove();

    let step_time = store.step_time_onload().get();
    store.step_time_onload().remove();

    cmd.spawn((
        Player,
        CharacterEntity,
        RenderLayers::layer(render_layer::OBJ),
    ))
    .insert(
        common_story::Character::Winnie
            .bundle_builder()
            .is_player(true)
            .with_initial_position(initial_position)
            .with_walking_to(walking_to)
            .with_initial_step_time(step_time)
            .build::<Apartment>(),
    );

    cmd.spawn((
        Name::from("Transparent overlay"),
        TransparentOverlay,
        CharacterEntity,
        RenderLayers::layer(render_layer::OBJ),
        SpriteBundle {
            texture: asset_server.load(assets::WINNIE_MEDITATING),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                zindex::BACKWALL_FURNITURE + 0.1,
            )),
            visibility: Visibility::Hidden,
            sprite: Sprite {
                color: Color::WHITE.with_a(0.5),
                ..default()
            },
            ..default()
        },
    ));
}

/// Will change the game state to meditation minigame.
pub(super) fn start_meditation_minigame_if_near_chair(
    mut cmd: Commands,
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    store: Res<GlobalStore>,
    map: Res<common_top_down::TileMap<Apartment>>,

    player: Query<(Entity, &Actor), With<Player>>,
    mut overlay: Query<&mut Sprite, With<TransparentOverlay>>,
) {
    let Some((entity, player)) = player.get_single_or_none() else {
        return;
    };

    let square = player.current_square();
    if !map.is_on(square, ApartmentTileKind::MeditationZone) {
        return;
    }

    // when we come back, we want to be next to the chair
    store
        .position_on_load()
        .set(POSITION_ON_LOAD_FROM_MEDITATION);
    store.walk_to_onload().set(WALK_TO_ONLOAD_FROM_MEDITATION);
    store
        .step_time_onload()
        .set(STEP_TIME_ONLOAD_FROM_MEDITATION);

    cmd.entity(entity).despawn_recursive();
    overlay.single_mut().color.set_a(1.0);

    cmd.insert_resource(LoadingScreenSettings {
        bg_image_asset: Some(common_assets::meditation::LOADING_SCREEN),
        stare_at_loading_screen_for_at_least: Some(
            WHEN_ENTERING_MEDITATION_SHOW_LOADING_IMAGE_FOR_AT_LEAST,
        ),
        ..default()
    });

    stack.push(GlobalGameStateTransition::ApartmentQuittingToMeditationLoading);
    next_state.set(GlobalGameState::ApartmentQuitting);
}

/// By entering the elevator, the player can this scene.
pub(super) fn enter_the_elevator(
    mut cmd: Commands,
    map: Res<common_top_down::TileMap<Apartment>>,

    player: Query<(Entity, &Actor), With<Player>>,
    elevator: Query<Entity, With<Elevator>>,
    camera: Query<Entity, With<CameraEntity>>,
) {
    let Some((entity, player)) = player.get_single_or_none() else {
        return;
    };

    let square = player.current_square();
    if !map.is_on(square, ApartmentTileKind::ElevatorZone) {
        return;
    }

    cutscenes::EnterTheElevator {
        player: entity,
        elevator: elevator.single(),
        camera: camera.single(),
    }
    .spawn(&mut cmd);
}

/// Zone overlay is a half transparent image that shows up when the character
/// gets close to certain zones.
/// We hide it if the character is not close to any zone.
/// We change the image to the appropriate one based on the zone.
pub(super) fn load_zone_overlay(
    mut events: EventReader<
        ActorMovementEvent<<Apartment as IntoMap>::LocalTileKind>,
    >,

    mut overlay: Query<
        (&mut Visibility, &mut Handle<Image>),
        With<TransparentOverlay>,
    >,
    asset_server: Res<AssetServer>,
) {
    let Some(event) = events.read().filter(|event| event.is_player()).last()
    else {
        return;
    };

    // TODO: refactor
    let (new_visibility, new_image) = match event {
        ActorMovementEvent::ZoneEntered { zone, .. } => match *zone {
            TileKind::Local(ApartmentTileKind::MeditationZone) => {
                (Visibility::Visible, Some(assets::WINNIE_MEDITATING))
            }
            TileKind::Local(ApartmentTileKind::BedZone) => {
                (Visibility::Visible, Some(assets::WINNIE_SLEEPING))
            }
            TileKind::Local(ApartmentTileKind::TeaZone) => {
                unimplemented!()
            }
            _ => (Visibility::Hidden, None),
        },
        ActorMovementEvent::ZoneLeft { .. } => (Visibility::Hidden, None),
    };

    let (mut visibility, mut image) = overlay.single_mut();

    *visibility = new_visibility;

    if let Some(new_image) = new_image {
        *image = asset_server
            .get_handle(new_image)
            .unwrap_or_else(|| asset_server.load(new_image));
    }
}
