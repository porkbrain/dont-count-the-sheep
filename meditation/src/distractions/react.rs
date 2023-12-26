use bevy::render::view::RenderLayers;
use bevy_magic_light_2d::gi::types::OmniLightSource2D;

use super::{
    consts::*, Distraction, DistractionDestroyedEvent, DistractionEntity,
    DistractionOccluder,
};
use crate::cameras::OBJ_RENDER_LAYER;
use crate::{
    climate::Climate,
    prelude::*,
    weather::{self, Weather},
};

/// If weather is very close and does special, the distraction is destroyed.
pub(super) fn to_weather_special(
    mut score: EventWriter<DistractionDestroyedEvent>,
    mut weather_actions: EventReader<weather::ActionEvent>,
    weather: Query<&Transform, (With<Weather>, Without<Distraction>)>,
    distractions: Query<(Entity, &Distraction, &Transform), Without<Weather>>,
    mut commands: Commands,
) {
    // it's possible that the game is paused the same frame as the event being
    // emitted, but that's so unlikely that we don't care
    let fired_special = weather_actions
        .read()
        .any(|a| matches!(a, weather::ActionEvent::FiredSpecial));
    if !fired_special {
        return;
    }

    let Ok(weather_transform) = weather.get_single() else {
        return;
    };

    let weather_translation = weather_transform.translation.truncate();

    for (entity, distraction, transform) in distractions.iter() {
        let translation = transform.translation.truncate();
        let distance_to_weather = translation.distance(weather_translation);

        if distance_to_weather <= WEATHER_SPECIAL_HITBOX_RADIUS {
            debug!("Distraction destroy by special event sent");
            score.send(DistractionDestroyedEvent {
                video: distraction.video,
                by_special: true,
                at_translation: translation,
            });
            commands.entity(entity).despawn_recursive();

            // ... go to next, can destroy multiple distractions per special
        }
    }
}

/// For each distraction:
/// 1. Check whether light is being cast on it (depends on rays from climate and
///    weather proximity)
/// 2. If it is, increase push back on distraction's occluder otherwise decrease
///    it
/// 3. If light is being cast on the distraction by climate, roll a dice to
///    crack it which adds a crack sprite to the distraction
/// 4. Remember number of cracks and if more than limit, destroy the distraction
/// 5. Find the line between climate and the distraction and place the occluder
///    on that line. Distance from center being the distraction's distance plus
///    the push back.
pub(super) fn to_environment(
    mut climate: Query<
        (&Climate, &Transform, &mut OmniLightSource2D),
        (
            Without<Weather>,
            Without<Distraction>,
            Without<DistractionOccluder>,
        ),
    >,
    weather: Query<
        &Transform,
        (
            Without<Climate>,
            With<Weather>,
            Without<Distraction>,
            Without<DistractionOccluder>,
        ),
    >,
    mut distraction_occluders: Query<
        (&Parent, &mut Transform),
        (
            Without<Climate>,
            Without<Weather>,
            Without<Distraction>,
            With<DistractionOccluder>,
        ),
    >,
    mut distractions: Query<
        (Entity, &Distraction, &Transform, &mut TextureAtlasSprite),
        (
            Without<Climate>,
            Without<Weather>,
            Without<DistractionOccluder>,
        ),
    >,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut score: EventWriter<DistractionDestroyedEvent>,
    mut commands: Commands,
) {
    let Ok(weather) = weather.get_single() else {
        return;
    };
    let Ok((climate, climate_transform, mut climate_light)) =
        climate.get_single_mut()
    else {
        return;
    };

    for (distraction_id, mut occluder_pos) in distraction_occluders.iter_mut() {
        let (distraction_entity, distraction, distraction_pos, mut sprite) =
            distractions
                .get_mut(distraction_id.get())
                .expect("Each occluder should have a distraction parent");

        //
        // 1.
        //

        // between [0; 1], increases as weather gets closer to distraction
        let weather_ray_bath = {
            let d = weather.translation.distance(distraction_pos.translation);

            1.0 / (d / HALF_OF_WEATHER_PUSH_BACK_FORCE_AT_DISTANCE + 1.0)
        };

        // between [0; 1], how much is the distraction being lit by the climate
        let climate_ray_bath = climate.ray_bath(
            climate_transform.translation.truncate(),
            distraction_pos.translation.truncate(),
        );

        //
        // 2.
        //

        // positive if pushed away from climate
        let push_back_force = PUSH_BACK_FORCE_AT_REST
            + weather_ray_bath * PUSH_BACK_FORCE_WEATHER_DISTANCE
            + climate_ray_bath * PUSH_BACK_FORCE_FULLY_CASTED_IN_CLIMATE_RAYS;

        //
        // 3.
        //

        let crack_chance = CRACK_CHANCE_PER_SECOND * time.delta_seconds();

        // TODO: balance, more predictable, clamp the time
        let should_crack =
            push_back_force > 45.0 && rand::random::<f32>() < crack_chance;

        let is_on_last_crack = sprite.index == MAX_CRACKS - 1;
        if should_crack && !is_on_last_crack {
            sprite.index += 1;

            let is_on_second_to_last_crack = sprite.index == MAX_CRACKS - 2;

            if is_on_second_to_last_crack {
                let first_frame = 0;

                let static_entity = commands
                    .spawn((
                        DistractionEntity,
                        Animation {
                            on_last_frame: AnimationEnd::Loop,
                            first: first_frame,
                            last: STATIC_ATLAS_FRAMES - 1,
                        },
                        AnimationTimer::new(
                            STATIC_ATLAS_FRAME_TIME,
                            TimerMode::Repeating,
                        ),
                        RenderLayers::layer(OBJ_RENDER_LAYER),
                    ))
                    .insert(SpriteSheetBundle {
                        texture_atlas: texture_atlases.add(
                            TextureAtlas::from_grid(
                                asset_server.load(assets::TV_STATIC_ATLAS),
                                vec2(
                                    DISTRACTION_SPRITE_SIZE,
                                    DISTRACTION_SPRITE_SIZE,
                                ),
                                STATIC_ATLAS_FRAMES,
                                1,
                                None,
                                None,
                            ),
                        ),
                        sprite: TextureAtlasSprite::new(first_frame),
                        transform: Transform::from_translation(
                            vec2(0.0, 0.0).extend(zindex::DISTRACTION_STATIC),
                        ),
                        ..default()
                    })
                    .id();

                commands.entity(distraction_entity).add_child(static_entity);
            }
        } else if should_crack && is_on_last_crack {
            //
            // 4.
            //

            debug!("Distraction destroy event sent");
            score.send(DistractionDestroyedEvent {
                video: distraction.video,
                by_special: false,
                at_translation: distraction_pos.translation.truncate(),
            });

            commands.entity(distraction_entity).despawn_recursive();
        }

        //
        // 5.
        //

        // On a line between climate and distraction, pushed back behind the
        // distraction by push_back_force.
        //
        // Our fork of the lighting dependency uses global transform instead of
        // transform, so the translation is relative to the entity.
        occluder_pos.translation = (distraction_pos.translation
            - climate_transform.translation)
            .normalize()
            * push_back_force;
    }

    // increase jitter intensity as more distractions are spawned
    climate_light.jitter_intensity =
        (distractions.iter().len() as f32 / 5.0).min(1.0);
}
