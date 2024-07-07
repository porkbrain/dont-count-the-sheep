use bevy::render::view::RenderLayers;
use common_visuals::camera::render_layer;
use main_game_lib::common_ext::QueryExt;

use super::{consts::*, Polpo, PolpoDestroyedEvent};
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

/// TODO: rework
pub(super) fn to_environment(
    mut cmd: Commands,
    mut score: EventWriter<PolpoDestroyedEvent>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,

    mut polpos: Query<
        (Entity, &mut Polpo, &Transform, &mut TextureAtlas),
        (Without<Climate>, Without<Hoshi>),
    >,
) {
    for (polpo_entity, polpo, polpo_pos, mut sprite) in polpos.iter_mut() {
        let dice_roll = rand::random::<f32>();

        let should_crack = {
            let crack_chance_per_second = 0.25;
            let crack_chance = crack_chance_per_second * time.delta_seconds();

            dice_roll < crack_chance
        };

        let is_on_last_crack = sprite.index == MAX_CRACKS - 1;
        if should_crack && !is_on_last_crack {
            sprite.index += 1;

            let is_on_second_to_last_crack = sprite.index == MAX_CRACKS - 2;

            if is_on_second_to_last_crack {
                let first_frame = 0;

                let static_entity = cmd
                    .spawn((
                        AtlasAnimation {
                            on_last_frame: AtlasAnimationEnd::LoopIndefinitely,
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
                    .insert(SpriteBundle {
                        texture: asset_server.load(assets::TV_STATIC_ATLAS),
                        transform: Transform::from_translation(
                            vec2(0.0, 0.0).extend(zindex::POLPO_STATIC),
                        ),
                        ..default()
                    })
                    .insert(TextureAtlas {
                        index: first_frame,
                        layout: texture_atlases.add(
                            TextureAtlasLayout::from_grid(
                                UVec2::splat(POLPO_SPRITE_SIZE as u32),
                                STATIC_ATLAS_FRAMES as u32,
                                1,
                                None,
                                None,
                            ),
                        ),
                    })
                    .id();

                cmd.entity(polpo_entity).add_child(static_entity);
            }
        } else if should_crack && is_on_last_crack {
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
    }
}
