use bevy::render::view::RenderLayers;

use crate::{cameras::CHARACTERS_RENDER_LAYER, prelude::*};

const MOVEMENT_SPEED: f32 = 75.0;

/// Useful for despawning entities when leaving the apartment.
#[derive(Component)]
struct CharacterEntity;

#[derive(Component)]
struct Controllable;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::ApartmentLoading), spawn)
            .add_systems(OnEnter(GlobalGameState::ApartmentQuitting), despawn);

        app.add_systems(
            Update,
            move_around.run_if(in_state(GlobalGameState::InApartment)),
        );
    }

    fn finish(&self, _: &mut App) {
        //
    }
}

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Controllable,
        CharacterEntity,
        RenderLayers::layer(CHARACTERS_RENDER_LAYER),
        SpriteBundle {
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
    mut query: Query<&mut Transform, With<Controllable>>,
    time: Res<Time>,
) {
    for mut transform in query.iter_mut() {
        let mut movement = Vec3::ZERO;

        if keyboard.pressed(KeyCode::W) {
            movement.y += 1.0;
        }
        if keyboard.pressed(KeyCode::S) {
            movement.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::A) {
            movement.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::D) {
            movement.x += 1.0;
        }

        transform.translation +=
            movement * time.delta_seconds() * MOVEMENT_SPEED;

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
