use bevy_grid_squared::{Square, SquareLayout};
use common_layout::IntoMap;
use lazy_static::lazy_static;
use main_game_lib::{vec2_ext::Vec2Ext, PIXEL_ZOOM};

use crate::prelude::*;

lazy_static! {
    static ref LAYOUT: SquareLayout = SquareLayout {
        square_size: 4.0,
        origin: vec2(356.0, 175.0).from_top_left_into_centered(),
    };
}

#[derive(Component, TypePath)]
pub(crate) struct Apartment;

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
            .world_pos_to_square((p / PIXEL_ZOOM).from_top_left_into_centered())
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
