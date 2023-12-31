use bevy::{render::view::RenderLayers, sprite::Anchor};
use bevy_grid_squared::{square, Square};
use common_layout::{IntoMap, SquareKind};

use crate::{
    cameras::CHARACTERS_RENDER_LAYER, layout::add_z_based_on_y, prelude::*,
    Apartment,
};

const WINNIE_ATLAS_COLS: usize = 15;
const WINNIE_ATLAS_ROWS: usize = 1;
const WINNIE_WIDTH: f32 = 19.0;
const WINNIE_HEIGHT: f32 = 35.0;
const WINNIE_ATLAS_PADDING: f32 = 1.0;

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
}

struct ControllableTarget {
    square: Square,
    for_this_long: Stopwatch,
    planned: Option<Square>,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::ApartmentLoading), spawn)
            .add_systems(OnEnter(GlobalGameState::ApartmentQuitting), despawn);

        app.add_systems(
            Update,
            (move_around.run_if(in_state(GlobalGameState::InApartment)),),
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
    let initial_square = square(-10, 5);
    let translation = add_z_based_on_y(
        Apartment::layout().square_to_world_pos(initial_square),
    );

    commands.spawn((
        Controllable {
            walking_from: initial_square,
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
            transform: Transform::from_translation(translation),
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
/// WASD
/// TODO: Add arrows and key bindings.
fn move_around(
    keyboard: Res<Input<KeyCode>>,
    map: Res<common_layout::Map<Apartment>>,
    mut character: Query<&mut Controllable>,
) {
    use bevy_grid_squared::direction::Direction;

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

    let up = keyboard.pressed(KeyCode::W);
    let down = keyboard.pressed(KeyCode::S);
    let left = keyboard.pressed(KeyCode::A);
    let right = keyboard.pressed(KeyCode::D);

    let up = up && !down;
    let down = down && !up;
    let left = left && !right;
    let right = right && !left;

    // Ordered by priority.
    let next_steps = if up && left {
        [Direction::TopLeft, Direction::Top, Direction::Left]
    } else if up && right {
        [Direction::TopRight, Direction::Top, Direction::Right]
    } else if down && left {
        [Direction::BottomLeft, Direction::Bottom, Direction::Left]
    } else if down && right {
        [Direction::BottomRight, Direction::Bottom, Direction::Right]
    } else if left {
        [Direction::Left, Direction::TopLeft, Direction::BottomLeft]
    } else if right {
        [
            Direction::Right,
            Direction::TopRight,
            Direction::BottomRight,
        ]
    } else if down {
        [
            Direction::Bottom,
            Direction::BottomLeft,
            Direction::BottomRight,
        ]
    } else if up {
        [Direction::Top, Direction::TopLeft, Direction::TopRight]
    } else {
        return;
    };

    // exhaustive match in case of future changes
    let is_available = |square: Square| match map.get(&square) {
        None => Apartment::contains(square),
        Some(SquareKind::None) => true,
        Some(SquareKind::Object | SquareKind::Wall) => false,
    };

    let plan_from = character
        .walking_to
        .as_ref()
        .map(|to| to.square)
        .unwrap_or(character.walking_from);

    let target_square = next_steps.into_iter().find_map(|direction| {
        let target = plan_from.neighbor(direction);
        is_available(target).then_some(target)
    });

    if let Some(target_square) = target_square {
        if let Some(walking_to) = &mut character.walking_to {
            debug_assert!(walking_to.planned.is_none());
            walking_to.planned = Some(target_square);
        } else {
            character.walking_to = Some(ControllableTarget {
                square: target_square,
                for_this_long: Stopwatch::new(),
                planned: None,
            });
        }
    }
}

fn animate_movement(
    mut character: Query<(&mut Controllable, &mut Transform)>,
    time: Res<Time>,
) {
    let Ok((mut character, mut transform)) = character.get_single_mut() else {
        return;
    };

    const STEP_DURATION_SECS: f32 = 0.05; // TODO

    let Some(walking_to) = character.walking_to.as_mut() else {
        return;
    };

    walking_to.for_this_long.tick(time.delta());

    let lerp_factor =
        walking_to.for_this_long.elapsed_secs() / STEP_DURATION_SECS;

    let to = Apartment::layout().square_to_world_pos(walking_to.square);

    if lerp_factor >= 1.0 {
        let new_from = walking_to.square;

        transform.translation = add_z_based_on_y(to);

        if let Some(planned) = walking_to.planned.take() {
            walking_to.for_this_long.reset();
            walking_to.square = planned;
        } else {
            character.walking_to = None;
        }

        character.walking_from = new_from;
    } else {
        let from =
            Apartment::layout().square_to_world_pos(character.walking_from);

        transform.translation = add_z_based_on_y(from.lerp(to, lerp_factor));
    }
}
