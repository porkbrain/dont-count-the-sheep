use bevy::render::view::RenderLayers;
use bevy::time::Stopwatch;
use common_physics::{GridCoords, PoissonsEquationUpdateEvent};

use crate::cameras::BG_RENDER_LAYER;
use crate::{
    gravity::{ChangeOfBasis, Gravity},
    prelude::*,
};

use super::consts::*;
use super::DistractionEntity;

#[derive(Component)]
struct BlackHole(GridCoords, Stopwatch);

/// Includes effects of gravity on the poissons equation.
pub(super) fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    gravity: &mut EventWriter<PoissonsEquationUpdateEvent<Gravity>>,
    at_translation: Vec2,
) {
    let gravity_grid_coords = PoissonsEquationUpdateEvent::send(
        gravity,
        BLACK_HOLE_GRAVITY,
        ChangeOfBasis::new(at_translation),
    );

    let on_last_frame = AnimationEnd::Custom(Box::new(
        move |entity,
              _animation,
              _timer,
              _atlas,
              _visibility,
              commands,
              _time| {
            // delete the black hole
            commands.entity(entity).despawn_recursive();

            // remove gravity influence
            commands.add(move |world: &mut World| {
                world.send_event(PoissonsEquationUpdateEvent::<Gravity>::new(
                    -BLACK_HOLE_GRAVITY,
                    ChangeOfBasis::new(at_translation),
                ))
            });
        },
    ));

    commands
        .spawn((
            BlackHole(gravity_grid_coords, Stopwatch::new()),
            DistractionEntity,
            Animation {
                first: 0,
                last: BLACK_HOLE_ATLAS_FRAMES - 1,
                on_last_frame,
            },
            BeginAnimationAtRandom {
                chance_per_second: BLACK_HOLE_DESPAWN_CHANCE_PER_SECOND,
                frame_time: BLACK_HOLE_FRAME_TIME,
            },
            RenderLayers::layer(BG_RENDER_LAYER),
        ))
        .insert(SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load("textures/distractions/blackhole_atlas.png"),
                vec2(BLACK_HOLE_SPRITE_SIZE, BLACK_HOLE_SPRITE_SIZE),
                BLACK_HOLE_ATLAS_FRAMES,
                1,
                None,
                None,
            )),
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
                RenderLayers::layer(BG_RENDER_LAYER),
                SpriteBundle {
                    texture: asset_server.load(
                        "textures/distractions/blackhole_flicker.png"
                            .to_string(),
                    ),
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
