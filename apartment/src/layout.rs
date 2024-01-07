use bevy::render::view::RenderLayers;
use bevy_grid_squared::{Square, SquareLayout};
use common_layout::IntoMap;
use lazy_static::lazy_static;
use main_game_lib::{vec2_ext::Vec2Ext, PIXEL_ZOOM};

use crate::{cameras::BG_RENDER_LAYER, prelude::*};

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
struct ApartmentEntity;

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

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    for (asset, zindex) in [
        (assets::BG, zindex::BG),
        (
            assets::BEDROOM_FURNITURE1,
            zindex::BEDROOM_FURNITURE_DISTANT,
        ),
        (assets::BEDROOM_FURNITURE2, zindex::BEDROOM_FURNITURE_MIDDLE),
        (
            assets::BEDROOM_FURNITURE3,
            zindex::BEDROOM_FURNITURE_CLOSEST,
        ),
        (
            assets::KITCHEN_FURNITURE1,
            zindex::KITCHEN_FURNITURE_DISTANT,
        ),
        (assets::KITCHEN_FURNITURE2, zindex::KITCHEN_FURNITURE_MIDDLE),
        (
            assets::KITCHEN_FURNITURE3,
            zindex::KITCHEN_FURNITURE_CLOSEST,
        ),
    ] {
        commands.spawn((
            ApartmentEntity,
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

fn despawn(
    query: Query<Entity, With<ApartmentEntity>>,
    mut commands: Commands,
) {
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
