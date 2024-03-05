use std::ops::RangeInclusive;

use bevy::{ecs::system::EntityCommands, sprite::Anchor};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Defines a scene.
pub trait SpriteScene: Send + Sync + 'static {
    /// Path to the .ron file with sprite scene data.
    fn asset_path() -> &'static str;

    /// The range of y coordinates on the map.
    fn y_range() -> RangeInclusive<f32>;

    /// Given a position on the map, add a z coordinate.
    /// Will return a z-coordinate in the range of -0.1 to 1.1.
    #[inline]
    fn extend_z(Vec2 { x, y }: Vec2) -> Vec3 {
        let (min, max) = Self::y_range().into_inner();
        let size = max - min;
        debug_assert!(size > 0.0, "{max} - {min} <= 0.0");

        // we allow for a tiny leeway for positions outside of the bounding box
        let z = ((max - y) / size).clamp(-0.1, 1.1);

        Vec3::new(x, y, z)
    }

    /// Given a position on the map, add a z coordinate as if the y coordinate
    /// was offset by `offset`.
    fn extend_z_with_y_offset(Vec2 { x, y }: Vec2, offset: f32) -> Vec3 {
        let z = Self::extend_z(Vec2 { x, y: y + offset }).z;
        Vec3::new(x, y, z)
    }
}

/// Used for loading of the scene.
#[derive(Component)]
pub struct SpriteSceneHandle<T> {
    /// The actual handle that can be used to load the file.
    pub handle: Handle<SceneSerde>,
    _phantom: std::marker::PhantomData<T>,
}

/// Stored in a .ron file.
#[derive(Asset, TypePath, Serialize, Deserialize, Debug)]
pub struct SceneSerde {
    sprites: Vec<SceneSpriteSerde>,
}

/// Any entity that's loaded from a config scene file will have this component.
#[derive(Component)]
pub struct LoadedFromSceneFile;

/// This is a single entity stored in the scene file.
/// Scene file is really just a collection of these entities.
#[derive(Serialize, Deserialize, Debug)]
struct SceneSpriteSerde {
    /// This will be inserted as a component to the entity.
    /// Changes to this component will be reflected in the entity in-game.
    reactive_component: SceneSpriteConfig,
    /// Used to construct the entity's components on deserialization.
    /// On serialization we collect the properties from the entity and store
    /// them here.
    serde_only: SceneSpriteSerdeOnly,
}

/// Used only for serialization and deserialization.
/// It cannot be updated in-game.
/// When the entity is stored, the properties are constructed from relevant
/// components.
#[derive(Serialize, Deserialize, Debug)]
struct SceneSpriteSerdeOnly {
    /// [`Name`]
    name: String,
    /// [`Transform`]
    #[serde(default)]
    initial_position: Vec2,
    /// [`Sprite`]
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    anchor: Option<AnchorSerde>,
    /// [`Sprite`]
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Color>,
}

/// This is only really useful for devtools.
#[derive(Component, Reflect, Debug, Default, Serialize, Deserialize, Clone)]
#[reflect(Component, Default)]
pub struct SceneSpriteConfig {
    // TODO: debug assert being loaded
    /// The path to the texture asset.
    pub asset_path: String,
    /// If not specified, it's calculated from position.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwrite_zindex: Option<f32>,
    /// If provided, the zindex will be calculated as if the y position was
    /// offset by this value.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calc_zindex_as_if_y_was_offset_by: Option<f32>,
    /// If the texture is a sprite atlas, this will be used to calculate the
    /// sprite's position in the atlas.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atlas: Option<SceneSpriteAtlas>,
}

/// This is only really useful for devtools.
#[derive(Reflect, Default, Serialize, Deserialize, Debug, Clone)]
#[reflect(Default)]
pub struct SceneSpriteAtlas {
    /// The index of the sprite in the atlas.
    #[serde(default)]
    pub index: usize,
    /// The size of a single tile in the atlas.
    pub tile_size: Vec2,
    /// The number of rows in the atlas.
    pub rows: usize,
    /// The number of columns in the atlas.
    pub columns: usize,
    /// Set to `Vec2::ZERO` for no padding.
    #[serde(default)]
    pub padding: Vec2,
    /// Set to `Vec2::ZERO` for no offset.
    #[serde(default)]
    pub offset: Vec2,
}

/// Serializable and deserializable version of `Anchor`.
#[derive(Reflect, Debug, Default, Serialize, Deserialize)]
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

impl SceneSerde {
    /// Spawns the next sprite in the scene.
    /// Returns the commands with which more components can be added to the
    /// entity.
    /// The [`Name`] component has already been inserted!
    /// It's returned so that it can be matched on.
    /// This is how you can determine what entity you're working with.
    pub fn spawn_next_sprite<'a, T: SpriteScene>(
        &mut self,
        cmd: &'a mut Commands,
        asset_server: &AssetServer,
        atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Option<(EntityCommands<'a>, Name)> {
        let (entity, name) = load_sprite::<T>(
            cmd,
            asset_server,
            atlas_layouts,
            self.sprites.pop()?,
        );

        Some((cmd.entity(entity), name))
    }
}

fn load_sprite<T: SpriteScene>(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    sprite: SceneSpriteSerde,
) -> (Entity, Name) {
    let SceneSpriteSerdeOnly {
        name,
        initial_position,
        anchor,
        color,
    } = sprite.serde_only;

    let translation =
        if let Some(z) = sprite.reactive_component.overwrite_zindex {
            initial_position.extend(z)
        } else {
            T::extend_z_with_y_offset(
                initial_position,
                sprite
                    .reactive_component
                    .calc_zindex_as_if_y_was_offset_by
                    .unwrap_or_default(),
            )
        };

    let transform = Transform::from_translation(translation);

    let mut root = cmd.spawn(LoadedFromSceneFile);
    let name = Name::new(name);

    root.insert(name.clone());
    root.insert(SpatialBundle {
        transform,
        ..default()
    });
    root.insert(Sprite {
        color: color.unwrap_or_default(),
        anchor: anchor.unwrap_or_default().into(),
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
    #[cfg(feature = "devtools")]
    {
        // the reactive component is used to update the entity in-game
        // conveniently, which is only used for scene making
        root.insert(sprite.reactive_component);
    }

    (root.id(), name)
}

#[cfg(feature = "devtools")]
pub(super) fn react_to_changes<T: SpriteScene>(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,

    mut sprites: Query<
        (
            Entity,
            &SceneSpriteConfig,
            &mut Transform,
            &mut Handle<Image>,
            Option<&mut TextureAtlas>,
        ),
        (Changed<SceneSpriteConfig>, With<LoadedFromSceneFile>),
    >,
) {
    for (entity, truth, mut transform, mut texture, atlas) in sprites.iter_mut()
    {
        *texture = asset_server.load(&truth.asset_path);

        transform.translation = if let Some(z) = truth.overwrite_zindex {
            transform.translation.truncate().extend(z)
        } else {
            T::extend_z_with_y_offset(
                transform.translation.truncate(),
                truth.calc_zindex_as_if_y_was_offset_by.unwrap_or_default(),
            )
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

// TODO: run this if UI button pressed
// also in the same UI we want to display form to add new sprite component
#[cfg(feature = "devtools")]
pub(super) fn store(
    sprites: Query<
        (&SceneSpriteConfig, &Name, &Transform, &Sprite),
        With<LoadedFromSceneFile>,
    >,
) {
    let sprites =
        sprites
            .iter()
            .map(|(scene_sprite, name, transform, sprite)| SceneSpriteSerde {
                reactive_component: scene_sprite.clone(),
                serde_only: SceneSpriteSerdeOnly {
                    name: name.to_string(),
                    initial_position: transform.translation.truncate(),
                    anchor: if sprite.anchor == default() {
                        None
                    } else {
                        Some(sprite.anchor.into())
                    },
                    color: if sprite.color == default() {
                        None
                    } else {
                        Some(sprite.color)
                    },
                },
            });

    let scene = SceneSerde {
        sprites: sprites.collect(),
    };

    let to_save = ron::ser::to_string_pretty(&scene, default()).unwrap();

    todo!("{to_save}");
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

impl<T> SpriteSceneHandle<T> {
    /// Creates a new handle.
    pub fn new(handle: Handle<SceneSerde>) -> Self {
        Self {
            handle,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> From<&SpriteSceneHandle<T>> for AssetId<SceneSerde> {
    fn from(SpriteSceneHandle { handle, .. }: &SpriteSceneHandle<T>) -> Self {
        handle.into()
    }
}

// #[cfg(test)]
// mod tests {
//     use ron::ser::PrettyConfig;

//     use super::*;

//     #[test]
//     fn xd() {
//         let with_atlas = SceneSpriteSerde {
//             reactive_component: SceneSpriteConfig {
//                 asset_path: "a".to_string(),
//                 overwrite_zindex: Some(0.0),
//                 calc_zindex_as_if_y_was_offset_by: Some(0.0),
//                 atlas: Some(SceneSpriteAtlas {
//                     index: 0,
//                     tile_size: Vec2::new(1.0, 1.0),
//                     rows: 1,
//                     columns: 1,
//                     padding: Vec2::ZERO,
//                     offset: Vec2::ZERO,
//                 }),
//             },
//             serde_only: SceneSpriteSerdeOnly {
//                 name: "a".to_string(),
//                 initial_position: Vec2::ZERO,
//                 anchor: None,
//                 color: None,
//             },
//         };

//         let raw = ron::ser::to_string_pretty(
//             &with_atlas,
//             PrettyConfig::default().struct_names(true),
//         )
//         .unwrap();

//         panic!("{:?}", raw);
//     }

//     #[test]
//     fn xdxd() {
//         let xd = r#"
//         (
//             sprites: [
//                 (
//                     reactive_component: (
//                         asset_path: "apartment/brown_light_door_atlas.png",
//                         calc_zindex_as_if_y_was_offset_by: Some(8.5),
//                     ),
//                     serde_only: (
//                         name: "Bedroom door",
//                         anchor: Some(BottomCenter),
//                         initial_position: (-105.0, -88.0),
//                         atlas: Some(SceneSpriteAtlas(
//                             index: 0,
//                             tile_size: (27.0, 53.0),
//                             rows: 1,
//                             columns: 2,
//                             padding: (1.0, 0.0),
//                             offset: (0.0, 0.0),
//                         ))
//                     )
//                 )
//             ],
//         )
//         "#;

//         let scene: SceneSerde = ron::de::from_str(xd).unwrap();

//         panic!("{:#?}", scene);
//     }
// }
