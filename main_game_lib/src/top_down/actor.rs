//! Player and NPC actor types.

pub mod npc;
pub mod player;

use std::{iter, time::Duration};

use bevy::{
    ecs::{entity::EntityHashMap, system::EntityCommands},
    prelude::*,
    render::view::RenderLayers,
    time::Stopwatch,
    utils::HashSet,
};
use bevy_grid_squared::{sq, GridDirection, Square};
use common_ext::QueryExt;
use common_story::Character;
use common_visuals::camera::{render_layer, PIXEL_ZOOM};
use itertools::Itertools;
use lazy_static::lazy_static;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use self::npc::BehaviorTree;
use crate::top_down::{
    layout::{ysort, TileIndex, LAYOUT},
    npc::NpcInTheMap,
    InspectLabelCategory, Player, TileKind, TileMap,
};

/// Use with [`IntoSystemConfigs::run_if`] to run a system only when an actor
/// triggers some movement event such as entering or leaving a zone.
pub fn movement_event_emitted(
) -> impl FnMut(EventReader<ActorMovementEvent>) -> bool {
    on_event::<ActorMovementEvent>()
}

/// Entity with this component can be moved around.
#[derive(Component, Reflect, Debug, Deserialize, Serialize)]
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
    pub walking_to: ActorMovement,
    /// Used for animations.
    pub direction: GridDirection,
    /// Squares that the actor currently occupies, along with the layer that
    /// the actor tile kind is assigned to.
    ///
    /// The number of squares ultimately depend on a map layout, specifically
    /// the size of a square. On more granular maps, the actor can occupy
    /// more squares.
    /// Also, different characters can occupy different squares.
    ///
    /// # Important
    /// The [`TileMap::replace_actor_tiles`] is responsible for updating this
    /// field.
    /// That logic contains an escape hatch for situations in which two actors
    /// are for some reason stuck on another.
    /// This escape hatch might set a tile to [`TileKind::Empty`] from the
    /// actor tile. Therefore, this field might contain indexes that now
    /// belong to other actors.
    /// It's necessary therefore to check the index before mutating it to
    /// confirm the escape hatch did not change it.
    occupies: Vec<TileIndex>,
}

/// Whether the actor is moving (and to where) or standing still (and for how
/// long).
#[derive(Component, Reflect, Debug, Deserialize, Serialize)]
pub enum ActorMovement {
    /// Where's the actor going
    Target(ActorTarget),
    /// Standing still
    Still {
        /// How long we've been standing still.
        since: Stopwatch,
    },
}

/// Target for an actor to walk towards.
#[derive(Reflect, Debug, Deserialize, Serialize)]
pub struct ActorTarget {
    /// The target square actor walks to.
    pub square: Square,
    /// How long we've been walking towards the target.
    ///
    /// Does not start stopped!
    /// But, you can stop it and it will resume on the next tick.
    pub since: Stopwatch,
    /// Once the current target is reached, we can plan the next one.
    pub planned: Option<(Square, GridDirection)>,
}

/// Maps actors to zones they currently occupy.
/// Each actor can be in multiple zones at once.
///
/// Only those tiles that are zones as returned by [`TileKind::is_zone`] are
/// stored.
#[derive(Resource, Serialize, Deserialize, Reflect, Default)]
#[reflect(Resource)]
pub struct ActorZoneMap {
    /// Set is used to avoid duplicates.
    /// Those could arise from a map that has the same zone multiple times in
    /// the same square (different layer.)
    ///
    /// The second tuple member is whether the actor is a player.
    map: EntityHashMap<(Character, bool, HashSet<TileKind>)>,
}

/// Some useful events for actors.
#[derive(Event, Reflect)]
pub enum ActorMovementEvent {
    /// Is emitted when an [`Actor`] enters a zone.
    ZoneEntered {
        /// The zone that was entered.
        zone: TileKind,
        /// The actor that entered the zone.
        who: Who,
    },
    /// Is emitted when an [`Actor`] leaves a zone.
    ZoneLeft {
        /// The zone that was left.
        zone: TileKind,
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
    ///
    /// If we sent [`ActorMovementEvent::ZoneEntered`] for a given entity
    /// that's a player, we guarantee that we will send a
    /// [`ActorMovementEvent::ZoneLeft`] for the same entity with the same
    /// `is_player` flag.
    pub entity: Entity,
    /// The character that entered the zone.
    pub character: Character,
    /// Where was the actor at the moment when the event was sent.
    ///
    /// Can be none if the actor was despawned.
    pub at: Option<Square>,
}

/// Helps setup a character bundle.
pub struct CharacterBundleBuilder {
    character: common_story::Character,
    initial_position: Vec2,
    initial_direction: GridDirection,
    walking_to: ActorMovement,
    initial_step_time: Option<Duration>,
    color: Option<Color>,
    behavior_tree: Option<BehaviorTree>,
}

/// Event that's emitted when the player clicks interaction near an NPC.
///
/// The entity must be an [`Actor`] but not the player.
#[derive(Event, Reflect, Clone)]
pub struct BeginDialogEvent(pub ActorOrCharacter);

/// Identify an NPC by entity or character.
#[derive(Reflect, Clone)]
pub enum ActorOrCharacter {
    /// Identify by entity.
    Actor(Entity),
    /// Identify by character.
    Character(Character),
}

/// Sends events when an actor does something interesting.
///
/// If you listen to this event then condition your system to run on
/// `run_if(on_event::<ActorMovementEvent>)` and
/// `after(actor::emit_movement_events)`.
///
/// We also emit a zone left event when an actor is despawned.
pub fn emit_movement_events(
    tilemap: Res<TileMap>,
    mut actor_zone_map: ResMut<ActorZoneMap>,
    mut event: EventWriter<ActorMovementEvent>,
    mut removed: RemovedComponents<Actor>,

    actors: Query<(Entity, &Actor), Changed<Transform>>,
) {
    for (entity, actor) in actors.iter() {
        let at = actor.current_square();
        let character = actor.character;

        let zone_left_event = |zone| ActorMovementEvent::ZoneLeft {
            zone,
            who: Who {
                at: Some(at),
                is_player: actor.is_player(),
                entity,
                character,
            },
        };

        let (_, _, active_zones) =
            actor_zone_map.map.entry(entity).or_insert_with(|| {
                (actor.character, actor.is_player(), HashSet::new())
            });

        let Some(tiles) = tilemap.get(at) else {
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
                    at: Some(at),
                    is_player: actor.is_player(),
                    entity,
                    character,
                },
            });
        }
    }

    // When an actor is despawned (or their `Actor` component is removed -
    // that's unlikely though), then we need to emit the zone left event.
    // Otherwise the zone will be left hanging with an actor that's no longer
    // there.
    //
    // There won't be any conflicts with the above loop because the actor
    // component will not be in the query.
    for entity in removed.read() {
        if let Some((character, is_player, active_zones)) =
            actor_zone_map.map.remove(&entity)
        {
            for active in active_zones {
                trace!("Actor {entity:?} despawned in zone {active:?}");
                event.send(ActorMovementEvent::ZoneLeft {
                    zone: active,
                    who: Who {
                        at: None,
                        is_player,
                        entity,
                        character,
                    },
                });
            }
        }
    }
}

/// Actually moves the actors.
/// Other systems will only edit the `Actor` component to plan the movement.
///
/// The z is based off y.
/// See the [`ysort`] for more info.
pub fn animate_movement(
    time: Res<Time>,
    mut tilemap: ResMut<TileMap>,

    mut actors: Query<
        (Entity, &mut Actor, &mut TextureAtlas, &mut Transform),
        Without<Player>,
    >,
    mut player: Query<
        (Entity, &mut Actor, &mut TextureAtlas, &mut Transform),
        With<Player>,
    >,
) {
    for (entity, mut actor, sprite, transform) in actors.iter_mut() {
        debug_assert!(!actor.is_player());

        animate_movement_for_actor(
            &time,
            &mut tilemap,
            entity,
            &mut actor,
            sprite,
            transform,
        );
    }

    // the player goes always last because of how we handle occupied tiles:
    // the later actor has an advantage
    // see `TileMap::replace_actor_tiles`
    if let Some((entity, mut actor, sprite, transform)) =
        player.get_single_mut_or_none()
    {
        debug_assert!(actor.is_player());
        animate_movement_for_actor(
            &time,
            &mut tilemap,
            entity,
            &mut actor,
            sprite,
            transform,
        );
    }
}

/// Moves the actor on screen and changes frames for the sprite.
fn animate_movement_for_actor(
    time: &Time,
    tilemap: &mut TileMap,
    entity: Entity,
    actor: &mut Actor,
    mut sprite: Mut<TextureAtlas>,
    mut transform: Mut<Transform>,
) {
    use GridDirection::*;

    let current_direction = actor.direction;
    let step_time = actor.step_time;

    let walking_to = match &mut actor.walking_to {
        ActorMovement::Target(walking_to) => walking_to,
        ActorMovement::Still { since } => {
            since.tick(time.delta());

            sprite.index = actor.character.standing_sprite_atlas_index(
                current_direction,
                time,
                Some(since.elapsed()),
            );

            // we need to update the tiles that the actor occupies because other
            // actors might be moving around it, freeing up some space
            // OPTIMIZE: the logic for replacing standing still tiles can be
            // simplified, and if it was, we could also run it when moving
            tilemap.replace_actor_tiles(entity, actor);

            // nowhere to move
            return;
        }
    };

    walking_to.since.tick(time.delta());

    // between 0 and 1, how far we are into the walk from square to square
    let lerp_factor = walking_to.since.elapsed_secs()
        / if let Top | Bottom | Left | Right = current_direction {
            step_time.as_secs_f32()
        } else {
            // we need to walk a bit slower when walking diagonally because
            // we cover more distance
            step_time.as_secs_f32() * 2.0f32.sqrt()
        };

    // the world pos in pxs where we're walking to
    let to = LAYOUT.square_to_world_pos(walking_to.square);

    if lerp_factor >= 1.0 {
        // reached the target, wat else

        let new_from = walking_to.square;

        let rounded = (to * PIXEL_ZOOM as f32).round() / PIXEL_ZOOM as f32;
        // prevents fractions if camera would want to follow the player
        transform.translation = rounded.extend(ysort(rounded));

        let standing_still_sprite_index = actor
            .character
            .standing_sprite_atlas_index(current_direction, time, None);

        if let Some((new_square, new_direction)) = walking_to.planned.take() {
            // there's still next target to walk to, let's check whether it's
            // still available

            if tilemap.is_walkable(new_square, entity) {
                walking_to.since.reset();
                walking_to.square = new_square;
                actor.direction = new_direction;
            } else {
                // can't go there anymore

                sprite.index = standing_still_sprite_index;
                actor.walking_to = default();
            }
        } else {
            // nowhere else to walk to, standing still

            sprite.index = standing_still_sprite_index;
            actor.walking_to = default();
        }

        actor.walking_from = new_from;

        tilemap.replace_actor_tiles(entity, actor);
    } else {
        // we're still walking to the target square, do the animation

        sprite.index = actor.character.walking_sprite_atlas_index(
            current_direction,
            time,
            step_time,
        );

        let from = LAYOUT.square_to_world_pos(actor.walking_from);
        let precise = from.lerp(to, lerp_factor);
        // prevents fractions if camera would want to follow the player
        let rounded = (precise * PIXEL_ZOOM as f32).round() / PIXEL_ZOOM as f32;
        transform.translation = rounded.extend(ysort(rounded));
    }
}

impl ActorMovementEvent {
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
    /// That is the square that the actor is about to arrive to if they're
    /// walking, or the square they're standing on if they're not walking.
    pub fn current_square(&self) -> Square {
        self.walking_to
            .target()
            .map(|to| to.square)
            .unwrap_or(self.walking_from)
    }

    /// Whether the actor is a player.
    ///
    /// This information is duplicated by the [`Player`] component.
    pub fn is_player(&self) -> bool {
        matches!(self.character, Character::Winnie)
    }

    /// Lets actor finish walking to the current target, but doesn't let them
    /// take the next planned step.
    fn remove_planned_step(&mut self) {
        if let Some(target) = self.walking_to.target_mut() {
            target.planned = None;
        }
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
    #[must_use]
    pub fn new(character: common_story::Character) -> Self {
        Self {
            character,
            initial_direction: GridDirection::Bottom,
            initial_position: default(),
            walking_to: default(),
            initial_step_time: default(),
            color: default(),
            behavior_tree: default(),
        }
    }

    /// Where to spawn the character.
    /// Converted into the square by [`LAYOUT`] (see
    /// the `common_layout` crate).
    /// The specific layout is provided in the
    /// [`CharacterBundleBuilder::insert_bundle_into`] method's `T`.
    pub fn initial_position(&mut self, initial_position: Vec2) -> &mut Self {
        self.initial_position = initial_position;
        self
    }

    /// Where to spawn the character.
    pub fn initial_square(&mut self, initial_square: Square) -> &mut Self {
        self.initial_position = LAYOUT.square_to_world_pos(initial_square);
        self
    }

    /// When the map is loaded, the character is spawned facing this
    /// direction.
    pub fn initial_direction(
        &mut self,
        initial_direction: GridDirection,
    ) -> &mut Self {
        self.initial_direction = initial_direction;
        self
    }

    /// Where to walk to initially.
    pub fn walking_to(&mut self, walking_to: ActorTarget) -> &mut Self {
        self.walking_to = ActorMovement::Target(walking_to);
        self
    }

    /// Sets the walking to target centered around the initial position.
    /// Note that this is best used _after_ the initial position is set.
    pub fn walking_to_from_initial_position(
        &mut self,
        walking_to: Square,
    ) -> &mut Self {
        let walking_to = ActorTarget::new(
            LAYOUT.world_pos_to_square(self.initial_position) + walking_to,
        );
        self.walking_to = ActorMovement::Target(walking_to);
        self
    }

    /// How long does it take to move one square.
    pub fn initial_step_time(&mut self, step_time: Duration) -> &mut Self {
        self.initial_step_time = Some(step_time);
        self
    }

    /// What behavior tree to use for the NPC.
    pub fn behavior_tree(
        &mut self,
        behavior_tree: impl Into<BehaviorTree>,
    ) -> &mut Self {
        self.behavior_tree = Some(behavior_tree.into());
        self
    }

    /// Spawns a bundle into the provided entity command queue.
    /// The bundle includes:
    /// - [`Name`] component with the character's name
    /// - [`Actor`] component with the character's movement data
    /// - [`SpriteSheetBundle`] with the character's sprite atlas
    ///
    /// # Important
    /// Since we don't yet have entity, we don't insert tiles into the
    /// tilemap. This will be immediately remedied in the
    /// [`animate_movement`] system, where the actor's tiles are recalculated
    /// when they stand still or when they do their first step.
    pub fn insert_bundle_into(
        self,
        asset_server: &AssetServer,
        cmd: &mut EntityCommands,
    ) {
        let id = cmd.id();

        let Self {
            character,
            initial_position,
            initial_direction,
            walking_to,
            initial_step_time: step_time,
            color,
            behavior_tree,
        } = self;

        let step_time = step_time.unwrap_or(character.default_step_time());

        if matches!(character, Character::Winnie) {
            cmd.insert(Player);
        } else {
            cmd.insert((
                NpcInTheMap::default(),
                InspectLabelCategory::Npc
                    .into_label(character.name())
                    .with_emit_event_on_interacted(BeginDialogEvent(id.into())),
            ));
        }

        if let Some(behavior_tree) = behavior_tree {
            cmd.insert(behavior_tree);
        }

        cmd.insert((
            Name::from(character.name()),
            RenderLayers::layer(render_layer::OBJ),
            Actor {
                character,
                step_time,
                direction: initial_direction,
                walking_from: LAYOUT.world_pos_to_square(initial_position),
                walking_to,
                // see the method docs
                occupies: default(),
            },
            SpriteBundle {
                texture: asset_server
                    .load(character.sprite_atlas_texture_path()),
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::BottomCenter,
                    color: color.unwrap_or_default(),
                    ..default()
                },
                transform: Transform::from_translation(
                    initial_position.extend(ysort(initial_position)),
                ),
                ..default()
            },
            TextureAtlas {
                layout: character.sprite_atlas_layout_handle(),
                index: 0,
            },
        ));
    }
}

/// Equal to where the actor is standing.
/// We will add the actors position to all values produced by
/// `ACTOR_ZONE_AT_ORIGIN`.
const O: Square = sq(0, 0);
/// We push the origin of the actor's zone shape up as it looks more natural.
const O_UP: Square = O.neighbor(GridDirection::Top);

lazy_static! {
    /// I tried implementing Bresenham's circle (filled) and couple of
    /// other options.
    /// Nothing felt satisfactory in shape.
    /// Given that most of the time we will have standard radii and layouts,
    /// and that this logic runs on every actor movement, implementing a
    /// static felt right.
    /// The squares are centered around `O` and the actor's current
    /// position needs to be added to each square, or equivalently the center
    /// must be subtracted from each square before checking `contains`.
    static ref ACTOR_ZONE_AT_ORIGIN: Vec<Square> = {
        let tiles_setup = vec![
                        sq(-1, 2),sq(0, 2),sq(1, 2),
                sq(-2,1),                            sq(2,1),
                sq(-2,0),          /*O_UP*/          sq(2,0),
                sq(-2,-1),                           sq(2,-1),
                        sq(-1,-2),sq(0,-2),sq(1,-2),
        ];

        iter::once(O_UP).chain(O_UP.neighbors_with_diagonal())
            .chain(tiles_setup.into_iter().map(|sq| sq + O_UP))
            .collect()
    };
}

impl TileMap {
    fn replace_actor_tiles(&mut self, entity: Entity, actor: &mut Actor) {
        for (sq, layer) in actor.occupies.drain(..) {
            // we can't assume it to eq the actor's tile because in some rare
            // edge cases we evict the actor, see below
            self.map_tile(sq, layer, |current| {
                if current == TileKind::Actor(entity) {
                    Some(TileKind::Empty)
                } else {
                    None
                }
            });
        }

        let actor_stands_at = actor.current_square();

        let can_move = self.can_actor_move(entity, actor_stands_at);

        // If the actor cannot move (rare but possible), we have following
        // strategies:
        if !can_move && actor.is_player() {
            // a) A player
            //    - Clear the way for the player by evicting all non-player
            //      actors from [top down left right]
            //    - Player must go last in the iteration over all actor movement

            for sq_to_clear in actor_stands_at.neighbors_with_diagonal() {
                self.map_tiles(sq_to_clear, |tile| {
                    if let TileKind::Actor(a) = tile {
                        if a != entity {
                            return TileKind::Empty;
                        }
                    }

                    tile
                });
            }
        } else if !can_move && !actor.is_player() {
            // b) An NPC
            //    - Collect all tiles that have an actor OR are walkable
            //    - Pick one at random to set the walking_to target
            //    - Continue onto the next square

            let candidates = actor_stands_at
                .neighbors_with_diagonal()
                .filter(|sq| {
                    // either has an actor or is walkable
                    self.all_on(*sq, |tile| {
                        matches!(tile, TileKind::Actor(_))
                            || tile.is_walkable(entity)
                    })
                })
                .collect_vec();

            // pick a random index from candidates
            if let Some(new_target) = candidates.choose(&mut thread_rng()) {
                actor.walking_to = ActorTarget::new(*new_target).into();
            } else {
                // Spawned in the middle of a nowhere? All directions
                // are unwalkable.
                // Perhaps caught in some big dynamic obstacle?
                error!("Actor is stuck - nowhere to go for {actor:?}");
            };
        }

        // then for the remaining squares that don't have the actor yet
        for sq_origin in ACTOR_ZONE_AT_ORIGIN.iter().copied() {
            let sq = sq_origin + actor_stands_at;

            if let Some(layer) =
                self.add_tile_to_first_empty_layer(sq, TileKind::Actor(entity))
            {
                actor.occupies.push((sq, layer));
            }
        }

        // always will be included in `ACTOR_ZONE_AT_ORIGIN`
        // can crash if the player is positioned outside of the map bounds
        debug_assert!(self.is_on(actor_stands_at, TileKind::Actor(entity)))
    }

    #[inline]
    fn can_actor_move(&self, entity: Entity, from: Square) -> bool {
        from.neighbors_with_diagonal()
            .any(|neighbor| self.is_walkable(neighbor, entity))
    }
}

impl ActorMovement {
    /// Gets mutable reference to the actor target if moving.
    pub fn target_mut(&mut self) -> Option<&mut ActorTarget> {
        match self {
            Self::Target(target) => Some(target),
            _ => None,
        }
    }

    /// Gets reference to the actor target if moving.
    pub fn target(&self) -> Option<&ActorTarget> {
        match self {
            Self::Target(target) => Some(target),
            _ => None,
        }
    }

    /// Whether the actor is not moving.
    pub fn is_still(&self) -> bool {
        matches!(self, Self::Still { .. })
    }
}

impl Default for ActorMovement {
    fn default() -> Self {
        Self::Still {
            since: Stopwatch::new(),
        }
    }
}

impl From<ActorTarget> for ActorMovement {
    fn from(target: ActorTarget) -> Self {
        Self::Target(target)
    }
}

impl From<Entity> for ActorOrCharacter {
    fn from(entity: Entity) -> Self {
        Self::Actor(entity)
    }
}

impl From<Character> for ActorOrCharacter {
    fn from(character: Character) -> Self {
        Self::Character(character)
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::SystemId;
    use rand::seq::IteratorRandom;
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn it_runs_tests_that_check_actors_dont_get_stuck_many_times() {
        for _ in 0..1000 {
            it_does_not_get_stuck_when_two_actors_are_centered_at_the_same_tile_and_walk_in_opposite_directions();

            it_does_not_get_stuck_when_first_actor_moves_and_second_stays_still(
            );

            it_does_not_get_stuck_when_first_actor_stays_still_and_second_moves(
            );
        }
    }

    #[test]
    fn it_does_not_get_stuck_when_two_actors_are_centered_at_the_same_tile_and_walk_in_opposite_directions(
    ) {
        let (mut w, system_id, marie, winnie) = prepare_world();

        let winnie_direction = GridDirection::Left;
        let marie_direction = GridDirection::Right;

        run_system_several_times(
            &mut w,
            system_id,
            &[
                (
                    winnie,
                    &[
                        winnie_direction,
                        // fallback to avoiding each other, prefer bottom
                        GridDirection::BottomLeft,
                        GridDirection::Bottom,
                        GridDirection::BottomRight,
                        GridDirection::TopLeft,
                        GridDirection::Top,
                        GridDirection::TopRight,
                    ],
                ),
                (
                    marie,
                    &[
                        marie_direction,
                        // fallback to avoiding each other, prefer top
                        GridDirection::TopRight,
                        GridDirection::Top,
                        GridDirection::TopLeft,
                        GridDirection::BottomRight,
                        GridDirection::Bottom,
                        GridDirection::BottomLeft,
                    ],
                ),
            ],
        );

        let tilemap = w.get_resource::<TileMap>().unwrap();

        let actor_pos = |actor_entity| {
            w.get_entity(actor_entity)
                .unwrap()
                .get::<Actor>()
                .unwrap()
                .walking_from
        };

        let winnie_pos = actor_pos(winnie);
        let marie_pos = actor_pos(marie);

        assert!(
            winnie_pos.x < -10,
            "Winnie is on {winnie_pos}, Marie on {marie_pos}"
        ); // you're gonna go far kid
        assert_eq!(
            &[TileKind::Actor(winnie)],
            tilemap.get(winnie_pos).unwrap()
        );

        assert!(marie_pos.x > 10); // you're gonna go far kid
        assert_eq!(&[TileKind::Actor(marie)], tilemap.get(marie_pos).unwrap());
    }

    #[test]
    fn it_does_not_get_stuck_when_first_actor_moves_and_second_stays_still() {
        let (mut w, system_id, marie, winnie) = prepare_world();

        let winnie_direction = GridDirection::Left;

        run_system_several_times(
            &mut w,
            system_id,
            &[(
                winnie,
                &[
                    winnie_direction,
                    // fallback
                    GridDirection::TopLeft,
                    GridDirection::BottomLeft,
                    // walk around if necessary
                    GridDirection::Top,
                    GridDirection::Bottom,
                    // sometimes you gotta backtrack
                    GridDirection::TopRight,
                    GridDirection::BottomRight,
                ],
            )],
        );

        let tilemap = w.get_resource::<TileMap>().unwrap();

        let actor_pos = |actor_entity| {
            w.get_entity(actor_entity)
                .unwrap()
                .get::<Actor>()
                .unwrap()
                .walking_from
        };

        let winnie_pos = actor_pos(winnie);
        assert!(winnie_pos.x < -10, "Winnie didn't make it {winnie_pos}");
        assert_eq!(
            &[TileKind::Actor(winnie)],
            tilemap.get(winnie_pos).unwrap()
        );

        let marie_pos = actor_pos(marie);
        let is_actor_alone = tilemap.all_on(marie_pos, |t| match t {
            TileKind::Actor(e) if e == marie => true,
            TileKind::Empty => true,
            _ => false,
        });
        assert!(is_actor_alone, "Marie not alone on {marie_pos}");
    }

    #[test]
    fn it_does_not_get_stuck_when_first_actor_stays_still_and_second_moves() {
        let (mut w, system_id, winnie, marie) = prepare_world();

        let marie_direction = GridDirection::Right;

        run_system_several_times(
            &mut w,
            system_id,
            &[(
                marie,
                &[
                    marie_direction,
                    // fallback
                    GridDirection::TopRight,
                    GridDirection::BottomRight,
                    // walk around if necessary
                    GridDirection::Top,
                    GridDirection::Bottom,
                    // sometimes you gotta backtrack
                    GridDirection::TopLeft,
                    GridDirection::BottomLeft,
                ],
            )],
        );

        let tilemap = w.get_resource::<TileMap>().unwrap();

        let actor_pos = |actor_entity| {
            w.get_entity(actor_entity)
                .unwrap()
                .get::<Actor>()
                .unwrap()
                .walking_from
        };

        let marie_pos = actor_pos(marie);
        assert!(marie_pos.x > 10, "Marie didn't make it {marie_pos}");
        assert_eq!(&[TileKind::Actor(marie)], tilemap.get(marie_pos).unwrap());

        let winnie_pos = actor_pos(winnie);
        let is_actor_alone = tilemap.all_on(winnie_pos, |t| match t {
            TileKind::Actor(e) if e == winnie => true,
            TileKind::Empty => true,
            _ => false,
        });
        assert!(is_actor_alone, "Winnie not alone on {winnie_pos}");
    }

    const STEP_TIME: Duration = Duration::from_secs(1);

    fn prepare_world() -> (World, SystemId, Entity, Entity) {
        let mut w = World::default();

        w.insert_resource(TileMap::default());
        w.insert_resource(Time::<()>::default());

        // both actors start at the same square

        let winnie = w
            .spawn(Actor {
                character: Character::Bolt,
                step_time: STEP_TIME,
                direction: GridDirection::Bottom,
                walking_from: sq(0, 0),
                walking_to: default(), // we get them moving later
                occupies: vec![],
            })
            .insert(SpatialBundle::default())
            .insert(TextureAtlas {
                index: 0,
                layout: Character::Bolt.sprite_atlas_layout_handle(),
            })
            .id();
        let marie = w
            .spawn(Actor {
                character: Character::Marie,
                step_time: STEP_TIME,
                direction: GridDirection::Bottom,
                walking_from: sq(0, 0),
                walking_to: default(), // we get them moving later
                occupies: vec![],
            })
            .insert(SpatialBundle::default())
            .insert(TextureAtlas {
                index: 0,
                layout: Character::Winnie.sprite_atlas_layout_handle(),
            })
            .id();

        let system_id = w.register_system(animate_movement);

        // run it once to initialize the occupied tiles
        w.run_system(system_id).unwrap();
        w.increment_change_tick();
        let actor_occupies_len = |actor_entity| {
            w.get_entity(actor_entity)
                .unwrap()
                .get::<Actor>()
                .unwrap()
                .occupies
                .len()
        };
        assert_ne!(0, actor_occupies_len(winnie));
        assert_ne!(0, actor_occupies_len(marie));

        (w, system_id, winnie, marie)
    }

    /// Run the system several times and move the actors in the given direction
    /// each time.
    /// More directions can be added to the array, first one that's walkable
    /// will be chosen.
    /// If direction array is empty, a random direction will be tried.
    fn run_system_several_times(
        w: &mut World,
        system_id: SystemId,
        actors_to_move: &[(Entity, &[GridDirection])],
    ) {
        for _ in 0..50 {
            w.run_system(system_id).unwrap();
            w.increment_change_tick();
            let mut time = w.get_resource_mut::<Time>().unwrap();
            time.advance_by(STEP_TIME + Duration::from_millis(1));

            let mut move_actor =
                |actor_entity, directions: &[GridDirection]| {
                    let tilemap =
                        TileMap::clone(w.get_resource::<TileMap>().unwrap());

                    let mut entity_ref =
                        w.get_entity_mut(actor_entity).unwrap();
                    let mut actor_comp = entity_ref.get_mut::<Actor>().unwrap();
                    if actor_comp.walking_to.is_still() {
                        let random_direction = GridDirection::iter()
                            .choose(&mut rand::thread_rng())
                            .unwrap();
                        let random_directions =
                            &[random_direction, random_direction.opposite()];
                        let directions = if directions.is_empty() {
                            random_directions
                        } else {
                            directions
                        };

                        for direction in directions {
                            let goto = actor_comp.walking_from + direction;

                            if tilemap.is_walkable(goto, actor_entity) {
                                trace!("Goto {goto}");
                                // go in direction if possible
                                actor_comp.walking_to =
                                    ActorTarget::new(goto).into();
                                break;
                            }
                        }
                    }
                };

            for (actor, direction) in actors_to_move {
                move_actor(*actor, *direction);
            }
        }
    }
}
