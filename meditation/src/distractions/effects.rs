use bevy::{render::view::RenderLayers, utils::Instant};

use crate::{cameras::OBJ_RENDER_LAYER, prelude::*};

use super::consts::BOLT_LIFETIME;

/// Special effect that goes from weather to a distraction that it hit.
#[derive(Component)]
pub(crate) struct Bolt {
    /// Relative to the distraction it's about to hit.
    /// The distraction is the origin.
    from: Pos2,
    /// Since it's an effect that's supposed to be short-lived, we don't need
    /// the pause functionality of Stopwatch.
    spawned_at: Instant,
}

pub(super) fn propel_bolt(
    mut bolts: Query<(Entity, &Bolt, &mut Transform)>,
    mut commands: Commands,
) {
    for (entity, bolt, mut transform) in bolts.iter_mut() {
        let lives_for = bolt.spawned_at.elapsed();

        if lives_for > BOLT_LIFETIME {
            commands.entity(entity).despawn();
        } else {
            let lerp_factor =
                lives_for.as_secs_f32() / BOLT_LIFETIME.as_secs_f32();

            let expected_pos = bolt.from.lerp(Vec2::ZERO, lerp_factor);
            transform.translation =
                expected_pos.extend(zindex::DISTRACTION_BOLT);

            // we need to rotate the bolt to face the towards
            // the destination
            let a = (Vec2::ZERO - bolt.from).angle_between(vec2(1.0, 0.0));
            transform.rotation = Quat::from_rotation_z(-a);
        }
    }
}

pub(super) fn get_bolt_bundle_with_respect_to_origin_at_zero(
    asset_server: &Res<AssetServer>,
    distraction: Pos2,
    weather: Pos2,
) -> impl Bundle {
    let change_of_basis_from = weather - distraction;

    (
        Bolt {
            from: change_of_basis_from,
            spawned_at: Instant::now(),
        },
        RenderLayers::layer(OBJ_RENDER_LAYER),
        SpriteBundle {
            texture: asset_server.load(assets::BOLT),
            transform: {
                let mut t = Transform::from_translation(
                    change_of_basis_from.extend(zindex::DISTRACTION_BOLT),
                );

                // we need to rotate the bolt to face the towards
                // the destination
                let a = (Vec2::ZERO - change_of_basis_from)
                    .angle_between(vec2(1.0, 0.0));
                t.rotate_z(-a);

                t
            },
            ..default()
        },
    )
}
