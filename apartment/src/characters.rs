use bevy::{render::view::RenderLayers, sprite::Anchor};
use bevy_grid_squared::{square, Square};
use common_layout::{IntoMap, SquareKind};

use crate::{cameras::CHARACTERS_RENDER_LAYER, prelude::*, Apartment};

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

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            texture: asset_server.load(assets::DEBUG_CHARACTER),
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

    // The preferred direction - and if not available, the alternatives.
    // The alternatives must be exclusive, ie. if both are available then none
    // is chosen.
    let (main, (a, b)) = if up && left {
        (Direction::TopLeft, (Direction::Top, Direction::Left))
    } else if up && right {
        (Direction::TopRight, (Direction::Top, Direction::Right))
    } else if down && left {
        (Direction::BottomLeft, (Direction::Bottom, Direction::Left))
    } else if down && right {
        (
            Direction::BottomRight,
            (Direction::Bottom, Direction::Right),
        )
    } else if left {
        (Direction::Left, (Direction::TopLeft, Direction::BottomLeft))
    } else if right {
        (
            Direction::Right,
            (Direction::TopRight, Direction::BottomRight),
        )
    } else if down {
        (
            Direction::Bottom,
            (Direction::BottomLeft, Direction::BottomRight),
        )
    } else if up {
        (Direction::Top, (Direction::TopLeft, Direction::TopRight))
    } else {
        return;
    };

    // exhaustive match in case of future changes
    let is_empty = |square: Square| match map.get(&square) {
        None => Apartment::contains(square),
        Some(SquareKind::None) => true,
        Some(SquareKind::Object | SquareKind::Wall) => false,
    };

    let plan_from = character
        .walking_to
        .as_ref()
        .map(|to| to.square)
        .unwrap_or(character.walking_from);

    // preferably go there if possible
    let main_square = plan_from.neighbor(main);

    let target_square = if is_empty(main_square) {
        Some(main_square)
    } else {
        // these are alternatives
        let a_square = plan_from.neighbor(a);
        let b_square = plan_from.neighbor(b);

        match (is_empty(a_square), is_empty(b_square)) {
            (true, false) => Some(a_square),
            (false, true) => Some(b_square),
            // cannot decide or cannot go anywhere
            (true, true) | (false, false) => None,
        }
    };

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

fn add_z_based_on_y(v: Vec2) -> Vec3 {
    // by a lucky chance, we can use 0.0 as the delimiter between the
    // layers
    //
    // this is stupid but simple and since the room does not
    // require anything more complex, let's roll with it
    v.extend(if v.y > 0.0 {
        zindex::BEDROOM_FURNITURE_MIDDLE - 0.25
    } else {
        zindex::BEDROOM_FURNITURE_MIDDLE + 0.25
    })
}
