use bevy::{
    ecs::event::event_update_condition, render::view::RenderLayers,
    sprite::Anchor, utils::Instant,
};
use bevy_grid_squared::{square, SquareLayout};
use common_visuals::{
    camera::render_layer, AtlasAnimation, AtlasAnimationEnd,
    AtlasAnimationTimer, ColorExt, PRIMARY_COLOR,
};
use lazy_static::lazy_static;
use main_game_lib::{
    common_ext::QueryExt,
    common_top_down::{
        actor,
        interactable::{
            self,
            door::{DoorBuilder, DoorOpenCriteria, DoorState},
        },
        Actor, ActorMovementEvent, IntoMap, TileMap,
    },
    cutscene::not_in_cutscene,
    vec2_ext::Vec2Ext,
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{consts::*, prelude::*, Apartment};

lazy_static! {
    static ref LAYOUT: SquareLayout = SquareLayout {
        square_size: 4.0,
        origin: vec2(356.0, 175.0).as_top_left_into_centered(),
    };
}

/// How long does it take to give hallway its full color.
const HALLWAY_FADE_TRANSITION_DURATION: Duration = from_millis(500);
/// In terms of map squares.
/// We don't use bounding box because so far the apartment does not
/// extend beyond this boundary.
const HALLWAY_TOP_BOUNDARY: i32 = -20;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ApartmentTileKind>();

        app.add_systems(
            Update,
            spawn.run_if(in_state(GlobalGameState::ApartmentLoading)),
        )
        .add_systems(OnExit(GlobalGameState::ApartmentQuitting), despawn)
        .add_systems(
            Update,
            smoothly_transition_hallway_color
                .run_if(in_state(GlobalGameState::InApartment))
                .run_if(not_in_cutscene()),
        )
        .add_systems(
            Update,
            interactable::door::toggle::<Apartment>
                .run_if(in_state(GlobalGameState::InApartment))
                .run_if(
                    event_update_condition::<
                        ActorMovementEvent<
                            <Apartment as IntoMap>::LocalTileKind,
                        >,
                    >,
                )
                .after(actor::emit_movement_events::<Apartment>),
        );
    }
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
    Eq,
    Hash,
    PartialEq,
    Reflect,
    Serialize,
)]
#[reflect(Default)]
pub(crate) enum ApartmentTileKind {
    #[default]
    BedZone,
    MainDoorZone,
    ElevatorZone,
    /// We want to darken the hallway when the player is in the apartment.
    HallwayZone,
    MeditationZone,
    TeaZone,
}

fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    tilemap: Option<ResMut<TileMap<Apartment>>>,

    entities: Query<Entity, With<LayoutEntity>>,
) {
    let Some(mut tilemap) = tilemap else {
        return; // wait for tilemap to load
    };

    if !entities.is_empty() {
        return; // already spawned
    }

    #[derive(Default)]
    struct ToSpawn {
        name: &'static str,
        asset: &'static str,
        // if not specified, it's calculated from position
        zindex: Option<f32>,
        position: Vec2,
        color: Option<Color>,
        is_hallway_entity: bool,
        anchor: Option<Anchor>,
        anchor_y_offset: f32,
    }

    for ToSpawn {
        name,
        asset,
        zindex,
        color,
        is_hallway_entity,
        position,
        anchor,
        anchor_y_offset,
    } in [
        ToSpawn {
            name: "Bedroom, bathroom and kitchen background",
            asset: assets::BG,
            zindex: Some(zindex::BG_BATHROOM_BEDROOM_AND_KITCHEN),
            ..default()
        },
        ToSpawn {
            name: "Back wall furniture",
            asset: assets::BACKWALL_FURNITURE,
            zindex: Some(zindex::BACKWALL_FURNITURE),
            ..default()
        },
        ToSpawn {
            name: "Bathroom toilet",
            asset: assets::TOILET,
            position: vec2(-143.0, -10.0),
            anchor: Some(Anchor::BottomCenter),
            ..default()
        },
        ToSpawn {
            name: "Bedroom cupboard",
            asset: assets::CUPBOARD,
            position: vec2(-95.0, -25.0),
            anchor: Some(Anchor::BottomCenter),
            anchor_y_offset: 2.5,
            ..default()
        },
        ToSpawn {
            name: "Bedroom laundry basket",
            asset: assets::LAUNDRY_BASKET,
            position: vec2(-51.0, -25.0),
            anchor: Some(Anchor::BottomCenter),
            anchor_y_offset: 5.0,
            ..default()
        },
        ToSpawn {
            name: "Bedroom shoe rack",
            asset: assets::SHOERACK,
            position: vec2(-63.0, -77.0),
            anchor: Some(Anchor::BottomCenter),
            ..default()
        },
        ToSpawn {
            name: "Kitchen fridge",
            asset: assets::FRIDGE,
            position: vec2(79.0, 37.0),
            anchor: Some(Anchor::BottomCenter),
            ..default()
        },
        ToSpawn {
            name: "Kitchen table",
            asset: assets::KITCHEN_TABLE,
            position: vec2(151.0, 11.0),
            anchor: Some(Anchor::BottomCenter),
            ..default()
        },
        ToSpawn {
            name: "Hallway background",
            asset: assets::HALLWAY,
            zindex: Some(zindex::BG_HALLWAY),
            color: Some(PRIMARY_COLOR),
            is_hallway_entity: true,
            ..default()
        },
        ToSpawn {
            name: "Hallway door #1",
            asset: assets::HALLWAY_DOOR,
            color: Some(PRIMARY_COLOR),
            is_hallway_entity: true,
            position: vec2(-204.0, -121.0),
            anchor: Some(Anchor::BottomCenter),
            ..default()
        },
        ToSpawn {
            name: "Hallway door #2",
            asset: assets::HALLWAY_DOOR,
            color: Some(PRIMARY_COLOR),
            is_hallway_entity: true,
            position: vec2(19.0, -121.0),
            anchor: Some(Anchor::BottomCenter),
            ..default()
        },
    ] {
        let translate = zindex
            .map(|zindex| position.extend(zindex))
            .unwrap_or_else(|| {
                Apartment::extend_z_with_y_offset(position, anchor_y_offset)
            });
        let mut entity = cmd.spawn((
            Name::from(name),
            LayoutEntity,
            RenderLayers::layer(render_layer::BG),
            SpriteBundle {
                texture: asset_server.load(asset),
                transform: Transform::from_translation(translate),
                sprite: Sprite {
                    color: color.unwrap_or_default(),
                    anchor: anchor.unwrap_or_default(),
                    ..default()
                },
                ..default()
            },
        ));

        if is_hallway_entity {
            entity.insert(HallwayEntity);
        }
    }

    // cloud atlas is rendered on top of the bg but below the furniture

    let mut cloud_atlas_bundle = |position: Pos2| {
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
                texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load(assets::CLOUD_ATLAS),
                    vec2(CLOUD_WIDTH, CLOUD_HEIGHT),
                    CLOUD_FRAMES,
                    1,
                    Some(vec2(CLOUD_PADDING, 0.0)),
                    None,
                )),
                sprite: TextureAtlasSprite::new(
                    thread_rng().gen_range(0..CLOUD_FRAMES),
                ),
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

    // bedroom door opens (sprite index 2) when the player is near the door
    cmd.spawn((
        Name::from("Bedroom door"),
        DoorBuilder::new(ApartmentTileKind::MainDoorZone)
            .add_open_criteria(DoorOpenCriteria::Character(
                common_story::Character::Winnie,
            ))
            .with_initial_state(DoorState::Closed)
            .with_obstacle_when_closed_between(
                square(-40, -21),
                square(-31, -21),
            )
            .build(&mut tilemap),
        LayoutEntity,
        RenderLayers::layer(render_layer::BG),
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load(assets::BEDROOM_MAIN_DOOR),
                vec2(26.0, 48.0),
                2,
                1,
                None,
                None,
            )),
            sprite: TextureAtlasSprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_translation(
                // sometimes to make the game feel better, the z coordinate
                // needs to be adjusted
                Apartment::extend_z_with_y_offset(vec2(-105.0, -88.0), 8.5),
            ),
            ..default()
        },
    ));

    // the elevator takes the player to the next location
    cmd.spawn((
        Name::from("Elevator"),
        Elevator,
        LayoutEntity,
        HallwayEntity,
        RenderLayers::layer(render_layer::BG),
        // this animation is important for elevator cutscene
        AtlasAnimation {
            on_last_frame: AtlasAnimationEnd::RemoveTimer,
            first: 0,
            last: 7,
            ..default()
        },
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load(assets::ELEVATOR_ATLAS),
                vec2(51.0, 50.0),
                8,
                1,
                Some(vec2(4.0, 0.0)),
                None,
            )),
            sprite: TextureAtlasSprite {
                index: 0,
                color: PRIMARY_COLOR,
                ..default()
            },
            transform: Transform::from_translation(
                vec2(-201.5, -53.0).extend(zindex::ELEVATOR),
            ),
            ..default()
        },
    ));

    cmd.spawn((
        Name::from("Vending machine"),
        LayoutEntity,
        HallwayEntity,
        RenderLayers::layer(render_layer::BG),
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load(assets::VENDING_MACHINE_ATLAS),
                vec2(30.0, 55.0),
                4,
                1,
                Some(vec2(1.0, 0.0)),
                None,
            )),
            sprite: TextureAtlasSprite {
                index: 0,
                color: PRIMARY_COLOR,
                ..default()
            },
            transform: Transform::from_translation(
                vec2(-268.0, -60.0).extend(zindex::ELEVATOR),
            ),
            ..default()
        },
    ));
}

fn despawn(mut cmd: Commands, query: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    for entity in query.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

// TODO: redo
fn smoothly_transition_hallway_color(
    _map: Res<common_top_down::TileMap<Apartment>>,

    player: Query<&Actor, With<Player>>,
    mut sprites: Query<&mut Sprite, With<HallwayEntity>>,
    mut atlases: Query<&mut TextureAtlasSprite, With<HallwayEntity>>,

    // (from color, how long ago started)
    mut local: Local<Option<(Color, Instant)>>,
) {
    let Some(player) = player.get_single_or_none() else {
        return;
    };

    let square = player.current_square();

    let in_hallway = square.y < HALLWAY_TOP_BOUNDARY;
    let by_the_door = true; // TODO: reimplement with zones

    let target_color = if in_hallway || by_the_door {
        Color::WHITE
    } else {
        PRIMARY_COLOR
    };

    // there should always be at least one hallway entity
    // all entities have the same color
    let sprite_color = sprites
        .iter()
        .next()
        .map(|sprite| sprite.color)
        .unwrap_or_default();

    if target_color == sprite_color {
        return;
    }

    let (from_color, transition_started_at) =
        local.get_or_insert_with(|| (sprite_color, Instant::now()));

    if *from_color == target_color {
        *from_color = sprite_color;
        *transition_started_at = Instant::now();
    }

    let elapsed = transition_started_at.elapsed();
    let new_color = if elapsed > HALLWAY_FADE_TRANSITION_DURATION {
        *local = None;
        target_color
    } else {
        from_color.lerp(
            target_color,
            elapsed.as_secs_f32()
                / HALLWAY_FADE_TRANSITION_DURATION.as_secs_f32(),
        )
    };

    for mut sprite in sprites.iter_mut() {
        sprite.color = new_color;
    }
    for mut sprite in atlases.iter_mut() {
        sprite.color = new_color;
    }
}

impl common_top_down::layout::Tile for ApartmentTileKind {
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    fn is_zone(&self) -> bool {
        match self {
            ApartmentTileKind::BedZone
            | ApartmentTileKind::MainDoorZone
            | ApartmentTileKind::ElevatorZone
            | ApartmentTileKind::HallwayZone
            | ApartmentTileKind::MeditationZone
            | ApartmentTileKind::TeaZone => true,
        }
    }
}

impl IntoMap for Apartment {
    type LocalTileKind = ApartmentTileKind;

    fn bounds() -> [i32; 4] {
        [-80, 40, -30, 20]
    }

    fn asset_path() -> &'static str {
        assets::MAP
    }

    fn layout() -> &'static SquareLayout {
        &LAYOUT
    }
}
