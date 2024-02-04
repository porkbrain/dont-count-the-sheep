//! Player and NPC actor types.

pub mod npc;
pub mod player;

use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch, utils::HashMap};
use bevy_grid_squared::{GridDirection, Square};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use common_story::Character;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{
    layout::{IntoMap, Tile},
    Player, TileKind, TileMap,
};

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

/// Maps actors to zones they currently occupy.
/// Each actor can be in multiple zones at once.
///
/// Only those tiles that are zones as returned by `TileKind::is_zone` are
/// stored.
#[derive(
    Resource, Serialize, Deserialize, Reflect, InspectorOptions, Default,
)]
#[reflect(Resource, InspectorOptions)]
pub struct ActorZoneMap<L: Default> {
    map: HashMap<Entity, SmallVec<[TileKind<L>; 3]>>,
}

/// Some useful events for actors.
#[derive(Event, Reflect)]
pub enum ActorMovementEvent<T> {
    /// Is emitted when an [`Actor`] enters a zone.
    ZoneEntered {
        /// The zone that was entered.
        zone: TileKind<T>,
        /// The actor that entered the zone.
        who: Who,
    },
    /// Is emitted when an [`Actor`] leaves a zone.
    ZoneLeft {
        /// The zone that was left.
        zone: TileKind<T>,
        /// The actor that left the zone.
        who: Who,
    },
}

/// Identifies an actor in the [`ActorMovementEvent`].
#[derive(Reflect)]
pub struct Who {
    /// Is the actor a player?
    /// Otherwise an NPC.
    pub is_player: bool,
    /// The entity that entered the zone.
    pub entity: Entity,
    /// The character that entered the zone.
    pub character: Character,
    /// Where was the actor at the moment when the event was sent.
    pub at: Square,
}

/// Helps setup a character bundle.
pub struct CharacterBundleBuilder {
    character: common_story::Character,
    initial_position: Vec2,
    initial_direction: GridDirection,
    walking_to: Option<ActorTarget>,
    initial_step_time: Option<Duration>,
    color: Option<Color>,
}

/// Sends events when an actor does something interesting.
/// This system is registered on call to [`crate::layout::register`].
///
/// If you listen to this event then condition your system to run on
/// `run_if(event_update_condition::<ActorMovementEvent>)` and
/// `after(actor::emit_movement_events::<T>)`.
pub fn emit_movement_events<T: IntoMap>(
    tilemap: Res<TileMap<T>>,
    mut actor_zone_map: ResMut<ActorZoneMap<T::LocalTileKind>>,
    mut event: EventWriter<ActorMovementEvent<T::LocalTileKind>>,

    actors: Query<(Entity, &Actor, Option<&Player>), Changed<Transform>>,
) {
    for (entity, actor, player) in actors.iter() {
        let at = actor.current_square();

        let Some(tiles) = tilemap.get(&at) else {
            continue;
        };

        let active_zones = actor_zone_map.map.entry(entity).or_default();

        // remove zones that are no longer active and send an event
        active_zones.retain(|active| {
            if tiles.contains(&active) {
                return true;
            }

            trace!("Actor {:?} left zone {active:?}", actor.character);
            event.send(ActorMovementEvent::ZoneLeft {
                zone: *active,
                who: Who {
                    at,
                    is_player: player.is_some(),
                    entity,
                    character: actor.character,
                },
            });

            false
        });

        // add zones that are new and send an event
        for tile in tiles.iter().filter(|tile| tile.is_zone()) {
            if active_zones.contains(tile) {
                continue;
            }

            active_zones.push(*tile);

            trace!("Actor {:?} is in zone {tile:?}", actor.character);
            event.send(ActorMovementEvent::ZoneEntered {
                zone: *tile,
                who: Who {
                    at,
                    is_player: player.is_some(),
                    entity,
                    character: actor.character,
                },
            });
        }
    }
}

/// Transform is used to change z index based on y.
pub fn animate_movement<T: IntoMap>(
    time: Res<Time>,

    mut actors: Query<
        (Entity, &mut Actor, &mut TextureAtlasSprite),
        With<Transform>,
    >,
    // separate query so that we don't change transform unless needed
    mut transform: Query<&mut Transform, With<Actor>>,
) {
    use GridDirection::*;

    for (entity, mut actor, mut sprite) in actors.iter_mut() {
        let current_direction = actor.direction;
        let step_time = actor.step_time;
        let standing_still_sprite_index = match current_direction {
            Bottom => 0,
            Top => 1,
            Right | TopRight | BottomRight => 6,
            Left | TopLeft | BottomLeft => 9,
        };

        let Some(walking_to) = actor.walking_to.as_mut() else {
            sprite.index = standing_still_sprite_index;

            continue;
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

        // safe cuz <With<Transform>>
        let mut transform = transform.get_mut(entity).unwrap();
        let to = T::layout().square_to_world_pos(walking_to.square);

        if lerp_factor >= 1.0 {
            let new_from = walking_to.square;

            transform.translation = T::extend_z(to);

            if let Some((new_square, new_direction)) = walking_to.planned.take()
            {
                walking_to.since.reset();
                walking_to.square = new_square;
                actor.direction = new_direction;
            } else {
                sprite.index = standing_still_sprite_index;

                actor.walking_to = None;
            }

            actor.walking_from = new_from;
        } else {
            let animation_step_time =
                animation_step_secs(step_time.as_secs_f32(), current_direction);
            let extra = (time.elapsed_seconds() / animation_step_time).floor()
                as usize
                % 2;

            sprite.index = match current_direction {
                Top => 2 + extra,
                Bottom => 4 + extra,
                Right | TopRight | BottomRight => 7 + extra,
                Left | TopLeft | BottomLeft => 10 + extra,
            };

            let from = T::layout().square_to_world_pos(actor.walking_from);

            transform.translation = T::extend_z(from.lerp(to, lerp_factor));
        }
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

impl<T> ActorMovementEvent<T> {
    /// Whether the actor is a player.
    pub fn is_player(&self) -> bool {
        match self {
            Self::ZoneEntered { who, .. } | Self::ZoneLeft { who, .. } => {
                who.is_player
            }
        }
    }
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
            color: default(),
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

    /// Sets the color of the sprite.
    pub fn with_sprite_color(mut self, color: Option<Color>) -> Self {
        self.color = color;

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
            color,
        } = self;

        let step_time = step_time.unwrap_or(character.default_step_time());

        (
            Name::from(character.name()),
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
                    color: color.unwrap_or_default(),
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
