//! Moves a sprite around with the mouse.
//! The position is always rounded to the nearest half a point.

use bevy::window::PrimaryWindow;
use common_visuals::camera::MainCamera;

use super::SceneMakerToolbar;
use crate::{prelude::*, scene_maker::LoadedFromSceneFile};

const HIGHLIGHT_COLOR: Color = Color::rgba(0.25, 0.25, 0.25, 0.8);

pub(super) enum MoveSprite {
    Highlighted {
        entity: Entity,
        og_color: Color,
    },
    Selected {
        entity: Entity,
        offset: Vec2,
        og_transform: Transform,
        og_color: Color,
    },
}

/// 1. Click on a sprite to select it (left button just pressed)
/// 2. Hold the mouse and move it to position the sprite (left button pressed)
/// 3. Release the mouse to place the sprite (left button just released)
/// 4. You can press Esc to cancel the movement.
/// 5. If no action is pressed, simply highlight the sprite closest to the
///    cursor.
pub fn system(
    mut toolbar: ResMut<SceneMakerToolbar>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    images: Res<Assets<Image>>,
    atlas_layouts: Res<Assets<TextureAtlasLayout>>,

    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut sprites: Query<
        (
            Entity,
            &mut Sprite,
            &GlobalTransform, // compare with cursor position
            &mut Transform,   // update when moving around
            &Handle<Image>,   // we need to know how big the sprite is
            &ViewVisibility,  // ignore hidden sprites
            Option<&TextureAtlas>, // if texture atlas, get single tile size
        ),
        With<LoadedFromSceneFile>,
    >,
) {
    if !toolbar.is_active {
        return;
    }

    let calc_cursor_pos = || {
        let cursor_pos = windows.single().cursor_position()?;
        let (camera, camera_transform) = camera.single();
        camera.viewport_to_world_2d(camera_transform, cursor_pos)
    };

    let entity_candidate = |cursor_pos| {
        sprites
            .iter()
            .filter(|(_, _, _, _, _, visibility, _)| visibility.get())
            .filter_map(
                |(entity, sprite, gtransform, _, texture, _, texture_atlas)| {
                    let size = if let Some(TextureAtlas { layout, .. }) =
                        texture_atlas
                    {
                        atlas_layouts.get(layout)?.textures.get(0)?.size()
                    } else {
                        images.get(texture)?.size().as_vec2()
                    };

                    let center = gtransform.translation().truncate()
                        - sprite.anchor.as_vec() * size;
                    let sprite_area = Rect::from_center_size(center, size);

                    sprite_area
                        .contains(cursor_pos)
                        .then(|| (entity, size.x * size.y))
                },
            )
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(entity, _)| entity)
    };

    let should_select_sprite = mouse.just_pressed(MouseButton::Left); // 1.
    let should_move_sprite =
        mouse.pressed(MouseButton::Left) && !should_select_sprite; // 2.
    let should_place_sprite = mouse.just_released(MouseButton::Left); // 3.
    let should_cancel_move =
        keyboard.just_pressed(KeyCode::Escape) && should_move_sprite; // 4.

    if should_place_sprite
        && let Some(MoveSprite::Selected {
            entity, og_color, ..
        }) = toolbar.selected_sprite.as_ref()
    {
        //
        // 3.
        //
        trace!("Placing sprite");

        let (_, mut sprite, ..) = sprites.get_mut(*entity).unwrap();
        sprite.color = *og_color;

        toolbar.selected_sprite = None;
    } else if should_select_sprite {
        //
        // 1.
        //
        trace!("Selecting sprite");

        let Some(cursor_pos) = calc_cursor_pos() else {
            return;
        };
        let Some(entity) = entity_candidate(cursor_pos) else {
            return;
        };

        match toolbar.selected_sprite.take() {
            Some(MoveSprite::Highlighted {
                entity: prev_entity,
                og_color,
            }) => {
                let (_, mut sprite, ..) = sprites.get_mut(prev_entity).unwrap();
                sprite.color = og_color;
            }
            Some(MoveSprite::Selected {
                entity: prev_entity,
                og_color,
                og_transform,
                ..
            }) => {
                let (_, mut sprite, _, mut current_transform, ..) =
                    sprites.get_mut(prev_entity).unwrap();
                *current_transform = og_transform;
                sprite.color = og_color;
            }
            None => {}
        }

        let (_, mut sprite, _, transform, ..) =
            sprites.get_mut(entity).unwrap();
        let offset = cursor_pos - transform.translation.truncate();
        let og_color = sprite.color;
        let og_transform = *transform;

        sprite.color = HIGHLIGHT_COLOR;
        toolbar.selected_sprite = Some(MoveSprite::Selected {
            entity,
            offset,
            og_transform,
            og_color,
        });
    } else if should_cancel_move
        && let Some(MoveSprite::Selected {
            entity,
            og_color,
            og_transform,
            ..
        }) = toolbar.selected_sprite.as_ref()
    {
        //
        // 4.
        //
        trace!("Cancelling move");

        let (_, mut sprite, _, mut current_transform, ..) =
            sprites.get_mut(*entity).unwrap();
        *current_transform = *og_transform;
        sprite.color = *og_color;

        toolbar.selected_sprite = None;
    } else if should_move_sprite
        && let Some(MoveSprite::Selected { entity, offset, .. }) =
            toolbar.selected_sprite.as_ref()
    {
        //
        // 2.
        //
        let Some(cursor_pos) = calc_cursor_pos() else {
            warn!("Cursor position not found when moving sprite");
            return;
        };

        let (_, _, _, mut transform, ..) = sprites.get_mut(*entity).unwrap();
        let new_pos = cursor_pos - *offset;
        transform.translation = Vec3::new(
            (new_pos.x * 2.0).round() / 2.0,
            (new_pos.y * 2.0).round() / 2.0,
            transform.translation.z,
        );
    } else {
        //
        // 5.
        //

        let Some(cursor_pos) = calc_cursor_pos() else {
            return;
        };

        let closest_entity = entity_candidate(cursor_pos);

        match (closest_entity, toolbar.selected_sprite.take()) {
            // closest entity different than the one highlighted
            (
                Some(closest_entity),
                Some(MoveSprite::Highlighted {
                    entity: prev_entity,
                    og_color,
                }),
            ) if closest_entity != prev_entity => {
                // cancel highlight on previous entity
                let (_, mut sprite, ..) = sprites.get_mut(prev_entity).unwrap();
                sprite.color = og_color;

                // apply highlight to closest entity
                let (_, mut sprite, ..) =
                    sprites.get_mut(closest_entity).unwrap();
                let og_color = sprite.color;
                sprite.color = HIGHLIGHT_COLOR;
                toolbar.selected_sprite = Some(MoveSprite::Highlighted {
                    entity: closest_entity,
                    og_color,
                });
            }
            // closest entity already highlighted or
            (Some(_), opt @ Some(_)) => {
                toolbar.selected_sprite = opt;
            }
            // nothing to do
            (None, None) => {}
            // closest entity not highlighted
            (Some(closest_entity), None) => {
                let (_, mut sprite, ..) =
                    sprites.get_mut(closest_entity).unwrap();
                let og_color = sprite.color;
                sprite.color = HIGHLIGHT_COLOR;
                toolbar.selected_sprite = Some(MoveSprite::Highlighted {
                    entity: closest_entity,
                    og_color,
                });
            }
            // no closest entity, cancel highlight
            (None, Some(MoveSprite::Highlighted { entity, og_color })) => {
                let (_, mut sprite, ..) = sprites.get_mut(entity).unwrap();
                sprite.color = og_color;
            }
            (_, Some(MoveSprite::Selected { .. })) => unreachable!(),
        }
    }
}
