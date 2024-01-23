use bevy::render::view::RenderLayers;
use bevy_magic_light_2d::gi::types::OmniLightSource2D;
use common_visuals::camera::render_layer;
use main_game_lib::common_ext::QueryExt;

use super::{
    consts::*, effects::bolt::get_bundle_with_respect_to_origin_at_zero, Polpo,
    PolpoDestroyedEvent, PolpoOccluder,
};
use crate::{
    climate::Climate,
    hoshi::{self, Hoshi},
    prelude::*,
};

/// If Hoshi is very close and does special, the polpo is destroyed.
pub(super) fn to_hoshi_special(
    mut cmd: Commands,
    mut score: EventWriter<PolpoDestroyedEvent>,
    mut hoshi_actions: EventReader<hoshi::ActionEvent>,

    hoshi: Query<&Transform, (With<Hoshi>, Without<Polpo>)>,
    polpos: Query<(Entity, &Polpo, &Transform), Without<Hoshi>>,
) {
    // it's possible that the game is paused the same frame as the event being
    // emitted, but that's so unlikely that we don't care
    let fired_special = hoshi_actions
        .read()
        .any(|a| matches!(a, hoshi::ActionEvent::FiredSpecial));
    if !fired_special {
        return;
    }

    let Some(hoshi_transform) = hoshi.get_single_or_none() else {
        return;
    };

    let hoshi_translation = hoshi_transform.translation.truncate();

    for (entity, polpo, transform) in polpos.iter() {
        let translation = transform.translation.truncate();
        let distance_to_hoshi = translation.distance(hoshi_translation);

        if distance_to_hoshi <= HOSHI_SPECIAL_HITBOX_RADIUS {
            debug!("Polpo destroy by special event sent ({entity:?})");
            score.send(PolpoDestroyedEvent {
                video: polpo.video,
                by_special: true,
                at_translation: translation,
            });
            cmd.entity(entity).despawn_recursive();

            // ... go to next, can destroy multiple Polpos per special
        }
    }
}

/// For each Polpo:
/// 1. Check whether light is being cast on it (depends on rays from climate and
///    Hoshi proximity)
/// 2. If it is, increase push back on Polpo's occluder otherwise decrease it
/// 3. If light is being cast on the Polpo by climate, roll a dice to crack it
///    which adds a crack sprite to the Polpo
/// 4. Remember number of cracks and if more than limit, destroy the Polpo
/// 5. Find the line between climate and the Polpo and place the occluder on
///    that line. Distance from center being the Polpo's distance plus the push
///    back.
pub(super) fn to_environment(
    mut cmd: Commands,
    mut score: EventWriter<PolpoDestroyedEvent>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,

    mut climate: Query<
        (&Climate, &Transform, &mut OmniLightSource2D),
        (Without<Hoshi>, Without<Polpo>, Without<PolpoOccluder>),
    >,
    hoshi: Query<
        &Transform,
        (
            Without<Climate>,
            With<Hoshi>,
            Without<Polpo>,
            Without<PolpoOccluder>,
        ),
    >,
    mut polpo_occluders: Query<
        (&Parent, &mut Transform),
        (
            Without<Climate>,
            Without<Hoshi>,
            Without<Polpo>,
            With<PolpoOccluder>,
        ),
    >,
    mut polpos: Query<
        (Entity, &mut Polpo, &Transform, &mut TextureAtlasSprite),
        (Without<Climate>, Without<Hoshi>, Without<PolpoOccluder>),
    >,
) {
    let Some(hoshi) = hoshi.get_single_or_none() else {
        return;
    };
    let Some((climate, climate_transform, mut climate_light)) =
        climate.get_single_mut_or_none()
    else {
        return;
    };

    let push_back_force_fully_casted_in_climate_rays = climate
        .mode()
        .push_back_force_fully_casted_in_climate_rays();
    let max_force: f32 = PUSH_BACK_FORCE_AT_REST
        + PUSH_BACK_FORCE_HOSHI_DISTANCE
        + push_back_force_fully_casted_in_climate_rays;

    for (polpo_id, mut occluder_pos) in polpo_occluders.iter_mut() {
        let (polpo_entity, mut polpo, polpo_pos, mut sprite) = polpos
            .get_mut(polpo_id.get())
            .expect("Each occluder should have a Polpo parent");

        //
        // 1.
        //

        // between [0; 1], increases as Hoshi gets closer to Polpo
        let hoshi_ray_bath = {
            let d = hoshi.translation.distance(polpo_pos.translation);

            let max = NONE_OF_HOSHI_PUSH_BACK_FORCE_AT_DISTANCE;
            if d >= max {
                0.0
            } else {
                (-d + max).sqrt() / max.sqrt()
            }
        };
        let hoshi_push_back_force_contrib =
            hoshi_ray_bath * PUSH_BACK_FORCE_HOSHI_DISTANCE;

        // between [0; 1], how much is the Polpo being lit by the climate
        let climate_ray_bath = climate.ray_bath(
            climate_transform.translation.truncate(),
            polpo_pos.translation.truncate(),
        );
        let climate_push_back_force_contrib =
            climate_ray_bath * push_back_force_fully_casted_in_climate_rays;

        //
        // 2.
        //

        // positive if pushed away from climate
        let push_back_force_without_hoshi_contrib =
            PUSH_BACK_FORCE_AT_REST + climate_push_back_force_contrib;
        let push_back_force_with_hoshi_contrib =
            push_back_force_without_hoshi_contrib
                + hoshi_push_back_force_contrib;

        //
        // 3.
        //

        let dice_roll = rand::random::<f32>();

        let should_crack = |push_back_force: f32| {
            let crack_chance_per_second = 2.0 * push_back_force / max_force;
            let crack_chance = crack_chance_per_second * time.delta_seconds();

            dice_roll < crack_chance
        };

        let should_crack_with_hoshi_contrib =
            should_crack(push_back_force_with_hoshi_contrib);

        let is_on_last_crack = sprite.index == MAX_CRACKS - 1;
        if should_crack_with_hoshi_contrib && !is_on_last_crack {
            // no real Hoshi push back force was applied to tip the scales
            // in favor of cracking
            let would_ve_cracked_anyway =
                should_crack(push_back_force_without_hoshi_contrib);
            if !would_ve_cracked_anyway {
                // with respect to origin at zero
                let change_of_basis_from = hoshi.translation.truncate()
                    - polpo_pos.translation.truncate();

                let bolt_entity = cmd
                    .spawn(get_bundle_with_respect_to_origin_at_zero(
                        &asset_server,
                        change_of_basis_from,
                    ))
                    .id();
                cmd.entity(polpo_entity).add_child(bolt_entity);

                polpo.jitter += change_of_basis_from.abs().normalize();
            }

            sprite.index += 1;

            let is_on_second_to_last_crack = sprite.index == MAX_CRACKS - 2;

            if is_on_second_to_last_crack {
                let first_frame = 0;

                let static_entity = cmd
                    .spawn((
                        AtlasAnimation {
                            on_last_frame: AtlasAnimationEnd::Loop,
                            first: first_frame,
                            last: STATIC_ATLAS_FRAMES - 1,
                            ..default()
                        },
                        AtlasAnimationTimer::new(
                            STATIC_ATLAS_FRAME_TIME,
                            TimerMode::Repeating,
                        ),
                        RenderLayers::layer(render_layer::OBJ),
                    ))
                    .insert(SpriteSheetBundle {
                        texture_atlas: texture_atlases.add(
                            TextureAtlas::from_grid(
                                asset_server.load(assets::TV_STATIC_ATLAS),
                                vec2(POLPO_SPRITE_SIZE, POLPO_SPRITE_SIZE),
                                STATIC_ATLAS_FRAMES,
                                1,
                                None,
                                None,
                            ),
                        ),
                        sprite: TextureAtlasSprite::new(first_frame),
                        transform: Transform::from_translation(
                            vec2(0.0, 0.0).extend(zindex::POLPO_STATIC),
                        ),
                        ..default()
                    })
                    .id();

                cmd.entity(polpo_entity).add_child(static_entity);
            }
        } else if should_crack_with_hoshi_contrib && is_on_last_crack {
            //
            // 4.
            //

            debug!("Polpo destroy event sent ({polpo_entity:?})");
            score.send(PolpoDestroyedEvent {
                video: polpo.video,
                by_special: false,
                at_translation: polpo_pos.translation.truncate(),
            });
            cmd.entity(polpo_entity).despawn_recursive();
        }

        //
        // 5.
        //

        // On a line between climate and polpo, pushed back behind the
        // polpo by push_back_force.
        //
        // Our fork of the lighting dependency uses global transform instead of
        // transform, so the translation is relative to the entity.
        occluder_pos.translation =
            (polpo_pos.translation - climate_transform.translation).normalize()
                * push_back_force_with_hoshi_contrib;
    }

    // increase jitter intensity as more polpos are spawned
    climate_light.jitter_intensity =
        (polpos.iter().len() as f32 / 5.0).min(2.0);
}
