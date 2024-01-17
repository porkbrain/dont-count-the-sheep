//! Player and NPC actor types.

pub mod npc;
pub mod player;

use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch};
use bevy_grid_squared::{direction::Direction as GridDirection, Square};
use common_story::Character;

use crate::layout::IntoMap;

/// Does not add any systems, only registers types.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Actor>().register_type::<ActorTarget>();
    }
}

/// Entity with this component can be moved around.
#[derive(Component, Reflect)]
pub struct Actor {
    /// What's the character type that's being represented.
    pub character: Character,
    /// How fast we move from square to square.
    pub step_time: Duration,
    /// If no target then this is the current position.
    /// If there's a target, current position is interpolated between this and
    /// the target.
    pub walking_from: Square,
    /// Target to walk towards.
    pub walking_to: Option<ActorTarget>,
    /// Used for animations.
    pub direction: GridDirection,
}

/// Target for an actor to walk towards.
#[derive(Reflect)]
pub struct ActorTarget {
    /// The target square actor walks to.
    pub square: Square,
    /// How long we've been walking towards the target.
    pub since: Stopwatch,
    /// Once the current target is reached, we can plan the next one.
    pub planned: Option<(Square, GridDirection)>,
}

impl Actor {
    /// Get the current square.
    pub fn current_square(&self) -> Square {
        self.walking_to
            .as_ref()
            .map(|to| to.square)
            .unwrap_or(self.walking_from)
    }
}

impl ActorTarget {
    /// Create a new target.
    pub fn new(square: Square) -> Self {
        Self {
            square,
            since: Stopwatch::new(),
            planned: None,
        }
    }
}

/// Transform is used to change z index based on y.
pub fn animate_movement<T: IntoMap>(
    mut character: Query<(&mut Actor, &mut TextureAtlasSprite)>,
    mut transform: Query<&mut Transform, With<Actor>>,
    time: Res<Time>,
) {
    use GridDirection::*;

    let Ok((mut character, mut sprite)) = character.get_single_mut() else {
        return;
    };

    let current_direction = character.direction;
    let step_time = character.step_time;
    let standing_still_sprite_index = match current_direction {
        Bottom => 0,
        Top => 1,
        Right | TopRight | BottomRight => 6,
        Left | TopLeft | BottomLeft => 9,
    };

    let Some(walking_to) = character.walking_to.as_mut() else {
        sprite.index = standing_still_sprite_index;

        return;
    };

    walking_to.since.tick(time.delta());

    let lerp_factor = walking_to.since.elapsed_secs()
        / if let Top | Bottom | Left | Right = current_direction {
            step_time.as_secs_f32()
        } else {
            // we need to walk a bit slower when walking diagonally because
            // we cover more distance
            step_time.as_secs_f32() * 2.0f32.sqrt()
        };

    let mut transform = transform.single_mut();
    let to = T::layout().square_to_world_pos(walking_to.square);

    if lerp_factor >= 1.0 {
        let new_from = walking_to.square;

        transform.translation = T::extend_z(to);

        if let Some((new_square, new_direction)) = walking_to.planned.take() {
            walking_to.since.reset();
            walking_to.square = new_square;
            character.direction = new_direction;
        } else {
            sprite.index = standing_still_sprite_index;

            character.walking_to = None;
        }

        character.walking_from = new_from;
    } else {
        let animation_step_time =
            animation_step_secs(step_time.as_secs_f32(), current_direction);
        let extra =
            (time.elapsed_seconds() / animation_step_time).floor() as usize % 2;

        sprite.index = match current_direction {
            Top => 2 + extra,
            Bottom => 4 + extra,
            Right | TopRight | BottomRight => 7 + extra,
            Left | TopLeft | BottomLeft => 10 + extra,
        };

        let from = T::layout().square_to_world_pos(character.walking_from);

        transform.translation = T::extend_z(from.lerp(to, lerp_factor));
    }
}

/// How often we change walking frame based on how fast we're walking from
/// square to square.
fn animation_step_secs(step_secs: f32, dir: GridDirection) -> f32 {
    match dir {
        GridDirection::Top | GridDirection::Bottom => step_secs * 5.0,
        _ => step_secs * 3.5,
    }
    .clamp(0.1, 0.5)
}

/// Helps setup a character bundle.
pub struct CharacterBundleBuilder {
    character: common_story::Character,
    initial_position: Vec2,
    initial_direction: GridDirection,
    walking_to: Option<ActorTarget>,
    initial_step_time: Option<Duration>,
}

impl From<common_story::Character> for CharacterBundleBuilder {
    fn from(character: common_story::Character) -> Self {
        Self::new(character)
    }
}

/// Extension trait for [`common_story::Character`].
pub trait CharacterExt {
    /// Returns a bundle builder for the character.
    fn bundle_builder(self) -> CharacterBundleBuilder;
}

impl CharacterExt for common_story::Character {
    fn bundle_builder(self) -> CharacterBundleBuilder {
        CharacterBundleBuilder::new(self)
    }
}

impl CharacterBundleBuilder {
    /// For which character to build the bundle.
    pub fn new(character: common_story::Character) -> Self {
        Self {
            character,
            initial_direction: GridDirection::Bottom,
            initial_position: default(),
            walking_to: default(),
            initial_step_time: default(),
        }
    }

    /// Where to spawn the character.
    /// Converted into the square by `IntoMap::world_pos_to_square` (see the
    /// `common_layout` crate).
    /// The specific layout is provided in the [`CharacterBundleBuilder::build`]
    /// method's `T`.
    pub fn with_initial_position(mut self, initial_position: Vec2) -> Self {
        self.initial_position = initial_position;

        self
    }

    /// When the map is loaded, the character is spawned facing this
    /// direction.
    pub fn with_initial_direction(
        mut self,
        initial_direction: GridDirection,
    ) -> Self {
        self.initial_direction = initial_direction;

        self
    }

    /// Where to walk to initially.
    pub fn with_walking_to(mut self, walking_to: Option<ActorTarget>) -> Self {
        self.walking_to = walking_to;

        self
    }

    /// How long does it take to move one square.
    pub fn with_initial_step_time(
        mut self,
        step_time: Option<Duration>,
    ) -> Self {
        self.initial_step_time = step_time;

        self
    }

    /// Returns a bundle that can be spawned.
    /// The bundle includes:
    /// - [`Name`] component with the character's name
    /// - [`Actor`] component with the character's movement data
    /// - [`SpriteSheetBundle`] with the character's sprite atlas
    pub fn build<T: IntoMap>(self) -> impl Bundle {
        let CharacterBundleBuilder {
            character,
            initial_position,
            initial_direction,
            walking_to,
            initial_step_time: step_time,
        } = self;

        let step_time = step_time.unwrap_or(character.default_step_time());

        (
            Name::from(common_story::Character::Winnie.name()),
            Actor {
                character,
                step_time,
                direction: initial_direction,
                walking_from: T::layout().world_pos_to_square(initial_position),
                walking_to,
            },
            SpriteSheetBundle {
                texture_atlas: character.sprite_atlas_handle(),
                sprite: TextureAtlasSprite {
                    anchor: bevy::sprite::Anchor::BottomCenter,
                    index: 0,
                    ..default()
                },
                transform: Transform::from_translation(T::extend_z(
                    initial_position,
                )),
                ..default()
            },
        )
    }
}
