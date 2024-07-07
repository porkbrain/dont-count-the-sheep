use bevy::{render::view::RenderLayers, time::Stopwatch};
use common_physics::PoissonsEquationUpdateEvent;

use super::{consts::*, PolpoEntity};
use crate::{
    gravity::{ChangeOfBasis, Gravity},
    prelude::*,
};

pub(crate) mod black_hole {
    use bevy::math::uvec2;
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
        let on_last_frame =
            AtlasAnimationEnd::run(Box::new(move |cmd, entity, _, _| {
                debug!("Despawning black hole ({entity:?})");

                cmd.entity(entity).despawn_recursive();

                // remove gravity influence
                cmd.add(move |world: &mut World| {
                    world.send_event(
                        PoissonsEquationUpdateEvent::<Gravity>::new(
                            -BLACK_HOLE_GRAVITY,
                            ChangeOfBasis::new(at_translation),
                        ),
                    );
                });
            }));

        cmd.spawn((
            BlackHole,
            PolpoEntity,
            AtlasAnimation {
                first: 0,
                last: BLACK_HOLE_ATLAS_FRAMES - 1,
                on_last_frame,
                ..default()
            },
            BeginAtlasAnimation {
                cond: common_visuals::BeginAtlasAnimationCond::AtRandom(
                    BLACK_HOLE_DESPAWN_CHANCE_PER_SECOND,
                ),
                frame_time: BLACK_HOLE_FRAME_TIME,
                with_min_delay: Some((BLACK_HOLE_MIN_LIFE, Stopwatch::new())),
            },
            RenderLayers::layer(render_layer::BG),
        ))
        .insert(SpriteBundle {
            texture: asset_server.load(assets::BLACKHOLE_ATLAS),
            transform: Transform::from_translation(
                at_translation.extend(zindex::BLACK_HOLE),
            ),
            ..default()
        })
        .insert(TextureAtlas {
            index: 0,
            layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                uvec2(
                    BLACK_HOLE_SPRITE_SIZE as u32,
                    BLACK_HOLE_SPRITE_SIZE as u32,
                ),
                BLACK_HOLE_ATLAS_FRAMES as u32,
                1,
                None,
                None,
            )),
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
