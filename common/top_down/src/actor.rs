//! Player and NPC actor types.

pub mod npc;
pub mod player;

use std::{iter, time::Duration};

use bevy::{
    ecs::event::event_update_condition,
    prelude::*,
    time::Stopwatch,
    utils::{HashMap, HashSet},
};
use bevy_grid_squared::{sq, GridDirection, Square};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use common_story::Character;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::{
    layout::{IntoMap, Tile, TileIndex},
    Player, TileKind, TileMap,
};

/// Use with [`IntoSystemConfigs::run_if`] to run a system only when an actor
/// moves.
pub fn movement_event_emitted<T: IntoMap>(
) -> impl FnMut(Res<Events<ActorMovementEvent<T::LocalTileKind>>>) -> bool {
    event_update_condition::<ActorMovementEvent<T::LocalTileKind>>
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
    /// Squares that the actor currently occupies, along with the layer that
    /// the actor tile kind is assigned to.
    ///
    /// The number of squares ultimately depend on a map layout, specifically
    /// the size of a square. On more granular maps, the actor can occupy
    /// more squares.
    /// Also, different characters can occupy different squares.
    occupies: Vec<TileIndex>,
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
pub struct ActorZoneMap<L: Default + Eq + std::hash::Hash> {
    /// Set is used to avoid duplicates.
    /// Those could arise from a map that has the same zone multiple times in
    /// the same square (different layer.)
    map: HashMap<Entity, HashSet<TileKind<L>>>,
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
    /// TODO: also when despawned in a zone.
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
        let character = actor.character;

        let zone_left_event = |zone| ActorMovementEvent::ZoneLeft {
            zone,
            who: Who {
                at,
                is_player: player.is_some(),
                entity,
                character,
            },
        };

        let active_zones = actor_zone_map.map.entry(entity).or_default();

        let Some(tiles) = tilemap.get(&at) else {
            for active in active_zones.drain() {
                trace!("Actor {character:?} left zone {active:?}");
                event.send(zone_left_event(active));
            }

            continue;
        };

        // remove zones that are no longer active and send an event
        active_zones.retain(|active| {
            if tiles.contains(active) {
                return true;
            }

            trace!("Actor {character:?} left zone {active:?}");
            event.send(zone_left_event(*active));

            false
        });

        // add zones that are new and send an event
        for tile in tiles.iter().filter(|tile| tile.is_zone()) {
            if active_zones.contains(tile) {
                continue;
            }

            active_zones.insert(*tile);

            trace!("Actor {character:?} is in zone {tile:?}");
            event.send(ActorMovementEvent::ZoneEntered {
                zone: *tile,
                who: Who {
                    at,
                    is_player: player.is_some(),
                    entity,
                    character,
                },
            });
        }
    }
}

/// Actually moves the actors.
/// Other systems will only edit the `Actor` component to plan the movement.
///
/// The z is based off y.
/// See the [`IntoMap::extend_z`] for more info.
pub fn animate_movement<T: IntoMap>(
    time: Res<Time>,
    mut tilemap: ResMut<TileMap<T>>,

    mut actors: Query<(
        Entity,
        &mut Actor,
        &mut TextureAtlasSprite,
        &mut Transform,
    )>,
) {
    for (entity, mut actor, sprite, transform) in actors.iter_mut() {
        if actor.occupies.is_empty() {
            // One time initialization of the actor's tiles.
            // It's more convenient to do it here than during
            // the building process of the actor.
            tilemap.replace_actor_tiles(entity, &mut actor);
        }

        animate_movement_for_actor::<T>(
            &time,
            &mut tilemap,
            entity,
            &mut actor,
            sprite,
            transform,
        );
    }
}

fn animate_movement_for_actor<T: IntoMap>(
    time: &Time,
    tilemap: &mut TileMap<T>,
    entity: Entity,
    actor: &mut Actor,
    mut sprite: Mut<TextureAtlasSprite>,
    mut transform: Mut<Transform>,
) {
    use GridDirection::*;

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

    let to = T::layout().square_to_world_pos(walking_to.square);

    if lerp_factor >= 1.0 {
        let new_from = walking_to.square;

        transform.translation = T::extend_z(to);

        if let Some((new_square, new_direction)) = walking_to.planned.take() {
            walking_to.since.reset();
            walking_to.square = new_square;
            actor.direction = new_direction;
        } else {
            sprite.index = standing_still_sprite_index;

            actor.walking_to = None;
        }

        actor.walking_from = new_from;

        tilemap.replace_actor_tiles(entity, actor);
    } else {
        let animation_step_time =
            animation_step_secs(step_time.as_secs_f32(), current_direction);
        let extra = (time.elapsed_seconds_wrapped() / animation_step_time)
            .floor() as usize
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
    ///
    /// # Important
    /// Since we don't yet have entity, we don't insert tiles into the
    /// tilemap. This will be immediately remedied in the
    /// [`animate_movement`] system.
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

        // see the method docs
        let occupies = default();

        (
            Name::from(character.name()),
            Actor {
                character,
                step_time,
                direction: initial_direction,
                walking_from: T::layout().world_pos_to_square(initial_position),
                walking_to,
                occupies,
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

const O: Square = sq(0, 0);

lazy_static! {
    /// I tried implementing Bresenham's circle (filled) and couple of
    /// other options.
    /// Nothing felt satisfactory in shape.
    /// Given that most of the time we will have standard radii and layouts,
    /// and that this logic runs on every actor movement, implementing a
    /// static felt right.
    /// The squares are centered around (0, 0) and the actor's current
    /// position needs to be added to each square, or equivalently the center
    /// must be subtracted from each square before checking `contains`.
    ///
    /// The second tuple member is the distance from origin in squares.
    static ref ACTOR_ZONE_AT_ORIGIN: Vec<(Square, usize)> = {
        let distance_two = vec![
                        sq(-1, 2),sq(0, 2),sq(1, 2),
                sq(-2,1),                            sq(2,1),
                sq(-2,0),          /*O*/             sq(2,0),
                sq(-2,-1),                           sq(2,-1),
                        sq(-1,-2),sq(0,-2),sq(1,-2),
        ];
        let distance_three = vec![
        sq(-3,0),                                             sq(3,0),
        ];

        iter::once((O, 0))
            .chain(O.neighbors().map(|sq| (sq, 1)))
            .chain(distance_two.into_iter().map(|sq| (sq, 2)))
            .chain(distance_three.into_iter().map(|sq| (sq, 3)))
            .collect()
    };
}

impl<T: IntoMap> TileMap<T> {
    /// The direction the actor is facing offsets the center (`actor_stands_on`)
    /// of the circle that will be converted to occupied squares.
    fn replace_actor_tiles(&mut self, entity: Entity, actor: &mut Actor) {
        // Round up the radius to the nearest tile
        let tile_radius = (actor.character.actor_personal_zone_px()
            / T::layout().square_size)
            .ceil() as i32;

        let mut actor_zone_at_origin = ACTOR_ZONE_AT_ORIGIN.clone();

        let center = actor
            .walking_to
            .as_ref()
            .map(|t| t.square)
            .unwrap_or(actor.walking_from)
            .neighbor(GridDirection::Top); // looks better when pushed a bit forward

        actor.occupies.retain(|(sq, layer)| {
            let sq_origin = sq - center; // center around origin

            // keep overlapping squares
            if actor_zone_at_origin.remove(&sq_origin) {
                return true;
            }

            // remove the non-overlapping squares
            // we no longer use origin because we want to remove the tile from
            // the tilemap with its actual position
            let prev_tile =
                self.set_tile_kind_layer(*sq, *layer, TileKind::Empty);
            debug_assert_eq!(TileKind::Actor(entity), prev_tile);

            false
        });
        // then for the remaining squares that don't have the actor yet
        for sq_origin in actor_zone_at_origin {
            let sq = sq_origin + center; // center around actor

            // if we didn't do this then it'd be possible for actor A to occupy
            // all tiles around actor B, preventing their movement
            if !self.is_walkable(sq, entity) {
                continue;
            }

            let layer =
                self.add_tile_to_first_empty_layer(sq, TileKind::Actor(entity));
            actor.occupies.push((sq, layer));
        }
    }
}
