use bevy::render::view::RenderLayers;
use bevy_grid_squared::{Square, SquareLayout};
use common_layout::IntoMap;
use common_visuals::{Animation, AnimationEnd, AnimationTimer};
use lazy_static::lazy_static;
use main_game_lib::{vec2_ext::Vec2Ext, PIXEL_ZOOM};

use crate::{cameras::BG_RENDER_LAYER, consts::*, prelude::*};

lazy_static! {
    static ref LAYOUT: SquareLayout = SquareLayout {
        square_size: 4.0,
        origin: vec2(356.0, 175.0).as_top_left_into_centered(),
    };
}

pub(crate) mod zones {
    pub(crate) const MEDITATION: u8 = 0;
    pub(crate) const BED: u8 = 1;
    pub(crate) const TEA: u8 = 2;
    pub(crate) const DOOR: u8 = 3;
}

#[derive(Component, TypePath)]
pub(crate) struct Apartment;

#[derive(Component)]
struct LayoutEntity;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::ApartmentLoading), spawn);
        app.add_systems(OnExit(GlobalGameState::ApartmentQuitting), despawn);
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}

fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    for (name, asset, zindex) in [
        ("Background", assets::BG, zindex::BG),
        (
            "Bedroom back furniture",
            assets::BEDROOM_FURNITURE1,
            zindex::BEDROOM_FURNITURE_DISTANT,
        ),
        (
            "Bedroom cupboard",
            assets::BEDROOM_FURNITURE2,
            zindex::BEDROOM_FURNITURE_MIDDLE,
        ),
        (
            "Bedroom door",
            assets::BEDROOM_FURNITURE3,
            zindex::BEDROOM_FURNITURE_CLOSEST,
        ),
        (
            "Kitchen countertop",
            assets::KITCHEN_FURNITURE1,
            zindex::KITCHEN_FURNITURE_DISTANT,
        ),
        (
            "Kitchen fridge",
            assets::KITCHEN_FURNITURE2,
            zindex::KITCHEN_FURNITURE_MIDDLE,
        ),
        (
            "Kitchen table",
            assets::KITCHEN_FURNITURE3,
            zindex::KITCHEN_FURNITURE_CLOSEST,
        ),
    ] {
        commands.spawn((
            Name::from(name),
            LayoutEntity,
            RenderLayers::layer(BG_RENDER_LAYER),
            SpriteBundle {
                texture: asset_server.load(asset),
                transform: Transform::from_translation(Vec3::new(
                    0.0, 0.0, zindex,
                )),
                ..default()
            },
        ));
    }

    // cloud atlas is rendered on top of the bg but below the furniture
    commands.spawn((
        Name::from("Cloud atlas"),
        LayoutEntity,
        RenderLayers::layer(BG_RENDER_LAYER),
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
}

pub(crate) fn add_z_based_on_y(v: Vec2) -> Vec3 {
    let y = v.y;

    // this is stupid 'n' simple but since the room does not
    // require anything more complex for now, let's roll with it
    v.extend(if y > 40.0 {
        zindex::KITCHEN_FURNITURE_MIDDLE - 0.1
    } else if y > 10.0 {
        zindex::KITCHEN_FURNITURE_MIDDLE + 0.1
    } else if y > -22.0 {
        zindex::BEDROOM_FURNITURE_MIDDLE - 0.1
    } else {
        zindex::BEDROOM_FURNITURE_MIDDLE + 0.1
    })
}

fn despawn(query: Query<Entity, With<LayoutEntity>>, mut commands: Commands) {
    debug!("Despawning layout entities");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

impl IntoMap for Apartment {
    fn bounds() -> [i32; 4] {
        [-40, 40, -20, 20]
    }

    fn asset_path() -> &'static str {
        assets::APARTMENT_MAP
    }

    fn layout() -> &'static SquareLayout {
        &LAYOUT
    }

    fn cursor_position_to_square(p: Vec2) -> Square {
        Self::layout()
            .world_pos_to_square((p / PIXEL_ZOOM).as_top_left_into_centered())
    }
}

impl Apartment {
    pub(crate) fn contains(square: Square) -> bool {
        let [min_x, max_x, min_y, max_y] = Self::bounds();

        square.x >= min_x
            && square.x <= max_x
            && square.y >= min_y
            && square.y <= max_y
    }
}
