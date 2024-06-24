use common_visuals::BeginInterpolationEvent;
use top_down::{actor::Who, ActorMovementEvent, TileKind};

use super::ApartmentWall;
use crate::prelude::*;

/// How long does it take for the entity to go transparent
const WALL_FADE_OUT_TRANSITION_DURATION: Duration = from_millis(500);
/// How long does it take for the entity to go to its full color.
const WALL_FADE_IN_TRANSITION_DURATION: Duration = from_millis(1500);

/// Listens to events about entering the
/// [`Building1Basement1TileKind::UpperApartmentWallHiddenZone`].
///
/// When entered, the [`ApartmentWall`] entity is hidden.
pub(super) fn system(
    mut movement_events: EventReader<
        ActorMovementEvent<<Building1Basement1 as TopDownScene>::LocalTileKind>,
    >,
    mut lerp_event: EventWriter<BeginInterpolationEvent>,

    wall: Query<Entity, With<ApartmentWall>>,
) {
    use Building1Basement1TileKind::UpperApartmentWallHiddenZone as TheZone;

    for event in movement_events.read() {
        match event {
            ActorMovementEvent::ZoneEntered {
                who:
                    Who {
                        is_player: true, ..
                    },
                zone: TileKind::Local(TheZone),
            } => {
                trace!("Hiding apartment wall");
                lerp_event.send(
                    BeginInterpolationEvent::of_color(
                        wall.single(),
                        None,
                        Color::NONE,
                    )
                    .over(WALL_FADE_OUT_TRANSITION_DURATION),
                );
            }
            ActorMovementEvent::ZoneLeft {
                who:
                    Who {
                        is_player: true, ..
                    },
                zone: TileKind::Local(TheZone),
            } => {
                trace!("Showing apartment wall");
                lerp_event.send(
                    BeginInterpolationEvent::of_color(
                        wall.single(),
                        None,
                        Color::WHITE,
                    )
                    .over(WALL_FADE_IN_TRANSITION_DURATION),
                );
            }

            // we don't care about other events
            _ => {}
        }
    }
}
