use common_visuals::BeginInterpolationEvent;
use main_game_lib::{
    common_ext::QueryExt,
    top_down::{actor::Who, scene_configs::ZoneTileKind, TileKind},
};
use top_down::{Actor, ActorMovementEvent, TileMap};

use super::HallwayEntity;
use crate::prelude::*;

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
    tilemap: Res<TileMap<Building1PlayerFloor>>,
    mut movement_events: EventReader<ActorMovementEvent>,
    mut lerp_event: EventWriter<BeginInterpolationEvent>,

    player: Query<&Actor, With<Player>>,
    hallway_entities: Query<Entity, With<HallwayEntity>>,
) {
    use ZoneTileKind::*;

    for event in movement_events.read() {
        match event {
            ActorMovementEvent::ZoneEntered {
                who:
                    Who {
                        is_player: true, ..
                    },
                zone:
                    TileKind::Zone(
                        HallwayZone
                        | BottomLeftApartmentZone
                        | BottomRightApartmentZone,
                    ),
            } => {
                on_player_entered_hallway(&mut lerp_event, &hallway_entities);
            }
            ActorMovementEvent::ZoneLeft {
                who:
                    Who {
                        is_player: true, ..
                    },
                zone:
                    TileKind::Zone(
                        HallwayZone
                        | BottomLeftApartmentZone
                        | BottomRightApartmentZone,
                    ),
            } => {
                on_player_left_hallway(&mut lerp_event, &hallway_entities);
            }
            ActorMovementEvent::ZoneEntered {
                who:
                    Who {
                        is_player: false,
                        entity,
                        ..
                    },
                zone:
                    TileKind::Zone(
                        HallwayZone
                        | BottomLeftApartmentZone
                        | BottomRightApartmentZone,
                    ),
            } => {
                on_npc_entered_hallway(
                    &tilemap,
                    &mut cmd,
                    &mut lerp_event,
                    &player,
                    *entity,
                );
            }
            ActorMovementEvent::ZoneLeft {
                who:
                    Who {
                        is_player: false,
                        entity,
                        ..
                    },
                zone:
                    TileKind::Zone(
                        HallwayZone
                        | BottomLeftApartmentZone
                        | BottomRightApartmentZone,
                    ),
            } => {
                on_npc_left_hallway(&mut cmd, &mut lerp_event, *entity);
            }

            // we don't care about any other event
            _ => {}
        }
    }
}

fn on_player_entered_hallway(
    lerp_event: &mut EventWriter<BeginInterpolationEvent>,
    hallway_entities: &Query<Entity, With<HallwayEntity>>,
) {
    trace!("Player entered hallway");
    hallway_entities.iter().for_each(|entity| {
        lerp_event.send(
            BeginInterpolationEvent::of_color(entity, None, Color::WHITE)
                .over(HALLWAY_FADE_IN_TRANSITION_DURATION),
        );
    });
}

fn on_player_left_hallway(
    lerp_event: &mut EventWriter<BeginInterpolationEvent>,
    hallway_entities: &Query<Entity, With<HallwayEntity>>,
) {
    trace!("Player left hallway");
    hallway_entities.iter().for_each(|entity| {
        lerp_event.send(
            BeginInterpolationEvent::of_color(entity, None, PRIMARY_COLOR)
                .over(HALLWAY_FADE_OUT_TRANSITION_DURATION),
        );
    });
}

fn on_npc_entered_hallway(
    tilemap: &TileMap<Building1PlayerFloor>,
    cmd: &mut Commands,
    lerp_event: &mut EventWriter<BeginInterpolationEvent>,
    player: &Query<&Actor, With<Player>>,
    entity: Entity,
) {
    trace!("NPC entered hallway");
    cmd.entity(entity).insert(HallwayEntity);

    let is_player_in_hallway = player
        .get_single_or_none()
        .map(|player| {
            tilemap.is_on(player.walking_from, ZoneTileKind::HallwayZone)
        })
        .unwrap_or(false);

    // if actor in the hallway but player is not, we need to change
    // their color back to primary
    if is_player_in_hallway {
        lerp_event.send(
            BeginInterpolationEvent::of_color(entity, None, PRIMARY_COLOR)
                .over(HALLWAY_FADE_OUT_TRANSITION_DURATION),
        );
    }
}

fn on_npc_left_hallway(
    cmd: &mut Commands,
    lerp_event: &mut EventWriter<BeginInterpolationEvent>,
    entity: Entity,
) {
    trace!("NPC left hallway");
    cmd.entity(entity).remove::<HallwayEntity>();

    // if actor not in hallway, we need to change their color
    lerp_event.send(
        BeginInterpolationEvent::of_color(entity, None, Color::WHITE)
            .over(HALLWAY_FADE_IN_TRANSITION_DURATION),
    );
}
