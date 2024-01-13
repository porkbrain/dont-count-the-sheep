use bevy::{render::view::RenderLayers, sprite::Anchor};
use bevy_grid_squared::{direction::Direction as GridDirection, Square};
use common_layout::{IntoMap, SquareKind};
use common_loading_screen::LoadingScreenSettings;
use common_store::{ApartmentStore, GlobalStore};
use common_story::portrait_dialog::{
    example::Example1, not_in_portrait_dialog,
};
use common_visuals::camera::render_layer;
use leafwing_input_manager::action_state::ActionState;
use main_game_lib::{
    interaction_pressed, move_action_pressed, GlobalAction,
    GlobalGameStateTransition, GlobalGameStateTransitionStack,
};

use crate::{
    consts::WHEN_ENTERING_MEDITATION_SHOW_LOADING_IMAGE_FOR_AT_LEAST,
    layout::{add_z_based_on_y, zones},
    prelude::*,
    Apartment,
};

const WINNIE_ATLAS_COLS: usize = 15;
const WINNIE_ATLAS_ROWS: usize = 1;
const WINNIE_WIDTH: f32 = 19.0;
const WINNIE_HEIGHT: f32 = 35.0;
const WINNIE_ATLAS_PADDING: f32 = 1.0;
/// How long does it take to move one square.
const DEFAULT_STEP_TIME: Duration = from_millis(50);
/// When the apartment is loaded, the character is spawned facing this
/// direction.
const INITIAL_DIRECTION: GridDirection = GridDirection::Bottom;
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

#[derive(Component, Reflect)]
struct Controllable {
    step_time: Duration,
    /// If no target then this is the current position.
    /// If there's a target, current position is interpolated between this and
    /// the target.
    walking_from: Square,
    walking_to: Option<ControllableTarget>,
    /// Used for animations.
    direction: GridDirection,
}

#[derive(Reflect)]
struct ControllableTarget {
    square: Square,
    since: Stopwatch,
    planned: Option<(Square, GridDirection)>,
}

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
                move_around.run_if(move_action_pressed()),
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
            animate_movement.run_if(in_state(GlobalGameState::InApartment)),
        );
    }
}

fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    store: Res<GlobalStore>,
) {
    let initial_position = store
        .position_on_load()
        .get()
        .unwrap_or(DEFAULT_INITIAL_POSITION);
    let walking_from =
        Apartment::layout().world_pos_to_square(initial_position);
    store.position_on_load().remove();

    let walking_to = store
        .walk_to_onload()
        .get()
        .map(|pos| Apartment::layout().world_pos_to_square(pos))
        .map(ControllableTarget::new);
    store.walk_to_onload().remove();

    let step_time = store.step_time_onload().get().unwrap_or(DEFAULT_STEP_TIME);
    store.step_time_onload().remove();

    cmd.spawn((
        Name::from("Controllable"),
        Controllable {
            step_time,
            direction: INITIAL_DIRECTION,
            walking_from,
            walking_to,
        },
        CharacterEntity,
        RenderLayers::layer(render_layer::OBJ),
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load(assets::WINNIE_ATLAS),
                vec2(WINNIE_WIDTH, WINNIE_HEIGHT),
                WINNIE_ATLAS_COLS,
                WINNIE_ATLAS_ROWS,
                Some(vec2(WINNIE_ATLAS_PADDING, 0.0)),
                None,
            )),
            sprite: TextureAtlasSprite {
                anchor: Anchor::BottomCenter,
                index: 0,
                ..default()
            },
            transform: Transform::from_translation(add_z_based_on_y(
                initial_position,
            )),
            ..default()
        },
    ));

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

/// Use keyboard to move around.
fn move_around(
    map: Res<common_layout::Map<Apartment>>,
    controls: Res<ActionState<GlobalAction>>,

    mut character: Query<&mut Controllable>,
) {
    use GridDirection::*;

    let next_steps = match controls.get_pressed().last() {
        Some(GlobalAction::MoveUp) => [Top, TopLeft, TopRight],
        Some(GlobalAction::MoveDown) => [Bottom, BottomLeft, BottomRight],
        Some(GlobalAction::MoveLeft) => [Left, TopLeft, BottomLeft],
        Some(GlobalAction::MoveRight) => [Right, TopRight, BottomRight],
        Some(GlobalAction::MoveUpLeft) => [TopLeft, Top, Left],
        Some(GlobalAction::MoveUpRight) => [TopRight, Top, Right],
        Some(GlobalAction::MoveDownLeft) => [BottomLeft, Bottom, Left],
        Some(GlobalAction::MoveDownRight) => [BottomRight, Bottom, Right],
        _ => {
            return;
        }
    };

    let Ok(mut character) = character.get_single_mut() else {
        return;
    };

    if character
        .walking_to
        .as_ref()
        .and_then(|to| to.planned)
        .is_some()
    {
        return;
    }

    // exhaustive match in case of future changes
    let is_available = |square: Square| match map.get(&square) {
        None => Apartment::contains(square),
        Some(SquareKind::None | SquareKind::Zone(_)) => true,
        Some(SquareKind::Object | SquareKind::Wall) => false,
    };

    let plan_from = character.current_square();

    let target = next_steps.iter().copied().find_map(|direction| {
        let target = plan_from.neighbor(direction);
        is_available(target).then_some((target, direction))
    });

    character.step_time = DEFAULT_STEP_TIME;
    if let Some((target_square, direction)) = target {
        character.direction = direction;

        if let Some(walking_to) = &mut character.walking_to {
            debug_assert!(walking_to.planned.is_none());
            walking_to.planned = Some((target_square, direction));
        } else {
            character.walking_to = Some(ControllableTarget::new(target_square));
        }
    } else {
        // Cannot move anywhere, but would like to? At least direction the
        // sprite towards the attempted direction.

        character.direction = next_steps[0];
    }
}

/// Transform is queried separately so that we can listen to just changes to it
/// when deciding whether to show some proximity UI.
fn animate_movement(
    mut character: Query<(&mut Controllable, &mut TextureAtlasSprite)>,
    mut transform: Query<&mut Transform, With<Controllable>>,
    time: Res<Time>,
) {
    use GridDirection::*;

    let Ok((mut character, mut sprite)) = character.get_single_mut() else {
        return;
    };

    let current_direction = character.direction;
    let step_time = character.step_time;
    let standing_still_sprite_index = match current_direction {
        Bottom => 0,
        Top => 1,
        Right | TopRight | BottomRight => 6,
        Left | TopLeft | BottomLeft => 9,
    };

    let Some(walking_to) = character.walking_to.as_mut() else {
        sprite.index = standing_still_sprite_index;

        return;
    };

    walking_to.since.tick(time.delta());

    let lerp_factor = walking_to.since.elapsed_secs()
        / if let Top | Bottom | Left | Right = current_direction {
            step_time.as_secs_f32()
        } else {
            // we need to walk a bit slower when walking diagonally because
            // we cover more distance
            step_time.as_secs_f32() * 2.0f32.sqrt()
        };

    let mut transform = transform.single_mut();
    let to = Apartment::layout().square_to_world_pos(walking_to.square);

    if lerp_factor >= 1.0 {
        let new_from = walking_to.square;

        transform.translation = add_z_based_on_y(to);

        if let Some((new_square, new_direction)) = walking_to.planned.take() {
            walking_to.since.reset();
            walking_to.square = new_square;
            character.direction = new_direction;
        } else {
            sprite.index = standing_still_sprite_index;

            character.walking_to = None;
        }

        character.walking_from = new_from;
    } else {
        let animation_step_time =
            animation_step_secs(step_time.as_secs_f32(), current_direction);
        let extra =
            (time.elapsed_seconds() / animation_step_time).floor() as usize % 2;

        sprite.index = match current_direction {
            Top => 2 + extra,
            Bottom => 4 + extra,
            Right | TopRight | BottomRight => 7 + extra,
            Left | TopLeft | BottomLeft => 10 + extra,
        };

        let from =
            Apartment::layout().square_to_world_pos(character.walking_from);

        transform.translation = add_z_based_on_y(from.lerp(to, lerp_factor));
    }
}

/// TODO: This is here for purposes only
fn start_conversation(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    map: Res<common_layout::Map<Apartment>>,

    character: Query<&Controllable>,
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

    character: Query<(Entity, &Controllable)>,
    mut overlay: Query<&mut Sprite, With<TransparentOverlay>>,
) {
    let Ok((entity, character)) = character.get_single() else {
        return;
    };

    let square = character.current_square();
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
        bg_image_asset: Some(common_assets::paths::meditation::LOADING_SCREEN),
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
    character: Query<&Controllable, Changed<Transform>>,
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

impl Controllable {
    fn current_square(&self) -> Square {
        self.walking_to
            .as_ref()
            .map(|to| to.square)
            .unwrap_or(self.walking_from)
    }
}

impl ControllableTarget {
    fn new(square: Square) -> Self {
        Self {
            square,
            since: Stopwatch::new(),
            planned: None,
        }
    }
}

/// How often we change walking frame based on how fast we're walking from
/// square to square.
fn animation_step_secs(step_secs: f32, dir: GridDirection) -> f32 {
    match dir {
        GridDirection::Top | GridDirection::Bottom => step_secs * 5.0,
        _ => step_secs * 3.5,
    }
    .clamp(0.1, 0.5)
}
