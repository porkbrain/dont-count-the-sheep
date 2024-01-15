use bevy::render::view::RenderLayers;
use common_actor::{player::Player, Actor, ActorTarget, CharacterExt};
use common_layout::{IntoMap, SquareKind};
use common_loading_screen::LoadingScreenSettings;
use common_store::{ApartmentStore, GlobalStore};
use common_story::portrait_dialog::{
    example::Example1, not_in_portrait_dialog,
};
use common_visuals::camera::render_layer;
use main_game_lib::{
    common_action::{interaction_pressed, move_action_pressed},
    GlobalGameStateTransition, GlobalGameStateTransitionStack,
};

use crate::{
    consts::WHEN_ENTERING_MEDITATION_SHOW_LOADING_IMAGE_FOR_AT_LEAST,
    layout::zones, prelude::*, Apartment,
};

/// When the apartment is loaded, the character is spawned at this square.
const DEFAULT_INITIAL_POSITION: Vec2 = vec2(-15.0, 15.0);
/// Upon going to the meditation minigame we set this value so that once the
/// game is closed, the character is spawned next to the meditation chair.
const POSITION_ON_LOAD_FROM_MEDITATION: Vec2 = vec2(25.0, 60.0);
/// And it does a little animation of walking down.
const WALK_TO_ONLOAD_FROM_MEDITATION: Vec2 = vec2(25.0, 40.0);
/// Walk down slowly otherwise it'll happen before the player even sees it.
const STEP_TIME_ONLOAD_FROM_MEDITATION: Duration = from_millis(750);

/// Useful for despawning entities when leaving the apartment.
#[derive(Component, Reflect)]
struct CharacterEntity;

/// When the character gets closer to certain zones, show UI to make it easier
/// to visually identify what's going on.
#[derive(Component, Reflect)]
struct TransparentOverlay;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::ApartmentLoading), spawn)
            .add_systems(OnExit(GlobalGameState::ApartmentQuitting), despawn);

        app.add_systems(
            Update,
            (
                common_actor::player::move_around::<Apartment>
                    .run_if(move_action_pressed()),
                load_zone_overlay,
                start_meditation_minigame_if_near_chair
                    .run_if(interaction_pressed()),
                start_conversation.run_if(interaction_pressed()),
            )
                .run_if(in_state(GlobalGameState::InApartment))
                .run_if(not_in_portrait_dialog()),
        );

        app.add_systems(
            FixedUpdate,
            common_actor::animate_movement::<Apartment>
                .run_if(in_state(GlobalGameState::InApartment)),
        );
    }
}

fn spawn(
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
                zindex::BEDROOM_FURNITURE_DISTANT + 0.1,
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

fn despawn(
    mut cmd: Commands,
    characters: Query<Entity, With<CharacterEntity>>,
) {
    debug!("Despawning character entities");

    for entity in characters.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

/// TODO: This is here for debug purposes only
fn start_conversation(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    map: Res<common_layout::Map<Apartment>>,

    character: Query<&Actor>,
) {
    let Ok(character) = character.get_single() else {
        return;
    };

    let square = character.current_square();
    if !matches!(map.get(&square), Some(SquareKind::Zone(zones::BED))) {
        return;
    }

    Example1::spawn(&mut cmd, &asset_server);
}

/// Will change the game state to meditation minigame.
fn start_meditation_minigame_if_near_chair(
    mut cmd: Commands,
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    store: Res<GlobalStore>,
    map: Res<common_layout::Map<Apartment>>,

    player: Query<(Entity, &Actor), With<Player>>,
    mut overlay: Query<&mut Sprite, With<TransparentOverlay>>,
) {
    let Ok((entity, player)) = player.get_single() else {
        return;
    };

    let square = player.current_square();
    if !matches!(map.get(&square), Some(SquareKind::Zone(zones::MEDITATION))) {
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

/// Zone overlay is a half transparent image that shows up when the character
/// gets close to certain zones.
/// We hide it if the character is not close to any zone.
/// We change the image to the appropriate one based on the zone.
fn load_zone_overlay(
    map: Res<common_layout::Map<Apartment>>,
    character: Query<&Actor, (Changed<Transform>, With<Player>)>,
    mut overlay: Query<
        (&mut Visibility, &mut Handle<Image>),
        With<TransparentOverlay>,
    >,
    asset_server: Res<AssetServer>,
) {
    let Ok(character) = character.get_single() else {
        return;
    };

    let (mut visibility, mut image) = overlay.single_mut();

    let square = character.current_square();
    let (new_visibility, new_image) = match map.get(&square) {
        Some(SquareKind::Zone(zones::MEDITATION)) => {
            (Visibility::Visible, Some(assets::WINNIE_MEDITATING))
        }
        Some(SquareKind::Zone(zones::BED)) => {
            (Visibility::Visible, Some(assets::WINNIE_SLEEPING))
        }
        Some(SquareKind::Zone(zones::DOOR)) => {
            unimplemented!()
        }
        Some(SquareKind::Zone(zones::TEA)) => {
            unimplemented!()
        }
        _ => (Visibility::Hidden, None),
    };

    *visibility = new_visibility;

    if let Some(new_image) = new_image {
        *image = asset_server
            .get_handle(new_image)
            .unwrap_or_else(|| asset_server.load(new_image));
    }
}
