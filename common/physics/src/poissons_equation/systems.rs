use bevy::prelude::*;

#[cfg(feature = "poissons-eq-visualization")]
use crate::{GridCoords, VectorArrow, WorldDimensions};
use crate::{PoissonsEquation, PoissonsEquationUpdateEvent};

/// Run it on [`Last`] schedule.
pub(crate) fn update<T: Send + Sync + 'static>(
    mut updates: EventReader<PoissonsEquationUpdateEvent<T>>,
    mut field: ResMut<PoissonsEquation<T>>,
) {
    for PoissonsEquationUpdateEvent { delta, coords, .. } in updates.read() {
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

/// Renders the field as non-overlapping half-transparent arrows.
/// The red nose is where the field points.
#[cfg(feature = "poissons-eq-visualization")]
pub(crate) fn spawn_visualization<
    T: Send + Sync + 'static,
    W: WorldDimensions,
>(
    mut cmd: Commands,

    field: Res<PoissonsEquation<T>>,
    mut images: ResMut<Assets<Image>>,
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
        .expect("Cannot load vector arrow image"),
    );

    for y in 0..field.height {
        for x in 0..field.width {
            // The whole field is a grid of these tiles.
            cmd.spawn((
                VectorArrow,
                SpriteBundle {
                    texture: arrow.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        x as f32 * W::width() / field.width as f32
                            - W::width() / 2.0,
                        y as f32 * W::height() / field.height as f32
                            - W::height() / 2.0,
                        0.0,
                    )),
                    ..default()
                },
            ));
        }
    }
}

/// Adjust the rotation of the arrows to match the field.
#[cfg(feature = "poissons-eq-visualization")]
pub(crate) fn update_visualization<
    T: Send + Sync + 'static,
    P: From<Transform> + Into<GridCoords>,
>(
    field: ResMut<PoissonsEquation<T>>,
    mut vector_arrows: Query<(&mut Transform, &mut Sprite), With<VectorArrow>>,
) {
    use bevy::math::vec2;

    for (mut transform, mut sprite) in vector_arrows.iter_mut() {
        if field.stop_smoothing_out {
            sprite.color.set_a(0.1);
        } else {
            sprite.color.set_a(1.0);

            let gradient = field.gradient_at(P::from(*transform));
            let a = gradient.angle_between(vec2(0.0, 1.0));
            transform.rotation = Quat::from_rotation_z(-a);
        }
    }
}
