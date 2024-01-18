use bevy::{
    ecs::event::event_update_condition, render::view::RenderLayers,
    utils::Instant,
};
use bevy_grid_squared::{Square, SquareLayout};
use common_visuals::{
    camera::{render_layer, PIXEL_ZOOM},
    AtlasAnimation, AtlasAnimationEnd, AtlasAnimationTimer, ColorExt,
    PRIMARY_COLOR,
};
use lazy_static::lazy_static;
use main_game_lib::{
    common_top_down::{actor, Actor, ActorMovementEvent, IntoMap, SquareKind},
    vec2_ext::Vec2Ext,
};

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

pub(crate) mod zones {
    use main_game_lib::common_top_down::layout::Zone;

    pub(crate) const MEDITATION: Zone = 0;
    pub(crate) const BED: Zone = 1;
    pub(crate) const TEA: Zone = 2;
    pub(crate) const DOOR: Zone = 3;
    pub(crate) const ELEVATOR: Zone = 4;
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::ApartmentLoading), spawn)
            .add_systems(OnExit(GlobalGameState::ApartmentQuitting), despawn)
            .add_systems(
                Update,
                smoothly_transition_hallway_color
                    .run_if(in_state(GlobalGameState::InApartment)),
            )
            .add_systems(
                Update,
                toggle_door
                    .run_if(in_state(GlobalGameState::InApartment))
                    .run_if(event_update_condition::<ActorMovementEvent>)
                    .after(actor::emit_movement_events::<Apartment>),
            );
    }
}

#[derive(Component)]
struct LayoutEntity;

/// Hallway is darkened when the player is in the apartment but once the player
/// approaches the door or is in the hallway, it's lit up.
#[derive(Component)]
struct HallwayEntity;

/// The main door is a special entity that has a sprite sheet with two frames.
/// When the player is near the door, the door opens.
#[derive(Component)]
struct MainDoor;

/// Elevator is a special entity that has a sprite sheet with several frames.
/// It opens when an actor is near it and closes when the actor leaves or
/// enters.
#[derive(Component)]
pub(crate) struct Elevator;

fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    #[derive(Default)]
    struct ToSpawn {
        name: &'static str,
        asset: &'static str,
        zindex: f32,
        color: Option<Color>,
        is_hallway_entity: bool,
    }

    for ToSpawn {
        name,
        asset,
        zindex,
        color,
        is_hallway_entity,
    } in [
        ToSpawn {
            name: "Background",
            asset: assets::BG,
            zindex: zindex::BG_ROOM_AND_KITCHEN,
            ..default()
        },
        ToSpawn {
            name: "Bedroom back furniture",
            asset: assets::BEDROOM_FURNITURE1,
            zindex: zindex::BEDROOM_FURNITURE_DISTANT,
            ..default()
        },
        ToSpawn {
            name: "Bedroom cupboard",
            asset: assets::BEDROOM_FURNITURE2,
            zindex: zindex::BEDROOM_FURNITURE_MIDDLE,
            ..default()
        },
        ToSpawn {
            name: "Bedroom shoe rack",
            asset: assets::BEDROOM_FURNITURE3,
            zindex: zindex::BEDROOM_FURNITURE_CLOSEST,
            ..default()
        },
        ToSpawn {
            name: "Kitchen countertop",
            asset: assets::KITCHEN_FURNITURE1,
            zindex: zindex::KITCHEN_FURNITURE_DISTANT,
            ..default()
        },
        ToSpawn {
            name: "Kitchen fridge",
            asset: assets::KITCHEN_FURNITURE2,
            zindex: zindex::KITCHEN_FURNITURE_MIDDLE,
            ..default()
        },
        ToSpawn {
            name: "Kitchen table",
            asset: assets::KITCHEN_FURNITURE3,
            zindex: zindex::KITCHEN_FURNITURE_CLOSEST,
            ..default()
        },
        ToSpawn {
            name: "Hallway",
            asset: assets::HALLWAY,
            zindex: zindex::BG_HALLWAY,
            color: Some(PRIMARY_COLOR),
            is_hallway_entity: true,
            ..default()
        },
        ToSpawn {
            name: "Hallway doors",
            asset: assets::HALLWAY_DOORS,
            zindex: zindex::HALLWAY_DOORS,
            color: Some(PRIMARY_COLOR),
            is_hallway_entity: true,
            ..default()
        },
    ] {
        let mut entity = cmd.spawn((
            Name::from(name),
            LayoutEntity,
            RenderLayers::layer(render_layer::BG),
            SpriteBundle {
                texture: asset_server.load(asset),
                transform: Transform::from_translation(Vec3::new(
                    0.0, 0.0, zindex,
                )),
                sprite: Sprite {
                    color: color.unwrap_or_default(),
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
    cmd.spawn((
        Name::from("Cloud atlas"),
        LayoutEntity,
        RenderLayers::layer(render_layer::BG),
        AtlasAnimation {
            on_last_frame: AtlasAnimationEnd::Loop,
            first: 0,
            last: CLOUD_FRAMES - 1,
            ..default()
        },
        AtlasAnimationTimer::new(CLOUD_ATLAS_FRAME_TIME, TimerMode::Repeating),
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load(assets::CLOUD_ATLAS),
                vec2(CLOUD_WIDTH, CLOUD_HEIGHT),
                CLOUD_FRAMES,
                1,
                Some(vec2(CLOUD_PADDING, 0.0)),
                None,
            )),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_translation(
                vec2(249.5, 66.5)
                    .as_top_left_into_centered()
                    .extend(zindex::CLOUD_ATLAS),
            ),
            ..default()
        },
    ));

    // bedroom door opens (sprite index 2) when the player is near the door
    cmd.spawn((
        Name::from("Bedroom door"),
        MainDoor,
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
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_translation(
                vec2(-105.0, -63.0).extend(zindex::BEDROOM_FURNITURE_CLOSEST),
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
}

/// When player gets near the door, the door opens.
fn toggle_door(
    mut events: EventReader<ActorMovementEvent>,

    mut door: Query<&mut TextureAtlasSprite, With<MainDoor>>,

    mut door_opened: Local<bool>,
) {
    let mut new_door_opened = *door_opened;
    for event in events.read().filter(|event| event.is_player()) {
        match event {
            ActorMovementEvent::ZoneEntered { zone, .. }
                if *zone == zones::DOOR =>
            {
                new_door_opened = true;
            }
            ActorMovementEvent::ZoneLeft { zone, .. }
                if *zone == zones::DOOR =>
            {
                new_door_opened = false;
            }
            _ => {}
        }
    }

    if new_door_opened != *door_opened {
        *door_opened = new_door_opened;
        door.single_mut().index = if *door_opened { 1 } else { 0 };
    }
}

fn despawn(mut cmd: Commands, query: Query<Entity, With<LayoutEntity>>) {
    debug!("Despawning layout entities");

    for entity in query.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

fn smoothly_transition_hallway_color(
    map: Res<common_top_down::Map<Apartment>>,

    player: Query<&Actor, With<Player>>,
    mut sprites: Query<&mut Sprite, With<HallwayEntity>>,
    mut atlases: Query<&mut TextureAtlasSprite, With<HallwayEntity>>,

    // (from color, how long ago started)
    mut local: Local<Option<(Color, Instant)>>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    let square = player.current_square();

    let in_hallway = square.y < HALLWAY_TOP_BOUNDARY;
    let by_the_door =
        matches!(map.get(&square), Some(SquareKind::Zone(zones::DOOR)));

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

impl IntoMap for Apartment {
    fn bounds() -> [i32; 4] {
        [-70, 40, -30, 20]
    }

    fn asset_path() -> &'static str {
        assets::MAP
    }

    fn layout() -> &'static SquareLayout {
        &LAYOUT
    }

    fn cursor_position_to_square(p: Vec2) -> Square {
        Self::layout().world_pos_to_square(
            (p / PIXEL_ZOOM as f32).as_top_left_into_centered(),
        )
    }

    fn extend_z(v: Vec2) -> Vec3 {
        let y = v.y;

        // this is stupid 'n' simple but since the room does not
        // require anything more complex for now, let's roll with it
        v.extend(if y > 40.0 {
            zindex::KITCHEN_FURNITURE_MIDDLE - 0.1
        } else if y > 10.0 {
            zindex::KITCHEN_FURNITURE_MIDDLE + 0.1
        } else if y > -22.0 {
            zindex::BEDROOM_FURNITURE_MIDDLE - 0.1
        } else if y > -78.0 {
            zindex::BEDROOM_FURNITURE_MIDDLE + 0.1
        } else {
            zindex::BEDROOM_FURNITURE_CLOSEST + 0.1
        })
    }
}
