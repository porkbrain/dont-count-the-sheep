use bevy::{render::view::RenderLayers, time::Stopwatch, utils::Instant};
use common_physics::PoissonsEquationUpdateEvent;

use super::{
    consts::{BOLT_LIFETIME, *},
    PolpoEntity,
};
use crate::{
    gravity::{ChangeOfBasis, Gravity},
    prelude::*,
};

pub(crate) mod bolt {
    use common_visuals::camera::render_layer;

    use super::*;

    /// Special effect that goes from Hoshi to a Polpo that it hit.
    #[derive(Component)]
    pub(crate) struct Bolt {
        /// Relative to the Polpo it's about to hit.
        /// The Polpo is the origin.
        from: Vec2,
        /// Since it's an effect that's supposed to be short-lived, we don't
        /// need the pause functionality of Stopwatch.
        spawned_at: Instant,
    }

    pub(crate) fn propel(
        mut cmd: Commands,

        mut bolts: Query<(Entity, &Bolt, &mut Transform)>,
    ) {
        for (entity, bolt, mut transform) in bolts.iter_mut() {
            let lives_for = bolt.spawned_at.elapsed();

            if lives_for > BOLT_LIFETIME {
                cmd.entity(entity).despawn();
            } else {
                let lerp_factor =
                    lives_for.as_secs_f32() / BOLT_LIFETIME.as_secs_f32();

                let expected_pos = bolt.from.lerp(Vec2::ZERO, lerp_factor);
                transform.translation = expected_pos.extend(zindex::POLPO_BOLT);

                // we need to rotate the bolt to face the towards
                // the destination
                let a = (Vec2::ZERO - bolt.from).angle_between(vec2(1.0, 0.0));
                transform.rotation = Quat::from_rotation_z(-a);
            }
        }
    }

    #[inline]
    pub(crate) fn get_bundle_with_respect_to_origin_at_zero(
        asset_server: &Res<AssetServer>,
        from_with_respect_to_polpo_as_origin: Vec2,
    ) -> impl Bundle {
        (
            Bolt {
                from: from_with_respect_to_polpo_as_origin,
                spawned_at: Instant::now(),
            },
            RenderLayers::layer(render_layer::OBJ),
            SpriteBundle {
                texture: asset_server.load(assets::BOLT),
                transform: {
                    let mut t = Transform::from_translation(
                        from_with_respect_to_polpo_as_origin
                            .extend(zindex::POLPO_BOLT),
                    );

                    // we need to rotate the bolt to face the towards
                    // the destination
                    let a = (Vec2::ZERO - from_with_respect_to_polpo_as_origin)
                        .angle_between(vec2(1.0, 0.0));
                    t.rotate_z(-a);

                    t
                },
                ..default()
            },
        )
    }
}

pub(crate) mod black_hole {
    use common_visuals::camera::render_layer;

    use super::*;

    #[derive(Component)]
    struct BlackHole;

    /// Includes effects of gravity on the poissons equation.
    pub(crate) fn spawn(
        cmd: &mut Commands,
        asset_server: &Res<AssetServer>,
        texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
        gravity: &mut EventWriter<PoissonsEquationUpdateEvent<Gravity>>,
        at_translation: Vec2,
    ) {
        PoissonsEquationUpdateEvent::send(
            gravity,
            BLACK_HOLE_GRAVITY,
            ChangeOfBasis::new(at_translation),
        );

        // the reason why black hole does not despawn while game is paused is
        // that we don't run the system while game is paused
        let on_last_frame = AtlasAnimationEnd::Custom(Box::new(
            move |entity,
                  _animation,
                  _timer,
                  _atlas,
                  _visibility,
                  commands,
                  _time| {
                debug!("Despawning black hole ({entity:?})");

                commands.entity(entity).despawn_recursive();

                // remove gravity influence
                commands.add(move |world: &mut World| {
                    world.send_event(
                        PoissonsEquationUpdateEvent::<Gravity>::new(
                            -BLACK_HOLE_GRAVITY,
                            ChangeOfBasis::new(at_translation),
                        ),
                    );
                });
            },
        ));

        cmd.spawn((
            BlackHole,
            PolpoEntity,
            AtlasAnimation {
                first: 0,
                last: BLACK_HOLE_ATLAS_FRAMES - 1,
                on_last_frame,
                ..default()
            },
            BeginAtlasAnimationAtRandom {
                chance_per_second: BLACK_HOLE_DESPAWN_CHANCE_PER_SECOND,
                frame_time: BLACK_HOLE_FRAME_TIME,
                with_min_delay: Some((BLACK_HOLE_MIN_LIFE, Stopwatch::new())),
            },
            RenderLayers::layer(render_layer::BG),
        ))
        .insert(SpriteSheetBundle {
            texture: asset_server.load(assets::BLACKHOLE_ATLAS),
            atlas: TextureAtlas {
                index: 0,
                layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                    vec2(BLACK_HOLE_SPRITE_SIZE, BLACK_HOLE_SPRITE_SIZE),
                    BLACK_HOLE_ATLAS_FRAMES,
                    1,
                    None,
                    None,
                )),
            },
            transform: Transform::from_translation(
                at_translation.extend(zindex::BLACK_HOLE),
            ),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Flicker::new(
                    BLACK_HOLE_FLICKER_CHANCE_PER_SECOND,
                    BLACK_HOLE_FLICKER_DURATION,
                ),
                RenderLayers::layer(render_layer::OBJ),
                SpriteBundle {
                    texture: asset_server.load(assets::BLACKHOLE_FLICKER),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        0.0,
                        zindex::BLACK_HOLE_TWINKLE,
                    )),
                    ..default()
                },
            ));
        });
    }
}
