use bevy::sprite::Anchor;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Asset, TypePath, Serialize, Deserialize)]
pub(super) struct SceneSerde {
    sprites: Vec<SceneSpriteSerde>,
}

/// This is a single entity stored in the scene file.
/// Scene file is really just a collection of these entities.
#[derive(Serialize, Deserialize)]
struct SceneSpriteSerde {
    /// This will be inserted as a component to the entity.
    /// Changes to this component will be reflected in the entity in-game.
    #[serde(flatten)]
    reactive_component: SceneSprite,
    /// Used to construct the entity's components on deserialization.
    /// On serialization we collect the properties from the entity and store
    /// them here.
    #[serde(flatten)]
    serde_only: SceneSpriteSerdeOnly,
}

/// Used only for serialization and deserialization.
/// It cannot be updated in-game.
/// When the entity is stored, the properties are constructed from relevant
/// components.
#[derive(Serialize, Deserialize)]
struct SceneSpriteSerdeOnly {
    /// [`Name`]
    name: String,
    /// [`Transform`]
    initial_position: Vec2,
    /// [`Sprite`]
    anchor: AnchorSerde,
    /// [`Sprite`]
    color: Color,
}

#[derive(Component, Reflect, Default, Serialize, Deserialize, Clone)]
#[reflect(Component, Default)]
pub(super) struct SceneSprite {
    // TODO: debug assert being loaded
    asset_path: String,
    /// If not specified, it's calculated from position.
    overwrite_zindex: Option<f32>,
    /// If provided, the zindex will be calculated as if the y position was
    /// offset by this value.
    calc_zindex_as_if_y_was_offset_by: f32,
    /// If the texture is a sprite atlas, this will be used to calculate the
    /// sprite's position in the atlas.
    atlas: Option<SceneSpriteAtlas>,
}

#[derive(Reflect, Default, Serialize, Deserialize, Clone)]
#[reflect(Default)]
pub(super) struct SceneSpriteAtlas {
    index: usize,
    tile_size: Vec2,
    rows: usize,
    columns: usize,
    padding: Vec2,
    offset: Vec2,
}

/// Serializable and deserializable version of `Anchor`.
#[derive(Reflect, Default, Serialize, Deserialize)]
enum AnchorSerde {
    /// `Vec2::ZERO`
    #[default]
    Center,
    /// `Vec2::new(-0.5, -0.5)`
    BottomLeft,
    /// `Vec2::new(0.0, -0.5)`
    BottomCenter,
    /// `Vec2::new(0.5, -0.5)`
    BottomRight,
    /// `Vec2::new(-0.5, 0.0)`
    CenterLeft,
    /// `Vec2::new(0.5, 0.0)`
    CenterRight,
    /// `Vec2::new(-0.5, 0.5)`
    TopLeft,
    /// `Vec2::new(0.0, 0.5)`
    TopCenter,
    /// `Vec2::new(0.5, 0.5)`
    TopRight,
    /// Custom anchor point. Top left is `(-0.5, 0.5)`, center is `(0.0,
    /// 0.0)`. The value will be scaled with the sprite size.
    Custom(Vec2),
}

pub(super) fn load_scene(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut scenes: ResMut<Assets<SceneSerde>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let handle: Handle<SceneSerde> = asset_server.load("path/to/asset");
    if !asset_server.is_loaded_with_dependencies(&handle) {
        return;
    }

    let Some(scene) = scenes.remove(handle) else {
        return;
    };

    for sprite in scene.sprites {
        load_sprite(&mut cmd, &asset_server, &mut atlas_layouts, sprite);
    }
}

fn load_sprite(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    sprite: SceneSpriteSerde,
) {
    let SceneSpriteSerdeOnly {
        name,
        initial_position,
        anchor,
        color,
    } = sprite.serde_only;

    let transform = Transform::from_translation(Vec3::new(
        initial_position.x,
        initial_position.y,
        if let Some(z) = sprite.reactive_component.overwrite_zindex {
            z
        } else {
            todo!("calc zindex");
        },
    ));

    let mut root = cmd.spawn_empty();

    root.insert(Name::new(name));
    root.insert(SpatialBundle {
        transform,
        ..default()
    });
    root.insert(Sprite {
        color,
        anchor: anchor.into(),
        ..default()
    });
    root.insert(
        asset_server.load::<Image>(&sprite.reactive_component.asset_path),
    );
    if let Some(atlas) = sprite.reactive_component.atlas.as_ref() {
        root.insert(TextureAtlas {
            layout: atlas_layouts.add(TextureAtlasLayout::from_grid(
                atlas.tile_size,
                atlas.columns,
                atlas.rows,
                Some(atlas.padding),
                Some(atlas.offset),
            )),
            index: atlas.index,
        });
    }
    root.insert(sprite.reactive_component);
}

#[cfg(feature = "devtools")]
pub(super) fn react_to_changes(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,

    mut sprites: Query<
        (
            Entity,
            &SceneSprite,
            &mut Transform,
            &mut Handle<Image>,
            Option<&mut TextureAtlas>,
        ),
        Changed<SceneSprite>,
    >,
) {
    for (entity, truth, mut transform, mut texture, atlas) in sprites.iter_mut()
    {
        *texture = asset_server.load(&truth.asset_path);

        transform.translation.z = if let Some(z) = truth.overwrite_zindex {
            z
        } else {
            todo!("calc zindex");
        };

        match (&truth.atlas, atlas.is_some()) {
            // just straight up replace the atlas
            (Some(atlas), _) => {
                cmd.entity(entity).insert(TextureAtlas {
                    layout: atlas_layouts.add(TextureAtlasLayout::from_grid(
                        atlas.tile_size,
                        atlas.columns,
                        atlas.rows,
                        Some(atlas.padding),
                        Some(atlas.offset),
                    )),
                    index: atlas.index,
                });
            }
            // remove atlas because it's been set to None
            (None, true) => {
                cmd.entity(entity).remove::<TextureAtlas>();
            }
            // nothing to do
            (None, false) => {}
        }
    }
}

#[cfg(feature = "devtools")]
pub(super) fn store(
    sprites: Query<(&SceneSprite, &Name, &Transform, &Sprite)>,
) {
    let sprites =
        sprites
            .iter()
            .map(|(scene_sprite, name, transform, sprite)| SceneSpriteSerde {
                reactive_component: scene_sprite.clone(),
                serde_only: SceneSpriteSerdeOnly {
                    name: name.to_string(),
                    initial_position: transform.translation.truncate(),
                    anchor: sprite.anchor.into(),
                    color: sprite.color,
                },
            });

    let scene = SceneSerde {
        sprites: sprites.collect(),
    };

    let to_save = ron::ser::to_string_pretty(&scene, default()).unwrap();
}

impl From<AnchorSerde> for Anchor {
    fn from(anchor: AnchorSerde) -> Self {
        match anchor {
            AnchorSerde::Center => Self::Center,
            AnchorSerde::BottomLeft => Self::BottomLeft,
            AnchorSerde::BottomCenter => Self::BottomCenter,
            AnchorSerde::BottomRight => Self::BottomRight,
            AnchorSerde::CenterLeft => Self::CenterLeft,
            AnchorSerde::CenterRight => Self::CenterRight,
            AnchorSerde::TopLeft => Self::TopLeft,
            AnchorSerde::TopCenter => Self::TopCenter,
            AnchorSerde::TopRight => Self::TopRight,
            AnchorSerde::Custom(point) => Self::Custom(point),
        }
    }
}

impl From<Anchor> for AnchorSerde {
    fn from(anchor: Anchor) -> Self {
        match anchor {
            Anchor::Center => Self::Center,
            Anchor::BottomLeft => Self::BottomLeft,
            Anchor::BottomCenter => Self::BottomCenter,
            Anchor::BottomRight => Self::BottomRight,
            Anchor::CenterLeft => Self::CenterLeft,
            Anchor::CenterRight => Self::CenterRight,
            Anchor::TopLeft => Self::TopLeft,
            Anchor::TopCenter => Self::TopCenter,
            Anchor::TopRight => Self::TopRight,
            Anchor::Custom(point) => Self::Custom(point),
        }
    }
}
