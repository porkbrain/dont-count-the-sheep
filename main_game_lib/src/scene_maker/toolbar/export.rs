use crate::prelude::*;

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
