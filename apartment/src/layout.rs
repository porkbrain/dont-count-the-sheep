use bevy::{render::view::RenderLayers, utils::Instant};
use bevy_grid_squared::{Square, SquareLayout};
use common_visuals::{
    camera::{render_layer, PIXEL_ZOOM},
    Animation, AnimationEnd, AnimationTimer, ColorExt, PRIMARY_COLOR,
};
use lazy_static::lazy_static;
use main_game_lib::{
    common_top_down::{Actor, IntoMap, SquareKind},
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
const HALLWAY_FADE_TRANSITION_DURATION: Duration = from_millis(1500);
/// In terms of map squares.
/// We don't use bounding box because so far the apartment does not
/// extend beyond this boundary.
const HALLWAY_TOP_BOUNDARY: i32 = -20;

pub(crate) mod zones {
    pub(crate) const MEDITATION: u8 = 0;
    pub(crate) const BED: u8 = 1;
    pub(crate) const TEA: u8 = 2;
    pub(crate) const DOOR: u8 = 3;
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
            );
    }
}

#[derive(Component)]
struct LayoutEntity;

/// Hallway is darkened when the player is in the apartment but once the player
/// approaches the door or is in the hallway, it's lit up.
#[derive(Component)]
struct HallwayEntity;

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
            zindex: zindex::BG,
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
            name: "Bedroom door", // TODO
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
            zindex: zindex::HALLWAY,
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
        Animation {
            on_last_frame: AnimationEnd::Loop,
            first: 0,
            last: CLOUD_FRAMES - 1,
        },
        AnimationTimer::new(CLOUD_ATLAS_FRAME_TIME, TimerMode::Repeating),
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
        LayoutEntity,
        RenderLayers::layer(render_layer::BG),
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load(assets::CLOUD_ATLAS),
                vec2(26.0, 48.0),
                2,
                1,
                None,
                None,
            )),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_translation(
                vec2(-20.0, 66.0).as_top_left_into_centered().extend(100.0),
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

fn smoothly_transition_hallway_color(
    map: Res<common_top_down::Map<Apartment>>,

    player: Query<&Actor, With<Player>>,
    mut hallway_entities: Query<&mut Sprite, With<HallwayEntity>>,

    mut local: Local<Option<(Color, Instant)>>,
) {
    let square = player.single().current_square();

    let in_hallway = square.y < HALLWAY_TOP_BOUNDARY;
    let by_the_door =
        matches!(map.get(&square), Some(SquareKind::Zone(zones::DOOR)));

    let target_color = if in_hallway || by_the_door {
        Color::WHITE
    } else {
        PRIMARY_COLOR
    };

    let (local_color, transition_started_at) = local.get_or_insert_with(|| {
        (
            hallway_entities
                .iter()
                .next()
                .map(|sprite| sprite.color)
                // there should always be at least one hallway entity
                .unwrap_or_default(),
            Instant::now(),
        )
    });

    if *local_color != target_color {
        *local_color = target_color;
        *transition_started_at = Instant::now();
    }

    let elapsed = transition_started_at.elapsed();
    if elapsed > HALLWAY_FADE_TRANSITION_DURATION {
        return;
    }

    for mut sprite in hallway_entities.iter_mut() {
        sprite.color = sprite.color.lerp(
            *local_color,
            elapsed.as_secs_f32()
                / HALLWAY_FADE_TRANSITION_DURATION.as_secs_f32(),
        );
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
        } else if y > -80.0 {
            zindex::BEDROOM_FURNITURE_MIDDLE + 0.1
        } else {
            zindex::HALLWAY + 0.1
        })
    }
}
