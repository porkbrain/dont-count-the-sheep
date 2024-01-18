//! Things that player can encounter in this scene.

use bevy::{ecs::event::event_update_condition, render::view::RenderLayers};
use bevy_grid_squared::{GridDirection, Square};
use common_loading_screen::LoadingScreenSettings;
use common_store::{ApartmentStore, GlobalStore};
use common_story::portrait_dialog::not_in_portrait_dialog;
use common_visuals::camera::render_layer;
use main_game_lib::{
    common_action::{interaction_pressed, move_action_pressed},
    common_store::DialogStore,
    common_story::portrait_dialog::apartment_elevator::TakeTheElevatorToGroundFloor,
    common_top_down::{
        actor::{self, CharacterExt},
        Actor, ActorMovementEvent, ActorTarget, IntoMap, SquareKind,
    },
    cutscene::IntoCutscene,
    GlobalGameStateTransition, GlobalGameStateTransitionStack,
};

use crate::{
    consts::*,
    layout::{zones, Elevator},
    prelude::*,
    Apartment,
};

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
                common_top_down::actor::player::move_around::<Apartment>
                    .run_if(move_action_pressed()),
                start_meditation_minigame_if_near_chair
                    .run_if(interaction_pressed()),
                enter_the_elevator.run_if(interaction_pressed()),
            )
                .run_if(in_state(GlobalGameState::InApartment))
                .run_if(not_in_portrait_dialog()),
        );

        app.add_systems(
            Update,
            load_zone_overlay
                .run_if(event_update_condition::<ActorMovementEvent>)
                .run_if(in_state(GlobalGameState::InApartment))
                .after(actor::emit_movement_events::<Apartment>),
        );

        app.add_systems(
            FixedUpdate,
            common_top_down::actor::animate_movement::<Apartment>
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

/// Will change the game state to meditation minigame.
fn start_meditation_minigame_if_near_chair(
    mut cmd: Commands,
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    store: Res<GlobalStore>,
    map: Res<common_top_down::Map<Apartment>>,

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

/// By entering the elevator, the player can this scene.
fn enter_the_elevator(
    mut cmd: Commands,
    map: Res<common_top_down::Map<Apartment>>,

    player: Query<(Entity, &Actor), With<Player>>,
    elevator: Query<Entity, With<Elevator>>,
) {
    let Ok((entity, player)) = player.get_single() else {
        return;
    };

    let square = player.current_square();
    if !matches!(map.get(&square), Some(SquareKind::Zone(zones::ELEVATOR))) {
        return;
    }

    cutscenes::EnterTheElevator {
        player: entity,
        elevator: elevator.single(),
    }
    .spawn(&mut cmd);
}

/// Zone overlay is a half transparent image that shows up when the character
/// gets close to certain zones.
/// We hide it if the character is not close to any zone.
/// We change the image to the appropriate one based on the zone.
fn load_zone_overlay(
    mut events: EventReader<ActorMovementEvent>,

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

    let (new_visibility, new_image) = match event {
        ActorMovementEvent::ZoneEntered { zone, .. } => match *zone {
            zones::MEDITATION => {
                (Visibility::Visible, Some(assets::WINNIE_MEDITATING))
            }
            zones::BED => (Visibility::Visible, Some(assets::WINNIE_SLEEPING)),
            zones::TEA => {
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

mod cutscenes {
    use main_game_lib::{
        common_story::portrait_dialog::DialogRoot,
        cutscene::{CutsceneStep, IntoCutscene},
        GlobalGameStateTransition as Ggst,
    };

    use super::*;

    pub(super) struct EnterTheElevator {
        pub(super) player: Entity,
        pub(super) elevator: Entity,
    }

    impl IntoCutscene for EnterTheElevator {
        fn sequence(self) -> Vec<CutsceneStep> {
            use CutsceneStep::*;
            let Self { player, elevator } = self;

            vec![
                RemovePlayerComponent(player),
                InsertAnimationTimerTo {
                    entity: elevator,
                    duration: from_millis(150),
                    mode: TimerMode::Repeating,
                },
                Sleep(from_millis(1250)),
                BeginSimpleWalkTo {
                    with: player,
                    square: Square::new(-57, -19),
                    planned: Some((
                        Square::new(-57, -20),
                        GridDirection::Bottom,
                    )),
                    step_time: None,
                },
                WaitUntilActorAtRest(player),
                Sleep(from_millis(300)),
                BeginPortraitDialog(DialogRoot::EnteredTheElevator),
                WaitForPortraitDialogToEnd,
                Sleep(from_millis(300)),
                IfTrueThisElseThat(
                    chose_to_leave,
                    Box::new(vec![ChangeGlobalState {
                        to: GlobalGameState::ApartmentQuitting,
                        with: Ggst::ApartmentQuittingToDowntownLoading,
                        loading_screen: Some(LoadingScreenSettings {
                            ..default()
                        }),
                        // this is already done in this scene's smooth exit sys
                        change_loading_screen_state_to_start: false,
                    }]),
                    Box::new(vec![
                        BeginSimpleWalkTo {
                            with: player,
                            square: Square::new(-57, -22),
                            step_time: Some(STEP_TIME_ON_EXIT_ELEVATOR),
                            planned: None,
                        },
                        WaitUntilActorAtRest(player),
                        // TODO: close the elevator?
                        AddPlayerComponent(player),
                    ]),
                ),
            ]
        }
    }
}

fn chose_to_leave(store: &GlobalStore) -> bool {
    store.was_this_the_last_dialog(TakeTheElevatorToGroundFloor)
}
