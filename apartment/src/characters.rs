use bevy::{render::view::RenderLayers, sprite::Anchor};
use bevy_grid_squared::{square, Square};

use crate::{
    cameras::CHARACTERS_RENDER_LAYER,
    layout::{self, IntoMap, SquareKind},
    prelude::*,
    Apartment,
};

/// Useful for despawning entities when leaving the apartment.
#[derive(Component)]
struct CharacterEntity;

#[derive(Component)]
struct Controllable {
    square: Square,
    next: Option<Square>,
    movement_timer: Stopwatch,
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
    commands.spawn((
        Controllable {
            square: square(-10, 5),
            next: None,
            movement_timer: Stopwatch::new(),
        },
        CharacterEntity,
        RenderLayers::layer(CHARACTERS_RENDER_LAYER),
        SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            texture: asset_server.load(assets::DEBUG_CHARACTER),
            transform: Transform::from_translation(Vec3::new(
                0.0, 0.0,
                100.0, // TODO: this must be variable based on movement
            )),
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
    map: Res<layout::Map<Apartment>>,
    mut character: Query<&mut Controllable>,
) {
    use bevy_grid_squared::direction::Direction;

    let Ok(mut character) = character.get_single_mut() else {
        return;
    };

    if character.next.is_some() {
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
        None => true, // TODO: check not out of bounds
        Some(SquareKind::None) => true,
        Some(SquareKind::Object | SquareKind::Wall) => false,
    };

    // preferably go there if possible
    let main_square = character.square.neighbor(main);

    let target_square = if is_empty(main_square) {
        Some(main_square)
    } else {
        // these are alternatives
        let a_square = character.square.neighbor(a);
        let b_square = character.square.neighbor(b);

        match (is_empty(a_square), is_empty(b_square)) {
            (true, false) => Some(a_square),
            (false, true) => Some(b_square),
            // cannot decide or cannot go anywhere
            (true, true) | (false, false) => None,
        }
    };

    if let Some(target_square) = target_square {
        character.next = Some(target_square);
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

    character.movement_timer.tick(time.delta());

    {
        let elapsed = character.movement_timer.elapsed_secs();
        if elapsed > STEP_DURATION_SECS {
            character.movement_timer.reset();
            character
                .movement_timer
                .tick(Duration::from_secs_f32(elapsed - STEP_DURATION_SECS));

            character.square =
                character.next.take().unwrap_or(character.square);
        }
    }

    {
        let expected_position =
            Apartment::layout().square_to_world_pos(character.square);
        let new_position = transform.translation.lerp(
            expected_position.extend(0.0),
            (character.movement_timer.elapsed_secs() / STEP_DURATION_SECS)
                .min(1.0),
        );
        transform.translation = new_position;
        update_z_based_on_y(&mut transform.translation);
    }
}

fn update_z_based_on_y(t: &mut Vec3) {
    // by a lucky chance, we can use 0.0 as the delimiter between the
    // layers
    //
    // this is stupid but simple and since the room does not
    // require anything more complex, let's roll with it
    if t.y > 0.0 {
        t.z = zindex::BEDROOM_FURNITURE_MIDDLE - 0.1;
    } else {
        t.z = zindex::BEDROOM_FURNITURE_MIDDLE + 0.1;
    }
}
