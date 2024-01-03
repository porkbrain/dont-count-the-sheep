use bevy::{render::view::RenderLayers, sprite::Anchor};
use bevy_grid_squared::{
    direction::Direction as GridDirection, square, Square,
};
use common_layout::{IntoMap, SquareKind};

use crate::{
    cameras::CHARACTERS_RENDER_LAYER,
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
const STEP_TIME: Duration = from_millis(50);
/// How often do we change the animation frame.
const STEP_ANIMATION_ALTERNATION: Duration = from_millis(250);
/// When the apartment is loaded, the character is spawned facing this
/// direction.
/// TODO: it will depend on the system actually
const INITIAL_DIRECTION: GridDirection = GridDirection::Bottom;
/// When the apartment is loaded, the character is spawned at this square.
/// TODO: it will depend on the system actually
const INITIAL_SQUARE: Square = square(-10, 5);

/// Useful for despawning entities when leaving the apartment.
#[derive(Component)]
struct CharacterEntity;

#[derive(Component)]
struct Controllable {
    /// If no target then this is the current position.
    /// If there's a target, current position is interpolated between this and
    /// the target.
    walking_from: Square,
    walking_to: Option<ControllableTarget>,
    /// Used for animations.
    direction: GridDirection,
}

struct ControllableTarget {
    square: Square,
    for_this_long: Stopwatch,
    planned: Option<(Square, GridDirection)>,
}

/// When the character gets closer to certain zones, show UI to make it easier
/// to visually identify what's going on.
#[derive(Component)]
struct TransparentOverlay;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::ApartmentLoading), spawn)
            .add_systems(OnEnter(GlobalGameState::ApartmentQuitting), despawn);

        app.add_systems(
            Update,
            (move_around, load_zone_overlay)
                .run_if(in_state(GlobalGameState::InApartment)),
        )
        .add_systems(
            FixedUpdate,
            animate_movement.run_if(in_state(GlobalGameState::InApartment)),
        );
    }

    fn finish(&self, _: &mut App) {
        //
    }
}

fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn((
        Controllable {
            direction: INITIAL_DIRECTION,
            walking_from: INITIAL_SQUARE,
            walking_to: None,
        },
        CharacterEntity,
        RenderLayers::layer(CHARACTERS_RENDER_LAYER),
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
                Apartment::layout().square_to_world_pos(INITIAL_SQUARE),
            )),
            ..default()
        },
    ));

    commands.spawn((
        TransparentOverlay,
        CharacterEntity,
        RenderLayers::layer(CHARACTERS_RENDER_LAYER),
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
    mut commands: Commands,
    characters: Query<Entity, With<CharacterEntity>>,
) {
    for entity in characters.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Use keyboard to move around.
fn move_around(
    keyboard: Res<Input<KeyCode>>,
    map: Res<common_layout::Map<Apartment>>,
    mut character: Query<&mut Controllable>,
) {
    use GridDirection::*;

    let (up, down, left, right) = {
        let up = keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up);
        let down =
            keyboard.pressed(KeyCode::S) || keyboard.pressed(KeyCode::Down);
        let left =
            keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left);
        let right =
            keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right);

        (up && !down, down && !up, left && !right, right && !left)
    };

    // Ordered by priority.
    let next_steps = if up && left {
        [TopLeft, Top, Left]
    } else if up && right {
        [TopRight, Top, Right]
    } else if down && left {
        [BottomLeft, Bottom, Left]
    } else if down && right {
        [BottomRight, Bottom, Right]
    } else if left {
        [Left, TopLeft, BottomLeft]
    } else if right {
        [Right, TopRight, BottomRight]
    } else if down {
        [Bottom, BottomLeft, BottomRight]
    } else if up {
        [Top, TopLeft, TopRight]
    } else {
        return;
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

    if let Some((target_square, direction)) = target {
        character.direction = direction;

        if let Some(walking_to) = &mut character.walking_to {
            debug_assert!(walking_to.planned.is_none());
            walking_to.planned = Some((target_square, direction));
        } else {
            character.walking_to = Some(ControllableTarget {
                square: target_square,
                for_this_long: Stopwatch::new(),
                planned: None,
            });
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

    walking_to.for_this_long.tick(time.delta());

    let lerp_factor = walking_to.for_this_long.elapsed_secs()
        / if let Top | Bottom | Left | Right = current_direction {
            STEP_TIME.as_secs_f32()
        } else {
            // we need to walk a bit slower when walking diagonally because
            // we cover more distance
            STEP_TIME.as_secs_f32() * 2.0f32.sqrt()
        };

    let mut transform = transform.single_mut();
    let to = Apartment::layout().square_to_world_pos(walking_to.square);

    if lerp_factor >= 1.0 {
        let new_from = walking_to.square;

        transform.translation = add_z_based_on_y(to);

        if let Some((new_square, new_direction)) = walking_to.planned.take() {
            walking_to.for_this_long.reset();
            walking_to.square = new_square;
            character.direction = new_direction;
        } else {
            sprite.index = standing_still_sprite_index;

            character.walking_to = None;
        }

        character.walking_from = new_from;
    } else {
        let extra = (time.elapsed_seconds()
            / STEP_ANIMATION_ALTERNATION.as_secs_f32())
        .floor() as usize
            % 2;

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
