use bevy::{render::view::RenderLayers, sprite::Anchor};
use bevy_grid_squared::sq;
use common_top_down::{
    actor::{self, movement_event_emitted, Who},
    environmental_objects::{
        self,
        door::{DoorBuilder, DoorOpenCriteria, DoorState},
    },
    inspect_and_interact::ZoneToInspectLabelEntity,
    Actor, ActorMovementEvent, InspectLabelCategory, TileKind, TileMap,
};
use common_visuals::{
    camera::render_layer, AtlasAnimation, AtlasAnimationEnd,
    AtlasAnimationTimer, BeginInterpolationEvent,
};
use main_game_lib::{
    common_ext::QueryExt,
    scene_maker::{self, SceneSerde, SpriteScene, SpriteSceneHandle},
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::{actor::ApartmentAction, consts::*, prelude::*, Apartment};

/// How long does it take to give hallway its full color.
const HALLWAY_FADE_IN_TRANSITION_DURATION: Duration = from_millis(500);
/// How long when leaving the hallway to make it into the dark primary color.
const HALLWAY_FADE_OUT_TRANSITION_DURATION: Duration = from_millis(1500);

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "devtools")]
        app.register_type::<ApartmentTileKind>();

        app.add_systems(
            Update,
            spawn.run_if(in_state(GlobalGameState::ApartmentLoading)),
        )
        .add_systems(
            Update,
            spawn2
                .run_if(in_state(GlobalGameState::ApartmentLoading))
                .run_if(not(
                    scene_maker::are_sprites_spawned_and_file_despawned::<
                        Apartment,
                    >(),
                )),
        )
        .add_systems(OnExit(GlobalGameState::ApartmentQuitting), despawn)
        .add_systems(
            Update,
            (
                watch_entry_to_hallway,
                environmental_objects::door::toggle::<Apartment>,
            )
                .run_if(in_state(GlobalGameState::InApartment))
                .run_if(movement_event_emitted::<Apartment>())
                .after(actor::emit_movement_events::<Apartment>),
        );
    }
}

// TODO
fn spawn2(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut scenes: ResMut<Assets<SceneSerde>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    tilemap: Option<ResMut<TileMap<Apartment>>>,

    q: Query<(Entity, &SpriteSceneHandle<Apartment>)>,
) {
    let Some((handle_entity, SpriteSceneHandle { handle, .. })) =
        q.get_single_or_none()
    else {
        trace!("Spawning Apartment sprite scene handle");
        cmd.spawn(SpriteSceneHandle::<Apartment>::new(
            asset_server.load(<Apartment as SpriteScene>::asset_path()),
        ));
        return;
    };
    if !asset_server.is_loaded_with_dependencies(handle) {
        return;
    }
    let Some(mut tilemap) = tilemap else {
        return; // wait for tilemap to load
    };

    let mut scene = scenes.remove(handle).expect("Scene file is loaded");

    info!("Spawning apartment scene");

    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();

    while let Some((mut entity_cmd, name)) = scene
        .spawn_next_sprite::<Apartment>(
            &mut cmd,
            &asset_server,
            &mut atlas_layouts,
        )
    {
        trace!("Spawned {name:?} from scene file");

        entity_cmd
            .insert((LayoutEntity, RenderLayers::layer(render_layer::BG)));

        match name.as_str() {
            "Bedroom, bathroom and kitchen background"
            | "Bathroom toilet"
            | "Bedroom shoe rack"
            | "Kitchen table"
            | "Kitchen fridge"
            | "Bedroom cupboard"
            | "Bedroom laundry basket"
            | "Back wall furniture" => {}
            "Hallway background" | "Hallway door #1" | "Hallway door #2" => {
                entity_cmd.insert(HallwayEntity);
                entity_cmd.add(|mut w: EntityWorldMut| {
                    w.get_mut::<Sprite>().expect("Sprite").color =
                        PRIMARY_COLOR;
                });
            }
            "Bedroom meditation chair" => {
                zone_to_inspect_label_entity
                    .map
                    .insert(ApartmentTileKind::MeditationZone, entity_cmd.id());

                entity_cmd.insert(
                    InspectLabelCategory::Default
                        .into_label("Meditate")
                        .emit_event_on_interacted(
                            ApartmentAction::StartMeditation,
                        ),
                );
            }
            _ => {
                error!("Sprite {name:?} not handled");
            }
        }
    }

    cmd.entity(handle_entity).despawn();
}

#[derive(Component)]
struct LayoutEntity;

/// Hallway is darkened when the player is in the apartment but once the player
/// approaches the door or is in the hallway, it's lit up.
#[derive(Component)]
pub(crate) struct HallwayEntity;

/// Elevator is a special entity that has a sprite sheet with several frames.
/// It opens when an actor is near it and closes when the actor leaves or
/// enters.
#[derive(Component)]
pub(crate) struct Elevator;

/// We arbitrarily derive the [`Default`] to allow reflection.
/// It does not have a meaningful default value.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    EnumIter,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Reflect,
    Serialize,
    strum::Display,
)]
#[reflect(Default)]
#[allow(clippy::enum_variant_names)]
pub enum ApartmentTileKind {
    /// We want to darken the hallway when the player is in the apartment.
    HallwayZone,
    /// Everything that's in the player's apartment.
    PlayerApartmentZone,
    #[default]
    BedZone,
    ElevatorZone,
    PlayerDoorZone,
    MeditationZone,
    TeaZone,
}

fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    tilemap: Option<ResMut<TileMap<Apartment>>>,

    entities: Query<Entity, With<LayoutEntity>>,
) {
    let Some(mut tilemap) = tilemap else {
        return; // wait for tilemap to load
    };

    if !entities.is_empty() {
        return; // already spawned
    }

    let mut zone_to_inspect_label_entity = ZoneToInspectLabelEntity::default();

    // cloud atlas is rendered on top of the bg but below the furniture

    let mut cloud_atlas_bundle = |position: Vec2| {
        (
            LayoutEntity,
            RenderLayers::layer(render_layer::BG),
            AtlasAnimation {
                on_last_frame: AtlasAnimationEnd::Loop,
                first: 0,
                last: CLOUD_FRAMES - 1,
                ..default()
            },
            AtlasAnimationTimer::new(
                CLOUD_ATLAS_FRAME_TIME,
                TimerMode::Repeating,
            ),
            SpriteSheetBundle {
                texture: asset_server.load(assets::CLOUD_ATLAS),
                atlas: TextureAtlas {
                    index: thread_rng().gen_range(0..CLOUD_FRAMES),
                    layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                        vec2(CLOUD_WIDTH, CLOUD_HEIGHT),
                        CLOUD_FRAMES,
                        1,
                        Some(vec2(CLOUD_PADDING, 0.0)),
                        None,
                    )),
                },
                transform: Transform::from_translation(
                    position.extend(zindex::CLOUD_ATLAS),
                ),
                ..default()
            },
        )
    };

    cmd.spawn(Name::from("Bedroom cloud atlas"))
        .insert(cloud_atlas_bundle(vec2(-70.5, 113.5)));
    cmd.spawn(Name::from("Kitchen cloud atlas"))
        .insert(cloud_atlas_bundle(vec2(96.0, 113.5)));
    cmd.spawn(Name::from("Bathroom cloud atlas"))
        .insert(cloud_atlas_bundle(vec2(-176.0, 145.0)));

    // TODO: the other two doors
    // bedroom door opens (sprite index 2) when the player is near the door
    cmd.spawn((
        Name::from("Bedroom door"),
        DoorBuilder::new(ApartmentTileKind::PlayerDoorZone)
            .add_open_criteria(DoorOpenCriteria::Character(
                common_story::Character::Winnie,
            ))
            .add_open_criteria(DoorOpenCriteria::Character(
                common_story::Character::Unnamed,
            ))
            .with_initial_state(DoorState::Closed)
            .with_obstacle_when_closed_between(sq(-40, -21), sq(-31, -21))
            .build(&mut tilemap),
        LayoutEntity,
        RenderLayers::layer(render_layer::BG),
        SpriteSheetBundle {
            texture: asset_server.load(assets::BEDROOM_MAIN_DOOR),
            atlas: TextureAtlas {
                index: 0,
                layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                    vec2(27.0, 53.0),
                    2,
                    1,
                    Some(vec2(1.0, 0.0)),
                    None,
                )),
            },
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_translation(
                // sometimes to make the game feel better, the z coordinate
                // needs to be adjusted
                <Apartment as TopDownScene>::extend_z_with_y_offset(
                    vec2(-105.0, -88.0),
                    8.5,
                ),
            ),
            ..default()
        },
    ));

    // the elevator takes the player to the next location
    let elevator = cmd
        .spawn((
            Name::from("Elevator"),
            Elevator,
            LayoutEntity,
            HallwayEntity,
            RenderLayers::layer(render_layer::BG),
            InspectLabelCategory::Default
                .into_label("Elevator")
                .emit_event_on_interacted(ApartmentAction::EnterElevator),
            // this animation is important for elevator cutscene
            AtlasAnimation {
                on_last_frame: AtlasAnimationEnd::RemoveTimer,
                first: 0,
                last: 7,
                ..default()
            },
            SpriteSheetBundle {
                texture: asset_server.load(assets::ELEVATOR_ATLAS),
                atlas: TextureAtlas {
                    index: 0,
                    layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                        vec2(51.0, 57.0),
                        8,
                        1,
                        Some(vec2(4.0, 0.0)),
                        None,
                    )),
                },
                sprite: Sprite {
                    color: PRIMARY_COLOR,
                    ..default()
                },
                transform: Transform::from_translation(
                    vec2(-201.5, -49.0).extend(zindex::ELEVATOR),
                ),
                ..default()
            },
        ))
        .id();
    zone_to_inspect_label_entity
        .map
        .insert(ApartmentTileKind::ElevatorZone, elevator);

    cmd.spawn((
        Name::from("Vending machine"),
        LayoutEntity,
        HallwayEntity,
        RenderLayers::layer(render_layer::BG),
        SpriteSheetBundle {
            texture: asset_server.load(assets::VENDING_MACHINE_ATLAS),
            atlas: TextureAtlas {
                index: 0,
                layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                    vec2(30.0, 55.0),
                    4,
                    1,
                    Some(vec2(1.0, 0.0)),
                    None,
                )),
            },
            sprite: Sprite {
                color: PRIMARY_COLOR,
                ..default()
            },
            transform: Transform::from_translation(
                vec2(-268.0, -60.0).extend(zindex::ELEVATOR),
            ),
            ..default()
        },
    ));

    cmd.insert_resource(zone_to_inspect_label_entity);
}

fn despawn(mut cmd: Commands, query: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    for entity in query.iter() {
        cmd.entity(entity).despawn_recursive();
    }

    cmd.remove_resource::<ZoneToInspectLabelEntity<
        <Apartment as TopDownScene>::LocalTileKind,
    >>();
}

/// Listens to events about entering the hallway (or just coming to the doors.)
///
/// When the player enters the hallway, all hallway entities go from primary
/// color to white.
/// When the player leaves the hallway, reversed.
///
/// When an NPC (non player actor) enters the hallway, assign them the hallway
/// component.
/// When an NPC leaves the hallway, remove the hallway component.
fn watch_entry_to_hallway(
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

impl common_top_down::layout::Tile for ApartmentTileKind {
    #[inline]
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    #[inline]
    fn is_zone(&self) -> bool {
        match self {
            Self::BedZone
            | Self::PlayerDoorZone
            | Self::PlayerApartmentZone
            | Self::ElevatorZone
            | Self::HallwayZone
            | Self::MeditationZone
            | Self::TeaZone => true,
        }
    }

    #[inline]
    fn zones_iter() -> impl Iterator<Item = Self> {
        Self::iter().filter(|kind| kind.is_zone())
    }
}
