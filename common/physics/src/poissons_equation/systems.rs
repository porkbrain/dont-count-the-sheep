use bevy::prelude::*;

use crate::{GridCoords, VectorArrow};

use super::types::{PoissonsEquation, PoissonsEquationUpdateEvent};

/// TODO: Hide behind a feature flag
pub(crate) fn update_poissons_equation(
    mut updates: EventReader<PoissonsEquationUpdateEvent>,
    mut field: ResMut<PoissonsEquation>,
) {
    for PoissonsEquationUpdateEvent { delta, coords } in updates.read() {
        field.set(*coords, *delta);
        field.stop_smoothing_out = false;
    }

    if !field.stop_smoothing_out {
        let correction = field.smooth_out();

        if correction < f32::EPSILON
            && correction < field.last_smoothing_correction
        {
            field.stop_smoothing_out = true;
        }

        field.last_smoothing_correction = correction;
    }
}

pub trait WorldDimensions {
    fn width() -> f32;
    fn height() -> f32;
}

/// Renders the field as non-overlapping half-transparent arrows.
///
/// TODO: hide behind a dev feature flag
pub fn spawn_visualization<W: WorldDimensions>(
    field: Res<PoissonsEquation>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let arrow = images.add(
        Image::from_buffer(
            include_bytes!("../../assets/vector_arrow.png"),
            bevy::render::texture::ImageType::Format(
                bevy::render::texture::ImageFormat::Png,
            ),
            default(),
            false,
            default(),
        )
        .expect("Cannot load vector arrow"),
    );

    // A tile that the vector arrow represents.
    // The whole field is a grid of these tiles.
    let tile_width = W::width() / field.width as f32 - W::width() / 2.0;
    let tile_height = W::height() / field.height as f32 - W::height() / 2.0;

    for y in 0..field.height {
        for x in 0..field.width {
            commands.spawn((
                VectorArrow,
                SpriteBundle {
                    texture: arrow.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        x as f32 * tile_width,
                        y as f32 * tile_height,
                        0.0,
                    )),
                    ..default()
                },
            ));
        }
    }
}

/// Adjust the rotation of the arrows to match the field.
///
/// TODO: hide behind a dev feature flag
pub fn update_visualization<T: From<Transform> + Into<GridCoords>>(
    field: ResMut<PoissonsEquation>,
    mut vector_arrows: Query<&mut Transform, With<VectorArrow>>,
) {
    if field.stop_smoothing_out {
        return;
    }

    for mut transform in vector_arrows.iter_mut() {
        let gradient = field.gradient_at(T::from(*transform));
        let a = gradient.angle_between(Vec2::new(0.0, 1.0));
        transform.rotation = Quat::from_rotation_z(-a);
    }
}
