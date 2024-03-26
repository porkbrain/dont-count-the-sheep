use common_visuals::BeginInterpolationEvent;
use main_game_lib::common_ext::QueryExt;
use top_down::{actor::Who, Actor, ActorMovementEvent, TileKind, TileMap};

use super::{ApartmentTileKind, HallwayEntity};
use crate::{prelude::*, Apartment};

/// How long does it take to give hallway its full color.
const HALLWAY_FADE_IN_TRANSITION_DURATION: Duration = from_millis(500);
/// How long when leaving the hallway to make it into the dark primary color.
const HALLWAY_FADE_OUT_TRANSITION_DURATION: Duration = from_millis(1500);

/// Listens to events about entering the hallway (or just coming to the doors.)
///
/// When the player enters the hallway, all hallway entities go from primary
/// color to white.
/// When the player leaves the hallway, reversed.
///
/// When an NPC (non player actor) enters the hallway, assign them the hallway
/// component.
/// When an NPC leaves the hallway, remove the hallway component.
pub(super) fn system(
    mut cmd: Commands,
    tilemap: Res<TileMap<Apartment>>,
    mut movement_events: EventReader<
        ActorMovementEvent<<Apartment as TopDownScene>::LocalTileKind>,
    >,
    mut lerp_event: EventWriter<BeginInterpolationEvent>,

    player: Query<&Actor, With<Player>>,
    hallway_entities: Query<Entity, With<HallwayEntity>>,
) {
    for event in movement_events.read() {
        match event {
            // player entered hallway or is by the door, all entities go to
            // white
            ActorMovementEvent::ZoneEntered {
                who:
                    Who {
                        is_player: true, ..
                    },
                zone: TileKind::Local(ApartmentTileKind::HallwayZone),
            }
            | ActorMovementEvent::ZoneEntered {
                who:
                    Who {
                        is_player: true, ..
                    },
                zone: TileKind::Local(ApartmentTileKind::PlayerDoorZone),
            } => {
                trace!("Player entered hallway");
                hallway_entities.iter().for_each(|entity| {
                    lerp_event.send(
                        BeginInterpolationEvent::of_color(
                            entity,
                            None,
                            Color::WHITE,
                        )
                        .over(HALLWAY_FADE_IN_TRANSITION_DURATION),
                    );
                });
            }
            // player left hallway, all entities go to primary
            ActorMovementEvent::ZoneLeft {
                who:
                    Who {
                        is_player: true, ..
                    },
                zone: TileKind::Local(ApartmentTileKind::HallwayZone),
            } => {
                trace!("Player left hallway");
                hallway_entities.iter().for_each(|entity| {
                    lerp_event.send(
                        BeginInterpolationEvent::of_color(
                            entity,
                            None,
                            PRIMARY_COLOR,
                        )
                        .over(HALLWAY_FADE_OUT_TRANSITION_DURATION),
                    );
                });
            }
            // Player left the door zone. This mean either
            // a) they are in the hallway - don't do anything
            // b) they are in the apartment - darken the hallway
            ActorMovementEvent::ZoneLeft {
                who:
                    Who {
                        at: Some(sq),
                        is_player: true,
                        ..
                    },
                zone: TileKind::Local(ApartmentTileKind::PlayerDoorZone),
            } if !tilemap.is_on(
                *sq,
                TileKind::Local(ApartmentTileKind::HallwayZone),
            ) =>
            {
                // b)
                trace!("Player left the door zone into the apartment");
                hallway_entities.iter().for_each(|entity| {
                    lerp_event.send(
                        BeginInterpolationEvent::of_color(
                            entity,
                            None,
                            PRIMARY_COLOR,
                        )
                        .over(HALLWAY_FADE_OUT_TRANSITION_DURATION),
                    );
                });
            }
            // NPC entered the hallway
            ActorMovementEvent::ZoneEntered {
                who:
                    Who {
                        is_player: false,
                        entity,
                        ..
                    },
                zone: TileKind::Local(ApartmentTileKind::HallwayZone),
            } => {
                trace!("NPC entered hallway");
                cmd.entity(*entity).insert(HallwayEntity);

                let is_player_in_hallway = player
                    .get_single_or_none()
                    .map(|player| {
                        tilemap.is_on(
                            player.walking_from,
                            ApartmentTileKind::HallwayZone,
                        )
                    })
                    .unwrap_or(false);

                // if actor in the hallway but player is not, we need to change
                // their color back to primary
                if !is_player_in_hallway {
                    lerp_event.send(
                        BeginInterpolationEvent::of_color(
                            *entity,
                            None,
                            PRIMARY_COLOR,
                        )
                        .over(HALLWAY_FADE_OUT_TRANSITION_DURATION),
                    );
                }
            }
            // NPC left the hallway
            ActorMovementEvent::ZoneLeft {
                who:
                    Who {
                        is_player: false,
                        entity,
                        ..
                    },
                zone: TileKind::Local(ApartmentTileKind::HallwayZone),
            } => {
                trace!("NPC left hallway");
                cmd.entity(*entity).remove::<HallwayEntity>();

                // if actor not in hallway, we need to change their color
                lerp_event.send(
                    BeginInterpolationEvent::of_color(
                        *entity,
                        None,
                        Color::WHITE,
                    )
                    .over(HALLWAY_FADE_IN_TRANSITION_DURATION),
                );
            }
            // we don't care about other events
            _ => {}
        }
    }
}
