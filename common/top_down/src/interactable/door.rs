//! Doors that can be opened and closed.
//! There are different [`DoorOpenCriteria`] that can be used to open the door.
//! Optionally, the door can have an obstacle that's inserted into the map when
//! the door is closed.

use bevy::prelude::*;
use bevy_grid_squared::Square;
use itertools::Itertools;
use smallvec::SmallVec;

use crate::{ActorMovementEvent, IntoMap, TileKind, TileMap};

/// For [`Door`]
pub struct DoorBuilder<L> {
    /// If an actor is in this zone, the door can be manipulated.
    zone_tile_kind: TileKind<L>,

    open_criteria: SmallVec<[DoorOpenCriteria; 3]>,
    initial_state: DoorState,
    /// If set, then when closed we draw a wall between these two squares.
    obstacle: Option<(Square, Square)>,
}

impl<L> DoorBuilder<L> {
    /// The only required parameter is the zone tile kind that opens the door.
    pub fn new(zone_tile_kind: impl Into<TileKind<L>>) -> Self {
        Self {
            zone_tile_kind: zone_tile_kind.into(),
            open_criteria: default(),
            initial_state: DoorState::Closed,
            obstacle: None,
        }
    }

    /// If the door is closed, we insert a wall between these two squares.
    pub fn with_obstacle_when_closed_between(
        mut self,
        from: Square,
        to: Square,
    ) -> Self {
        self.obstacle = Some((from, to));
        self
    }

    /// The initial state of the door.
    /// Note that if the initial door is open, you need to graphically set the
    /// sprite to the open state.
    pub fn with_initial_state(mut self, state: DoorState) -> Self {
        self.initial_state = state;
        self
    }

    /// Add a criteria for opening the door.
    pub fn add_open_criteria(mut self, criteria: DoorOpenCriteria) -> Self {
        self.open_criteria.push(criteria);
        self
    }

    /// If the door is closed, we insert a wall between the obstacle squares
    /// if set.
    pub fn build<T: IntoMap>(self, tilemap: &mut TileMap<T>) -> Door<L> {
        let obstacle = self.obstacle.map(|(from, to)| {
            let layers = if matches!(self.initial_state, DoorState::Closed) {
                bevy_grid_squared::shapes::rectangle_between(from, to)
                    .map(|sq| {
                        tilemap
                            .add_tile_to_first_empty_layer(sq, TileKind::Wall)
                    })
                    .collect()
            } else {
                vec![]
            };

            DoorObstacle {
                rect: (from, to),
                layers,
            }
        });

        Door {
            zone_tile_kind: self.zone_tile_kind,
            state: self.initial_state,
            open_criteria: self.open_criteria,
            obstacle,

            actors_near: 0,
        }
    }
}

/// A door that can be opened and closed.
#[derive(Component, Reflect)]
pub struct Door<L> {
    /// If an actor is in this zone, the door can be manipulated.
    zone_tile_kind: TileKind<L>,
    /// The state is updated on runtime.
    state: DoorState,
    /// In an `OR` relationship.
    open_criteria: SmallVec<[DoorOpenCriteria; 3]>,
    /// If defined then when the door is closed, we set `Wall` .
    obstacle: Option<DoorObstacle>,
    /// Only when this gets to 0 do we close the door.
    actors_near: usize,
}

/// When the door is closed, we insert a wall between these two squares.
#[derive(Reflect)]
struct DoorObstacle {
    /// Area where the `Wall` is placed when the door is closed.
    rect: (Square, Square),
    /// When the door is closed, each square has a wall that's been inserted
    /// in some layer.
    /// We remember the layer so that we can remove the wall when the door
    /// opens.
    /// Note that the way the layers are mapped onto the tilemap with
    /// [`bevy_grid_squared::shapes::rectangle_between`].
    layers: Vec<usize>,
}

/// When all conditions are met, the door opens.
#[derive(Reflect, Default, Clone, Copy)]
pub enum DoorState {
    /// The door is open and can be walked through.
    /// If the door has an obstacle, it's removed.
    Open,
    /// If the door has an obstacle, it's inserted into the map as wall.
    #[default]
    Closed,
}

/// Door can have different criteria for opening.
#[derive(Reflect)]
pub enum DoorOpenCriteria {
    /// Can only be opened by a specific character.
    /// Note that when the door is open, _any_ character can walk through it.
    Character(common_story::Character),
}

/// When player gets near the door, the door opens.
///
/// Run this after the [`crate::actor::emit_movement_events`] system and only
/// if there are events.
pub fn toggle<T: IntoMap>(
    mut tilemap: ResMut<TileMap<T>>,
    mut events: EventReader<ActorMovementEvent<T::LocalTileKind>>,

    mut door: Query<(&mut Door<T::LocalTileKind>, &mut TextureAtlasSprite)>,
) {
    let events = events.read().collect_vec();

    for (mut door, mut sprite) in door.iter_mut() {
        for event in &events {
            apply_event_to_door_and_map(
                &mut tilemap,
                &mut door,
                &mut sprite,
                event,
            );
        }
    }
}

/// When an actor gets near the door, the door opens if criteria are met.
/// When the actor leaves the zone, the door closes.
///
/// Optionally, the door can have an obstacle that's inserted into the map when
/// the door is closed.
fn apply_event_to_door_and_map<T: IntoMap>(
    tilemap: &mut ResMut<TileMap<T>>,
    door: &mut Door<T::LocalTileKind>,
    sprite: &mut Mut<'_, TextureAtlasSprite>,
    event: &ActorMovementEvent<<T as IntoMap>::LocalTileKind>,
) {
    match event {
        ActorMovementEvent::ZoneEntered { zone, who }
            if *zone == door.zone_tile_kind =>
        {
            door.actors_near += 1;

            if !matches!(door.state, DoorState::Closed) {
                return;
            }

            let can_be_opened =
                door.open_criteria.iter().any(|criteria| match criteria {
                    DoorOpenCriteria::Character(character) => {
                        who.character == *character
                    }
                });

            if !can_be_opened {
                return;
            }

            trace!("Open door");

            door.state = DoorState::Open;
            sprite.index = 1;

            if let Some(DoorObstacle {
                rect: (from, to),
                layers,
            }) = door.obstacle.as_mut()
            {
                bevy_grid_squared::shapes::rectangle_between(*from, *to)
                    .zip(layers.drain(..))
                    .for_each(|(sq, layer)| {
                        tilemap.set_tile_kind_layer(sq, layer, TileKind::Empty);
                    });
            }
        }
        ActorMovementEvent::ZoneLeft { zone, .. }
            if *zone == door.zone_tile_kind =>
        {
            door.actors_near -= 1;

            if door.actors_near > 0 {
                return;
            }

            trace!("Close door");
            door.state = DoorState::Closed;
            sprite.index = 0;

            if let Some(DoorObstacle {
                rect: (from, to),
                layers,
            }) = door.obstacle.as_mut()
            {
                bevy_grid_squared::shapes::rectangle_between(*from, *to)
                    .for_each(|sq| {
                        layers.push(
                            tilemap.add_tile_to_first_empty_layer(
                                sq,
                                TileKind::Wall,
                            ),
                        );
                    });
            }
        }
        _ => {}
    };
}
